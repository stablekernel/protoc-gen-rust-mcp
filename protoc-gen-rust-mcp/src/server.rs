//! Generates, for one gRPC service, the MCP server struct and its tool
//! definitions, handlers and registration API: the counterpart of Go's
//! `generateMcpServerStruct`, `generateMcpServerService`, `generateMCPTool`,
//! `generateHandler`, `generateToolRegistration` and
//! `generateDefaultToolsRegistration` (`cmd/protoc-gen-go-mcp/mcp.go` at
//! commit `9e072f9`). `method_handler_fn` (this issue, #7) emits the real
//! handler body: validate the raw arguments against the tool's input
//! schema (the validation Go's `mcp.AddTool` does, which `rmcp` itself
//! does not), decode them with the pbjson-generated `Deserialize`, call
//! the tonic client, and encode the response with the pbjson-generated
//! `Serialize`. `method_public_handler_fn` emits the public
//! `pub async fn <method>_handler(&self, args)` wrapper Go also exports,
//! so a caller can delegate to the default behavior from an overriding
//! tool.
//!
//! # Argument validation, decoding, the tonic call, and the response
//!
//! - **Validation** happens first, against a `jsonschema::Validator`
//!   compiled once per tool from the same schema embedded in the tool's
//!   `inputSchema` (a package-level `static ..._VALIDATOR: LazyLock`, next
//!   to the existing `..._INPUT_SCHEMA` constant). `None` arguments are
//!   treated as `{}` (an empty object), matching Go's SDK (`applySchema`
//!   unmarshals missing/`null` arguments into an empty map before
//!   validating), since every input schema this generator emits has
//!   `"type": "object"` and no required top-level properties (RPCs take
//!   exactly one message argument, never scalars). On failure, every
//!   failing instance path and reason is listed in the tool error text,
//!   and the backend is never called.
//! - **Decoding** uses `serde_json::from_value::<Request>`, the pbjson-
//!   generated counterpart of Go's `protojson.Unmarshal`. A decode error
//!   (schema validation already ruled out most of these, but pbjson's
//!   `Deserialize` is independent code with its own opinions, e.g. an
//!   out-of-range number for a sized integer type) becomes a tool error
//!   without calling the backend.
//! - **The tonic call** clones the handler's own client (already captured
//!   per-registration, not per-call; see the module doc comment below) and
//!   calls it with `tonic::Request::new(request)`. A `tonic::Status`
//!   becomes a tool error formatted like grpc-go's `Status.Error()`:
//!   `"rpc error: code = {code:?} desc = {message}"` — `tonic::Code`'s
//!   `Debug` impl renders exactly grpc-go's CamelCase names (`NotFound`,
//!   `InvalidArgument`, ...), confirmed against `tonic::Code` 0.14.6's
//!   definition, so no separate name table is needed here.
//! - **Encoding** uses `serde_json::to_string`, the pbjson-generated
//!   counterpart of Go's `protojson.Marshal`: zero-valued fields are
//!   omitted, 64-bit integers are JSON strings, and enums are field
//!   names, matching protojson's own conventions on every point except
//!   one documented divergence: pbjson serializes a non-finite
//!   float/double as JSON `null`, where protojson emits the string
//!   `"NaN"`/`"Infinity"`/`"-Infinity"`; this only affects tool *results*
//!   (`float_schema` in `schema.rs` already documents that inputs accept
//!   all three spellings either way). Reported in this issue's PR per
//!   review feedback on #18, and pinned by `examples/tests/handler.rs`'s
//!   `non_finite_floats_in_result_are_encoded_as_json_null`.
//!
//! # Shape
//!
//! Go passes one shared `*mcp.Server` to every `New<Service>MCPServer` call
//! and registers tools on it directly. `rmcp`'s own composition primitive
//! for that, `ToolRouter<S>`, is parameterized by the server's own `Self`
//! type, and its `ToolCallContext` borrows `&S` for the duration of a
//! call; threading that through a generated, per-service struct that also
//! wants to be its own `ServerHandler` produces self-referential lifetimes
//! with no clean solution (confirmed by prototyping against rmcp 3.5.1,
//! see this issue's discussion). This generator instead keeps a
//! `BTreeMap<String, (Tool, ToolHandler)>` on the generated struct and
//! implements `rmcp::ServerHandler`'s `get_info`/`list_tools`/
//! `call_tool`/`get_tool` by hand:
//!
//! - **Handlers capture a cloned tonic client, not `self`.** Each handler
//!   is `Arc<dyn Fn(Option<JsonObject>) -> BoxFuture<'static,
//!   CallToolResult> + Send + Sync>`, built from an `async fn(client,
//!   args) -> CallToolResult` associated function with a cloned client
//!   captured in the closure. That keeps every future `'static` with no
//!   `Arc` cycle back through the `tools` map.
//! - **Composition across services:** `into_tools()` hands a service's
//!   registered `(Tool, ToolHandler)` pairs to another generated server
//!   via `extend_tools()`, so several services' tools can be served from
//!   one `impl ServerHandler`, mirroring Go's "pass the same `*mcp.Server`
//!   to several `New<Service>MCPServer` calls".
//! - **Unknown tool:** `ErrorData::invalid_params("tool not found", None)`,
//!   the same message `rmcp`'s own `ToolRouter::call` uses for a miss, so
//!   a caller sees the same error either way.
//! - **`register_tool`** lets a caller override or add a single tool by
//!   name (last write wins, like `BTreeMap::insert`), matching Go's
//!   `RegisterTool`.

