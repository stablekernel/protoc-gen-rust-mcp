//! Generates, for one gRPC service, the MCP server struct and its tool
//! definitions and registration API: the counterpart of Go's
//! `generateMcpServerStruct`, `generateMcpServerService`, `generateMCPTool`,
//! `generateToolRegistration` and `generateDefaultToolsRegistration`
//! (`cmd/protoc-gen-go-mcp/mcp.go` at commit `9e072f9`). Handler *bodies*
//! are #7; `method_handler_stub` below only emits a "not implemented" tool
//! error, matching this issue's scope. `method_public_handler_fn` emits
//! the public `pub async fn <method>_handler(&self, args)` wrapper Go also
//! exports, so a caller can delegate to the default behavior from an
//! overriding tool.
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

use heck::{ToShoutySnakeCase, ToSnakeCase};
use prost_reflect::{MethodDescriptor, ServiceDescriptor};
use serde_json::Value;

use crate::comments::{camel_to_space, process_comment_to_string};
use crate::rust_literal::rust_string_literal;
use crate::schema::message_input_schema;
use crate::source_info::leading_comments;

/// Renders the full generated module for one service: the input schema
/// constants, the tool handler type alias, the server struct, its `impl`
/// block (constructor, one `<method>_tool()` + stub handler + public
/// `<method>_handler()` wrapper per unary RPC, `register_tool`,
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
         \u{20}   T: ::std::clone::Clone + ::std::marker::Send + ::std::marker::Sync + 'static,\n\
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
        out.push_str(&method_handler_stub(&client_path, method));
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
        "    /// Registers every unary RPC's tool with its stub handler, the\n\
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
         \u{20}   T: ::std::clone::Clone + ::std::marker::Send + ::std::marker::Sync + 'static,\n\
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

/// The stub handler for one method: a private async associated function
/// that takes an owned client and always returns a "not implemented" tool
/// error. A later generator issue (#7) replaces this with argument
/// validation, pbjson decoding, the tonic call, and a pbjson-encoded
/// response. Named `call_<method>` (not `<method>_handler`, which is the
/// public wrapper in [`method_public_handler_fn`]) because
/// `register_default_tools` builds its closures from this one directly,
/// cloning `self.client` once per registration rather than per call.
fn method_handler_stub(client_path: &str, method: &MethodDescriptor) -> String {
    let method_name = method.name();
    let fn_name = method_fn_name(method);
    format!(
        "    /// Stub handler for `{method_name}`: always returns a \"not\n\
         \u{20}   /// implemented\" tool error. A later generator issue (#7) replaces\n\
         \u{20}   /// this with argument validation, pbjson decoding, the `{method_name}`\n\
         \u{20}   /// tonic call, and a pbjson-encoded response.\n\
         \u{20}   async fn call_{fn_name}(\n\
         \u{20}       _client: {client_path}<T>,\n\
         \u{20}       _args: ::std::option::Option<::rmcp::model::JsonObject>,\n\
         \u{20}   ) -> ::rmcp::model::CallToolResult {{\n\
         \u{20}       ::rmcp::model::CallToolResult::error(vec![::rmcp::model::ContentBlock::text(\n\
         \u{20}           \"{method_name} is not implemented\",\n\
         \u{20}       )])\n\
         \u{20}   }}\n"
    )
}

/// The public `pub async fn <method>_handler(&self, args) -> CallToolResult`
/// associated function for one method: exported (like Go's `XxxHandler`
/// methods) so a caller can wrap a default handler, e.g. register an
/// overriding tool that pre-processes arguments and then delegates to this
/// one, which `register_default_tools` itself does not need (it calls the
/// private [`method_handler_stub`] directly with a clone of `self.client`
/// made once per registration, not once per call).
fn method_public_handler_fn(method: &MethodDescriptor) -> String {
    let method_name = method.name();
    let fn_name = method_fn_name(method);
    format!(
        "    /// Calls the stub handler for `{method_name}` with a clone of this\n\
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