use std::fmt::Write as _;

use heck::{ToShoutySnakeCase, ToSnakeCase, ToUpperCamelCase};
use prost_reflect::{MessageDescriptor, MethodDescriptor, ServiceDescriptor};
use serde_json::Value;

use crate::comments::{camel_to_space, process_comment_to_string};
use crate::rust_literal::rust_string_literal;
use crate::schema::message_input_schema;
use crate::source_info::leading_comments;

/// The bounds this generator puts on a generated server's `T` type
/// parameter: exactly what `VibeServiceClient<T>::unary` (tonic-generated,
/// see e.g. `examples.v1.tonic.rs`) needs to be callable, plus `Clone +
/// Send + Sync + 'static` so the generated struct itself is `Send + Sync`
/// (required by `rmcp::ServerHandler`) and so a handler can clone the
/// client into a `'static` future. Verified against rmcp 3.5.1 and tonic
/// 0.14.6 by compiling and running the generated code's handler against a
/// real tonic server (see this issue's discussion).
const CLIENT_BOUNDS: &str = "\
    T: ::tonic::client::GrpcService<::tonic::body::Body> + ::std::clone::Clone + ::std::marker::Send + ::std::marker::Sync + 'static,\n\
    \u{20}   T::Error: ::std::convert::Into<::tonic::codegen::StdError>,\n\
    \u{20}   T::ResponseBody: ::tonic::codegen::Body<Data = ::tonic::codegen::Bytes> + ::std::marker::Send + 'static,\n\
    \u{20}   <T::ResponseBody as ::tonic::codegen::Body>::Error: ::std::convert::Into<::tonic::codegen::StdError> + ::std::marker::Send,\n\
    \u{20}   T::Future: ::std::marker::Send,\n\
";

/// Renders the full generated module for one service: the input schema
/// and validator constants, the tool handler type alias, the server
/// struct, its `impl` block (constructor, one `<method>_tool()`, handler
/// and public `<method>_handler()` wrapper per unary RPC, `register_tool`,
/// `register_default_tools`, `into_tools`/`extend_tools`), and its
/// `rmcp::ServerHandler` impl. Streaming RPCs are skipped, with a comment
/// noting each one skipped (per the issue).
pub fn generate_service(service: &ServiceDescriptor) -> String {
    let service_name = service.name();
    let server_name = format!("{service_name}McpServer");
    let client_name = format!("{service_name}Client");
    let client_mod = client_mod_name(service);
    let client_path = format!("{client_mod}::{client_name}");
    let handler_type = format!("{server_name}ToolHandler");

    let mut unary_methods = Vec::new();
    let mut skipped = Vec::new();
    for method in service.methods() {
        if method.is_client_streaming() || method.is_server_streaming() {
            skipped.push(method.name().to_string());
        } else {
            unary_methods.push(method);
        }
    }

    let mut out = String::new();

    for name in &skipped {
        let _ = writeln!(
            out,
            "// Skipping {name}: client- or server-streaming RPCs are not supported as MCP tools."
        );
    }
    if !skipped.is_empty() {
        out.push('\n');
    }

    for method in &unary_methods {
        out.push_str(&schema_constant(service_name, method));
        out.push('\n');
        out.push_str(&validator_constant(service_name, method));
        out.push('\n');
    }

    let _ = write!(
        out,
        "/// A registered tool's handler: takes the raw tool call arguments\n\
         /// (already looked up by name), returns the result. Built once per\n\
         /// registered tool from an `async fn(client, args) -> CallToolResult`\n\
         /// associated function so every handler future is `'static` without an\n\
         /// `Arc` cycle back through the `tools` map (see the module doc comment\n\
         /// on why this, not `rmcp::ToolRouter`).\n\
         pub type {handler_type} = ::std::sync::Arc<\n\
         \u{20}   dyn Fn(\n\
         \u{20}       ::std::option::Option<::rmcp::model::JsonObject>,\n\
         \u{20}   ) -> ::futures::future::BoxFuture<'static, ::rmcp::model::CallToolResult>\n\
         \u{20}       + ::std::marker::Send\n\
         \u{20}       + ::std::marker::Sync,\n\
         >;\n\n"
    );

    let _ = write!(
        out,
        "/// The MCP server for the `{service_name}` gRPC service: exposes each of\n\
         /// its non-streaming RPCs as an MCP tool and forwards tool calls to a\n\
         /// `{client_name}<T>`. No tools are registered until\n\
         /// [`{server_name}::register_default_tools`] or\n\
         /// [`{server_name}::register_tool`] is called, matching the Go plugin.\n\
         #[derive(Clone)]\n\
         pub struct {server_name}<T> {{\n\
         \u{20}   client: {client_path}<T>,\n\
         \u{20}   tools: ::std::collections::BTreeMap<::std::string::String, (::rmcp::model::Tool, {handler_type})>,\n\
         \u{20}   server_info: ::rmcp::model::Implementation,\n\
         }}\n\n"
    );

    let _ = write!(
        out,
        "impl<T> {server_name}<T>\n\
         where\n\
         \u{20}   {CLIENT_BOUNDS}\
         {{\n"
    );
    let _ = write!(
        out,
        "    /// Creates a new `{server_name}` with no tools registered yet; call\n\
         \u{20}   /// [`register_default_tools`](Self::register_default_tools) or\n\
         \u{20}   /// [`register_tool`](Self::register_tool) to add some.\n\
         \u{20}   pub fn new(client: {client_path}<T>) -> Self {{\n\
         \u{20}       Self {{\n\
         \u{20}           client,\n\
         \u{20}           tools: ::std::collections::BTreeMap::new(),\n\
         \u{20}           server_info: ::rmcp::model::Implementation::from_build_env(),\n\
         \u{20}       }}\n\
         \u{20}   }}\n\n"
    );
    out.push_str(
        "    /// Overrides the `serverInfo` advertised by `get_info` (defaults to this\n\
         \u{20}   /// crate's name and version).\n\
         \u{20}   pub fn with_server_info(mut self, server_info: ::rmcp::model::Implementation) -> Self {\n\
         \u{20}       self.server_info = server_info;\n\
         \u{20}       self\n\
         \u{20}   }\n\n",
    );

    for method in &unary_methods {
        out.push_str(&method_tool_fn(service_name, method));
        out.push('\n');
        out.push_str(&method_handler_fn(service_name, &client_path, method));
        out.push('\n');
        out.push_str(&method_public_handler_fn(method));
        out.push('\n');
    }

    let _ = write!(
        out,
        "    /// Registers a single tool, overriding any existing tool with the same\n\
         \u{20}   /// name (callers can also use this to add a tool this generator did\n\
         \u{20}   /// not emit).\n\
         \u{20}   pub fn register_tool<F, Fut>(&mut self, tool: ::rmcp::model::Tool, handler: F) -> &mut Self\n\
         \u{20}   where\n\
         \u{20}       F: Fn(::std::option::Option<::rmcp::model::JsonObject>) -> Fut\n\
         \u{20}           + ::std::marker::Send\n\
         \u{20}           + ::std::marker::Sync\n\
         \u{20}           + 'static,\n\
         \u{20}       Fut: ::std::future::Future<Output = ::rmcp::model::CallToolResult> + ::std::marker::Send + 'static,\n\
         \u{20}   {{\n\
         \u{20}       let handler: {handler_type} = ::std::sync::Arc::new(move |args| ::std::boxed::Box::pin(handler(args)));\n\
         \u{20}       self.tools.insert(tool.name.to_string(), (tool, handler));\n\
         \u{20}       self\n\
         \u{20}   }}\n\n"
    );

    out.push_str(
        "    /// Registers every unary RPC's tool with its default handler, the\n\
         \u{20}   /// counterpart of Go's `RegisterDefaultTools`.\n\
         \u{20}   pub fn register_default_tools(&mut self) -> &mut Self {\n",
    );
    for method in &unary_methods {
        let fn_name = method_fn_name(method);
        let _ = write!(
            out,
            "        {{\n\
             \u{20}           let client = self.client.clone();\n\
             \u{20}           self.register_tool(Self::{fn_name}_tool(), move |args| Self::call_{fn_name}(client.clone(), args));\n\
             \u{20}       }}\n"
        );
    }
    out.push_str("        self\n    }\n\n");

    let _ = write!(
        out,
        "    /// Hands this server's registered tools to another generated server,\n\
         \u{20}   /// so one MCP server can serve several services' tools (pass the\n\
         \u{20}   /// result to [`extend_tools`](Self::extend_tools) on the other server).\n\
         \u{20}   pub fn into_tools(self) -> impl ::std::iter::Iterator<Item = (::rmcp::model::Tool, {handler_type})> {{\n\
         \u{20}       self.tools.into_values()\n\
         \u{20}   }}\n\n"
    );
    let _ = write!(
        out,
        "    /// Adds tools (e.g. from another service's\n\
         \u{20}   /// [`into_tools`](Self::into_tools)) to this server, overriding any\n\
         \u{20}   /// existing tool with the same name.\n\
         \u{20}   pub fn extend_tools(\n\
         \u{20}       &mut self,\n\
         \u{20}       tools: impl ::std::iter::IntoIterator<Item = (::rmcp::model::Tool, {handler_type})>,\n\
         \u{20}   ) -> &mut Self {{\n\
         \u{20}       for (tool, handler) in tools {{\n\
         \u{20}           self.tools.insert(tool.name.to_string(), (tool, handler));\n\
         \u{20}       }}\n\
         \u{20}       self\n\
         \u{20}   }}\n\
         }}\n\n"
    );

    let _ = write!(
        out,
        "impl<T> ::rmcp::ServerHandler for {server_name}<T>\n\
         where\n\
         \u{20}   {CLIENT_BOUNDS}\
         {{\n\
         \u{20}   fn get_info(&self) -> ::rmcp::model::ServerConfig {{\n\
         \u{20}       ::rmcp::model::ServerConfig::new(\n\
         \u{20}           ::rmcp::model::ServerCapabilities::builder().enable_tools().build(),\n\
         \u{20}       )\n\
         \u{20}       .with_server_info(self.server_info.clone())\n\
         \u{20}   }}\n\n\
         \u{20}   async fn list_tools(\n\
         \u{20}       &self,\n\
         \u{20}       _request: ::std::option::Option<::rmcp::model::PaginatedRequestParams>,\n\
         \u{20}       _context: ::rmcp::service::RequestContext<::rmcp::RoleServer>,\n\
         \u{20}   ) -> ::std::result::Result<::rmcp::model::ListToolsResult, ::rmcp::ErrorData> {{\n\
         \u{20}       Ok(::rmcp::model::ListToolsResult::with_all_items(\n\
         \u{20}           self.tools.values().map(|(tool, _)| tool.clone()).collect(),\n\
         \u{20}       ))\n\
         \u{20}   }}\n\n\
         \u{20}   async fn call_tool(\n\
         \u{20}       &self,\n\
         \u{20}       request: ::rmcp::model::CallToolRequestParams,\n\
         \u{20}       _context: ::rmcp::service::RequestContext<::rmcp::RoleServer>,\n\
         \u{20}   ) -> ::std::result::Result<::rmcp::model::CallToolResponse, ::rmcp::ErrorData> {{\n\
         \u{20}       let (_, handler) = self\n\
         \u{20}           .tools\n\
         \u{20}           .get(request.name.as_ref())\n\
         \u{20}           .ok_or_else(|| ::rmcp::ErrorData::invalid_params(\"tool not found\", None))?;\n\
         \u{20}       Ok(handler(request.arguments).await.into())\n\
         \u{20}   }}\n\n\
         \u{20}   fn get_tool(&self, name: &str) -> ::std::option::Option<::rmcp::model::Tool> {{\n\
         \u{20}       self.tools.get(name).map(|(tool, _)| tool.clone())\n\
         \u{20}   }}\n\
         }}\n"
    );

    out
}

/// The name of the package-level `static ..._INPUT_SCHEMA` constant for
/// one method, prefixed with its service's name so that two services in
/// the same proto package (and so the same generated `.mcp.rs` file) that
/// happen to share a method name (e.g. both have a `Get` RPC) don't
/// collide: `VIBE_SERVICE_SET_VIBE_INPUT_SCHEMA`, not `SET_VIBE_INPUT_SCHEMA`.
fn schema_const_name(service_name: &str, method: &MethodDescriptor) -> String {
    format!(
        "{}_{}_INPUT_SCHEMA",
        service_name.to_shouty_snake_case(),
        method.name().to_shouty_snake_case()
    )
}

/// The `static <SERVICE>_<METHOD>_INPUT_SCHEMA: LazyLock<Arc<JsonObject>>`
/// declaration for one method, parsed once from a JSON literal embedding
/// the schema `message_input_schema` builds for the method's input
/// message (the issue's "embedded as a JSON literal parsed once").
fn schema_constant(service_name: &str, method: &MethodDescriptor) -> String {
    let const_name = schema_const_name(service_name, method);
    let schema: Value = message_input_schema(&method.input());
    let schema_json = serde_json::to_string(&schema)
        .expect("schema is built entirely out of maps, slices and JSON-safe scalars");
    let literal = rust_string_literal(&schema_json);
    format!(
        "static {const_name}: ::std::sync::LazyLock<::std::sync::Arc<::rmcp::model::JsonObject>> =\n    \
         ::std::sync::LazyLock::new(|| {{\n        \
         ::std::sync::Arc::new(\n            \
         match ::serde_json::from_str({literal}) {{\n                \
         Ok(::serde_json::Value::Object(map)) => map,\n                \
         Ok(_) => unreachable!(\"input schema is always a JSON object\"),\n                \
         Err(e) => panic!(\"parsing generated input schema: {{e}}\"),\n            \
         }},\n        \
         )\n    \
         }});\n"
    )
}

/// The `fn <method>_tool() -> rmcp::model::Tool` associated function for
/// one method: its name (the RPC's proto method name as written), its
/// description (leading comment, falling back to `camel_to_space`), and
/// the schema constant as its input schema.
fn method_tool_fn(service_name: &str, method: &MethodDescriptor) -> String {
    let method_name = method.name();
    let fn_name = method_fn_name(method);
    let description = method_description(method);
    let const_name = schema_const_name(service_name, method);
    let name_lit = rust_string_literal(method_name);
    let desc_lit = rust_string_literal(&description);
    format!(
        "    /// Builds the [`rmcp::model::Tool`] for `{method_name}`.\n\
         \u{20}   pub fn {fn_name}_tool() -> ::rmcp::model::Tool {{\n\
         \u{20}       ::rmcp::model::Tool::new({name_lit}, {desc_lit}, {const_name}.clone())\n\
         \u{20}   }}\n"
    )
}

/// The method's description: its leading proto comment, processed, or
/// `camel_to_space` of its name if it has none.
fn method_description(method: &MethodDescriptor) -> String {
    let path = method.path();
    let parent_file = method.parent_file();
    let file = parent_file.file_descriptor_proto();
    let raw = leading_comments(file.source_code_info.as_ref(), path);
    if raw.is_empty() {
        camel_to_space(method.name())
    } else {
        process_comment_to_string(raw)
    }
}

/// The name of the package-level `static ..._VALIDATOR: LazyLock<Validator>`
/// constant for one method, named like [`schema_const_name`] but with a
/// `_VALIDATOR` suffix instead of `_INPUT_SCHEMA`, so the two don't
/// collide.
fn validator_const_name(service_name: &str, method: &MethodDescriptor) -> String {
    format!(
        "{}_{}_VALIDATOR",
        service_name.to_shouty_snake_case(),
        method.name().to_shouty_snake_case()
    )
}

/// The `static <SERVICE>_<METHOD>_VALIDATOR: LazyLock<jsonschema::Validator>`
/// declaration for one method, compiled once from the method's own
/// `..._INPUT_SCHEMA` constant: the validator `method_handler_fn`'s body
/// checks a tool call's raw arguments against before decoding them, the
/// validation Go's `mcp.AddTool` does and `rmcp` itself does not (this
/// issue).
fn validator_constant(service_name: &str, method: &MethodDescriptor) -> String {
    let schema_const = schema_const_name(service_name, method);
    let validator_const = validator_const_name(service_name, method);
    format!(
        "static {validator_const}: ::std::sync::LazyLock<::jsonschema::Validator> =\n    \
         ::std::sync::LazyLock::new(|| {{\n        \
         ::jsonschema::validator_for(&::serde_json::Value::Object({schema_const}.as_ref().clone()))\n            \
         .unwrap_or_else(|e| panic!(\"compiling generated input schema: {{e}}\"))\n    \
         }});\n"
    )
}

/// The real handler for one method (this issue, #7): a private async
/// associated function that takes an owned client and the raw tool call
/// arguments (`None` treated as `{{}}`), and:
///
/// 1. Validates them against the method's `..._VALIDATOR`, returning a
///    tool error listing every failing instance path and reason (backend
///    not called) on failure.
/// 2. Decodes them into the request message with
///    `serde_json::from_value` (the pbjson-generated `Deserialize`),
///    returning a tool error on failure (backend not called).
/// 3. Calls the tonic client, returning a tool error formatted like
///    grpc-go's `Status.Error()` on a `tonic::Status`.
/// 4. Encodes the response with `serde_json::to_string` (the
///    pbjson-generated `Serialize`) and returns it as a successful tool
///    result.
///
/// Named `call_<method>` (not `<method>_handler`, which is the public
/// wrapper in [`method_public_handler_fn`]) because `register_default_tools`
/// builds its closures from this one directly, cloning `self.client` once
/// per registration rather than once per call.
fn method_handler_fn(service_name: &str, client_path: &str, method: &MethodDescriptor) -> String {
    let method_name = method.name();
    let fn_name = method_fn_name(method);
    let validator_const = validator_const_name(service_name, method);
    let request_path = rust_message_path(&method.input());
    format!(
        "    /// Validates, decodes, calls the backend for, and encodes the\n\
         \u{20}   /// response of, a `{method_name}` tool call: see this module's doc\n\
         \u{20}   /// comment for the exact steps.\n\
         \u{20}   async fn call_{fn_name}(\n\
         \u{20}       mut client: {client_path}<T>,\n\
         \u{20}       args: ::std::option::Option<::rmcp::model::JsonObject>,\n\
         \u{20}   ) -> ::rmcp::model::CallToolResult {{\n\
         \u{20}       let args = ::serde_json::Value::Object(args.unwrap_or_default());\n\
         \u{20}       let errors: ::std::vec::Vec<::std::string::String> = {validator_const}\n\
         \u{20}           .iter_errors(&args)\n\
         \u{20}           .map(|e| ::std::format!(\"{{}}: {{e}}\", e.instance_path()))\n\
         \u{20}           .collect();\n\
         \u{20}       if !errors.is_empty() {{\n\
         \u{20}           return ::rmcp::model::CallToolResult::error(vec![::rmcp::model::ContentBlock::text(\n\
         \u{20}               ::std::format!(\"invalid arguments: {{}}\", errors.join(\"; \")),\n\
         \u{20}           )]);\n\
         \u{20}       }}\n\
         \u{20}       let request: {request_path} = match ::serde_json::from_value(args) {{\n\
         \u{20}           Ok(r) => r,\n\
         \u{20}           Err(e) => {{\n\
         \u{20}               return ::rmcp::model::CallToolResult::error(vec![::rmcp::model::ContentBlock::text(\n\
         \u{20}                   ::std::format!(\"decoding arguments: {{e}}\"),\n\
         \u{20}               )]);\n\
         \u{20}           }}\n\
         \u{20}       }};\n\
         \u{20}       match client.{fn_name}(::tonic::Request::new(request)).await {{\n\
         \u{20}           Ok(response) => match ::serde_json::to_string(response.get_ref()) {{\n\
         \u{20}               Ok(json) => ::rmcp::model::CallToolResult::success(vec![::rmcp::model::ContentBlock::text(json)]),\n\
         \u{20}               Err(e) => ::rmcp::model::CallToolResult::error(vec![::rmcp::model::ContentBlock::text(\n\
         \u{20}                   ::std::format!(\"encoding response: {{e}}\"),\n\
         \u{20}               )]),\n\
         \u{20}           }},\n\
         \u{20}           // grpc-go's Status.Error() format, which Go's generated server\n\
         \u{20}           // returns verbatim as the tool error text.\n\
         \u{20}           Err(status) => ::rmcp::model::CallToolResult::error(vec![::rmcp::model::ContentBlock::text(\n\
         \u{20}               ::std::format!(\"rpc error: code = {{:?}} desc = {{}}\", status.code(), status.message()),\n\
         \u{20}           )]),\n\
         \u{20}       }}\n\
         \u{20}   }}\n"
    )
}

/// The public `pub async fn <method>_handler(&self, args) -> CallToolResult`
/// associated function for one method: exported (like Go's `XxxHandler`
/// methods) so a caller can wrap a default handler, e.g. register an
/// overriding tool that pre-processes arguments and then delegates to this
/// one, which `register_default_tools` itself does not need (it calls the
/// private [`method_handler_fn`] directly with a clone of `self.client`
/// made once per registration, not once per call).
fn method_public_handler_fn(method: &MethodDescriptor) -> String {
    let method_name = method.name();
    let fn_name = method_fn_name(method);
    format!(
        "    /// Calls the handler for `{method_name}` with a clone of this\n\
         \u{20}   /// server's client. Exported so a caller can wrap it, e.g. an\n\
         \u{20}   /// overriding tool (via [`register_tool`](Self::register_tool)) that\n\
         \u{20}   /// pre-processes arguments and then delegates here.\n\
         \u{20}   pub async fn {fn_name}_handler(\n\
         \u{20}       &self,\n\
         \u{20}       args: ::std::option::Option<::rmcp::model::JsonObject>,\n\
         \u{20}   ) -> ::rmcp::model::CallToolResult {{\n\
         \u{20}       Self::call_{fn_name}(self.client.clone(), args).await\n\
         \u{20}   }}\n"
    )
}

/// The snake_case Rust identifier fragment for `method`'s generated
/// `<fn_name>_tool`/`<fn_name>_handler` associated functions (e.g.
/// `set_vibe` for the `SetVibe` RPC), matching `prost-build`'s own method
/// naming ([`heck::ToSnakeCase`], the same case conversion
/// `prost_build::ident::to_snake` uses) so the generated code reads like
/// the sibling tonic client's own method names. The tool's wire `name`
/// (used in [`method_tool_fn`]) is always the proto method name as
/// written, unaffected by this.
fn method_fn_name(method: &MethodDescriptor) -> String {
    method.name().to_snake_case()
}

/// Returns the Rust module name tonic-build/protoc-gen-tonic generates for
/// `service`'s client (e.g. `vibe_service_client` for `VibeService`),
/// matching `protoc-gen-tonic`'s own `naive_snake_case` (`to_snake` in
/// `tonic-build`): lowercase with an underscore inserted before each
/// uppercase letter, which agrees with [`heck::ToSnakeCase`] on every
/// identifier this plugin's own test data and `example.proto` use.
pub fn client_mod_name(service: &ServiceDescriptor) -> String {
    format!("{}_client", service.name().to_snake_case())
}

/// Returns the Rust path `prost-build` generates for `message`, relative to
/// the scope this generator's own output runs in (the package's module:
/// the generated `.mcp.rs` is `include!`d directly into prost's own
/// `<package>.rs`, the same scope every top-level message in the package
/// lives in — see `generator.rs`'s module doc comment). A top-level
/// message's path is just its own `UpperCamelCase` name (e.g.
/// `SetVibeRequest`); a message nested inside another one picks up a
/// `snake_case` module segment per enclosing message (e.g.
/// `vibe_scalar::VibeEnum`'s sibling `VibeScalar::SomeNested` would be
/// `vibe_scalar::SomeNested`), matching prost-build's own nested-type
/// module naming (confirmed against `examples.v1.rs`'s committed
/// `vibe_scalar` module) and [`heck::ToUpperCamelCase`]/[`heck::ToSnakeCase`]
/// for the case conversions themselves (the same crate `prost-build` 0.14's
/// `ident::to_upper_camel`/`to_snake` are built on). RPC request/response
/// messages are never map entries (protoc rejects a map field as an RPC's
/// type), so that prost-build special case does not apply here.
fn rust_message_path(message: &MessageDescriptor) -> String {
    let mut segments = Vec::new();
    segments.push(message.name().to_upper_camel_case());
    let mut parent = message.parent_message();
    while let Some(m) = parent {
        segments.push(m.name().to_snake_case());
        parent = m.parent_message();
    }
    segments.reverse();
    segments.join("::")
}

#[cfg(test)]
mod tests {
    use super::*;
    use prost_reflect::DescriptorPool;
    use prost_types::field_descriptor_proto::{Label, Type};
    use prost_types::{DescriptorProto, FieldDescriptorProto, FileDescriptorProto};

    /// A `FileDescriptorProto` whose top-level `Outer` message has a
    /// nested `Outer.Inner` message, for [`rust_message_path`]'s nested-
    /// message case (not exercised by `example.proto` or
    /// `tests/testdata/mcpgen/fixture.proto`, whose RPCs only use
    /// top-level messages).
    fn pool_with_nested_message() -> DescriptorPool {
        let field = |name: &str, number: i32| FieldDescriptorProto {
            name: Some(name.to_string()),
            number: Some(number),
            label: Some(Label::Optional as i32),
            r#type: Some(Type::String as i32),
            ..Default::default()
        };
        let inner = DescriptorProto {
            name: Some("Inner".to_string()),
            field: vec![field("value", 1)],
            ..Default::default()
        };
        let outer = DescriptorProto {
            name: Some("Outer".to_string()),
            field: vec![field("value", 1)],
            nested_type: vec![inner],
            ..Default::default()
        };
        let file = FileDescriptorProto {
            name: Some("nested.proto".to_string()),
            package: Some("nestedpb".to_string()),
            message_type: vec![outer],
            syntax: Some("proto3".to_string()),
            ..Default::default()
        };
        DescriptorPool::from_file_descriptor_set(prost_types::FileDescriptorSet {
            file: vec![file],
        })
        .expect("building descriptor pool")
    }

    #[test]
    fn rust_message_path_for_top_level_message() {
        let pool = pool_with_nested_message();
        let outer = pool
            .get_message_by_name("nestedpb.Outer")
            .expect("Outer message");
        assert_eq!(rust_message_path(&outer), "Outer");
    }

    #[test]
    fn rust_message_path_for_nested_message() {
        let pool = pool_with_nested_message();
        let inner = pool
            .get_message_by_name("nestedpb.Outer.Inner")
            .expect("Outer.Inner message");
        assert_eq!(rust_message_path(&inner), "outer::Inner");
    }
}
