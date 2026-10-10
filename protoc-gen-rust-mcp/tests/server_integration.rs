//! Integration test for `server::generate_service`'s output (issue #6's
//! acceptance criteria), exercised end-to-end over an in-memory MCP
//! transport against the real tonic-generated client code for
//! `tests/testdata/mcpgen/fixture.proto` (compiled by `build.rs` with real
//! `tonic-prost-build`, and this crate's own generator run against the
//! same descriptors; see `build.rs`'s `generate_mcpgen_fixture` for how
//! `mcpgen.rs` and `mcpgen.mcp.rs` land in `OUT_DIR`).
//!
//! Unlike `examples/`, this fixture's generated code is not committed:
//! `fixture.proto` exists only to give this test a service with a
//! streaming RPC (not in `example.proto`) and a second service (for the
//! cross-service composition test), without changing the shared parity
//! fixture.

#[allow(
    dead_code,
    missing_docs,
    clippy::all,
    clippy::pedantic,
    unreachable_pub
)]
mod mcpgen {
    include!(concat!(env!("OUT_DIR"), "/mcpgen.rs"));
    include!(concat!(env!("OUT_DIR"), "/mcpgen.serde.rs"));
    include!(concat!(env!("OUT_DIR"), "/mcpgen.mcp.rs"));
}

use mcpgen::other_fixture_service_client::OtherFixtureServiceClient;
use mcpgen::other_fixture_service_server::{OtherFixtureService, OtherFixtureServiceServer};
use mcpgen::vibe_fixture_service_client::VibeFixtureServiceClient;
use mcpgen::vibe_fixture_service_server::{VibeFixtureService, VibeFixtureServiceServer};
use rmcp::model::{CallToolRequestParams, JsonObject};
use rmcp::{ServerHandler, ServiceExt};
use tonic::transport::Endpoint;
use tonic::transport::server::TcpIncoming;

/// A `VibeFixtureServiceClient<Channel>` that never actually connects
/// (`connect_lazy`): for tests that only inspect registered tool
/// metadata, call an overriding tool that never touches the client, or
/// must prove the backend is never called (schema/decode failures),
/// where a real connection would mask a bug by letting an
/// unexpectedly-forwarded call fail for an unrelated reason.
fn lazy_vibe_client() -> VibeFixtureServiceClient<tonic::transport::Channel> {
    let channel = Endpoint::from_static("http://127.0.0.1:1").connect_lazy();
    VibeFixtureServiceClient::new(channel)
}

/// A `VibeFixtureService` backend for tests that exercise the real
/// handler body (this issue, #7) end to end: `SetVibe` with
/// `vibe == "missing"` returns a gRPC `NotFound`, so the error-formatting
/// path (`rpc error: code = NotFound desc = ...`) gets exercised against
/// a real `tonic::Status`, not a constructed one; every other RPC echoes
/// its input back.
struct FakeVibeBackend;

#[tonic::async_trait]
impl VibeFixtureService for FakeVibeBackend {
    async fn set_vibe(
        &self,
        request: tonic::Request<mcpgen::SetVibeRequest>,
    ) -> Result<tonic::Response<mcpgen::SetVibeResponse>, tonic::Status> {
        let vibe = request.into_inner().vibe;
        if vibe == "missing" {
            return Err(tonic::Status::not_found("vibe not found"));
        }
        Ok(tonic::Response::new(mcpgen::SetVibeResponse { vibe }))
    }

    async fn get_vibe(
        &self,
        _request: tonic::Request<mcpgen::GetVibeRequest>,
    ) -> Result<tonic::Response<mcpgen::GetVibeResponse>, tonic::Status> {
        Ok(tonic::Response::new(mcpgen::GetVibeResponse {
            vibe: "chill".to_string(),
        }))
    }

    type StreamVibeStream = tonic::codegen::tokio_stream::Once<
        std::result::Result<mcpgen::StreamVibeResponse, tonic::Status>,
    >;

    async fn stream_vibe(
        &self,
        _request: tonic::Request<mcpgen::StreamVibeRequest>,
    ) -> Result<tonic::Response<Self::StreamVibeStream>, tonic::Status> {
        Err(tonic::Status::unimplemented(
            "not used by this test; VibeFixtureService.StreamVibe produces no tool",
        ))
    }
}

/// An `OtherFixtureService` backend: `DoOther` echoes its input, used by
/// the cross-service composition test.
struct FakeOtherBackend;

#[tonic::async_trait]
impl OtherFixtureService for FakeOtherBackend {
    async fn do_other(
        &self,
        request: tonic::Request<mcpgen::OtherRequest>,
    ) -> Result<tonic::Response<mcpgen::OtherResponse>, tonic::Status> {
        Ok(tonic::Response::new(mcpgen::OtherResponse {
            value: request.into_inner().value,
        }))
    }

    async fn get_vibe(
        &self,
        _request: tonic::Request<mcpgen::OtherRequest>,
    ) -> Result<tonic::Response<mcpgen::OtherResponse>, tonic::Status> {
        Err(tonic::Status::unimplemented("not used by this test"))
    }
}

/// Starts a real in-process tonic server for `VibeFixtureService` (backed
/// by [`FakeVibeBackend`]) on an OS-assigned ephemeral TCP port, and
/// returns a `VibeFixtureServiceClient` connected to it: so a test can
/// exercise the generated handler body's tonic call and response/error
/// handling against a real backend, not a never-connecting client.
async fn serve_vibe_backend() -> VibeFixtureServiceClient<tonic::transport::Channel> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind ephemeral port");
    let addr = listener.local_addr().expect("local_addr");
    let incoming = TcpIncoming::from(listener);
    tokio::spawn(
        tonic::transport::Server::builder()
            .add_service(VibeFixtureServiceServer::new(FakeVibeBackend))
            .serve_with_incoming(incoming),
    );
    // Not `VibeFixtureServiceClient::connect`: `tonic-prost-build`'s
    // generated `connect()` needs its own "transport" feature (distinct
    // from `tonic`'s own), which this crate's dev-dependency on
    // `tonic-prost-build` doesn't enable; building the `Channel` directly
    // avoids a dependency this test doesn't otherwise need.
    let channel = Endpoint::from_shared(format!("http://{addr}"))
        .expect("valid endpoint")
        .connect()
        .await
        .expect("connect to in-process server");
    VibeFixtureServiceClient::new(channel)
}

/// Starts a real in-process tonic server for `OtherFixtureService`
/// (backed by [`FakeOtherBackend`]), the counterpart of
/// [`serve_vibe_backend`] for the cross-service composition test.
async fn serve_other_backend() -> OtherFixtureServiceClient<tonic::transport::Channel> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind ephemeral port");
    let addr = listener.local_addr().expect("local_addr");
    let incoming = TcpIncoming::from(listener);
    tokio::spawn(
        tonic::transport::Server::builder()
            .add_service(OtherFixtureServiceServer::new(FakeOtherBackend))
            .serve_with_incoming(incoming),
    );
    let channel = Endpoint::from_shared(format!("http://{addr}"))
        .expect("valid endpoint")
        .connect()
        .await
        .expect("connect to in-process server");
    OtherFixtureServiceClient::new(channel)
}

/// Serves `handler` over an in-memory duplex transport and returns a
/// connected rmcp client, so the test calls `tools/list` and `tools/call`
/// the same way a real MCP client would, rather than calling
/// `ServerHandler` methods directly (which would need a `Peer`/
/// `RequestContext` this crate has no public way to construct).
async fn serve<H>(handler: H) -> rmcp::service::RunningService<rmcp::RoleClient, ()>
where
    H: ServerHandler + Send + Sync + 'static,
{
    let (server_io, client_io) = tokio::io::duplex(64 * 1024);
    tokio::spawn(async move {
        let running = handler.serve(server_io).await.expect("serve");
        let _ = running.waiting().await;
    });
    ().serve(client_io).await.expect("client connect")
}

/// `register_default_tools()` registers one tool per unary RPC (`SetVibe`,
/// `GetVibe`), by its proto method name, skipping the server-streaming
/// `StreamVibe` RPC: the issue's "a streaming RPC in the test data
/// produces no tool" criterion.
#[tokio::test]
async fn register_default_tools_registers_only_unary_methods() {
    let mut server = mcpgen::VibeFixtureServiceMcpServer::new(lazy_vibe_client());
    server.register_default_tools();
    assert!(
        server.get_tool("StreamVibe").is_none(),
        "a server-streaming RPC must not produce a tool"
    );

    let client = serve(server).await;
    let tools = client.list_all_tools().await.expect("list_all_tools");
    let mut names: Vec<String> = tools.iter().map(|t| t.name.to_string()).collect();
    names.sort_unstable();
    assert_eq!(names, vec!["GetVibe".to_string(), "SetVibe".to_string()]);
}

/// Calling a tool that isn't registered returns rmcp's normal "tool not
/// found" error, the issue's explicit requirement.
#[tokio::test]
async fn unregistered_tool_is_not_found() {
    let server = mcpgen::VibeFixtureServiceMcpServer::new(lazy_vibe_client());
    let client = serve(server).await;

    let err = client
        .call_tool(CallToolRequestParams::new("Nope"))
        .await
        .expect_err("unregistered tool must error");
    assert!(
        err.to_string().contains("tool not found"),
        "unexpected error: {err}"
    );
}

/// `register_tool` overrides a tool registered by
/// `register_default_tools`, keeping the same name but swapping in a
/// caller-provided handler: the issue's "callers can override... a single
/// tool via register_tool" criterion.
#[tokio::test]
async fn register_tool_overrides_a_default_tool() {
    let mut server = mcpgen::VibeFixtureServiceMcpServer::new(lazy_vibe_client());
    server.register_default_tools();
    server.register_tool(
        mcpgen::VibeFixtureServiceMcpServer::<tonic::transport::Channel>::set_vibe_tool(),
        |_args: Option<JsonObject>| async {
            rmcp::model::CallToolResult::success(vec![rmcp::model::ContentBlock::text(
                "overridden",
            )])
        },
    );

    let client = serve(server).await;

    // Still exactly one tool named "SetVibe"; overriding doesn't add a
    // second entry.
    let tools = client.list_all_tools().await.expect("list_all_tools");
    assert_eq!(tools.iter().filter(|t| t.name == "SetVibe").count(), 1);

    let result = client
        .call_tool(CallToolRequestParams::new("SetVibe"))
        .await
        .expect("call_tool");
    assert_eq!(result.is_error, Some(false));
    let rmcp::model::ContentBlock::Text(text) = &result.content[0] else {
        panic!("expected text content");
    };
    assert_eq!(text.text, "overridden");
}

/// The public `<method>_handler` wrapper (`get_vibe_handler`) lets an
/// overriding tool delegate to the default behavior after pre-processing
/// arguments, the way Go's exported `XxxHandler` methods do (review
/// feedback on #21: "Expose the handlers publicly, as Go does").
#[tokio::test]
async fn public_handler_wrapper_can_be_delegated_to_from_an_override() {
    let mut server = mcpgen::VibeFixtureServiceMcpServer::new(serve_vibe_backend().await);
    server.register_default_tools();

    // Cloned before the override is registered, so the clone's own `tools`
    // map is irrelevant: `get_vibe_handler` only reads `self.client`, not
    // `self.tools`, so this purely exercises the public wrapper delegating
    // to the same default behavior as the default registration.
    let delegate = server.clone();
    server.register_tool(
        mcpgen::VibeFixtureServiceMcpServer::<tonic::transport::Channel>::get_vibe_tool(),
        move |args| {
            let delegate = delegate.clone();
            async move { delegate.get_vibe_handler(args).await }
        },
    );

    let client = serve(server).await;
    let result = client
        .call_tool(CallToolRequestParams::new("GetVibe"))
        .await
        .expect("call_tool");
    // Same real-backend behavior as calling the default handler directly
    // (FakeVibeBackend::get_vibe always returns "chill"), proving the
    // override actually delegated rather than short-circuiting.
    assert_eq!(result.is_error, Some(false));
    let rmcp::model::ContentBlock::Text(text) = &result.content[0] else {
        panic!("expected text content");
    };
    assert_eq!(text.text, r#"{"vibe":"chill"}"#);
}

/// `into_tools`/`extend_tools` let one MCP server serve tools from
/// several services: the issue's "several services' tools can be served
/// from one MCP server" criterion.
///
/// `OtherFixtureService` also declares a `GetVibe` RPC (same name as
/// `VibeFixtureService.GetVibe`, on purpose — review feedback on #21):
/// this test compiling and passing at all proves the two services' input
/// schema constants don't collide in the generated `mcpgen.mcp.rs` (they
/// used to be named only after the method, e.g. two
/// `static GET_VIBE_INPUT_SCHEMA` items, which failed to compile).
#[tokio::test]
async fn one_server_can_serve_tools_from_two_services() {
    let mut vibe_server = mcpgen::VibeFixtureServiceMcpServer::new(serve_vibe_backend().await);
    vibe_server.register_default_tools();

    let mut other_server = mcpgen::OtherFixtureServiceMcpServer::new(serve_other_backend().await);
    other_server.register_default_tools();

    vibe_server.extend_tools(other_server.into_tools());

    let client = serve(vibe_server).await;
    let tools = client.list_all_tools().await.expect("list_all_tools");
    let mut names: Vec<String> = tools.iter().map(|t| t.name.to_string()).collect();
    names.sort_unstable();
    // "GetVibe" appears once (last write wins on the shared tool name, as
    // `register_tool`'s doc promises), even though both services declare
    // it.
    assert_eq!(
        names,
        vec![
            "DoOther".to_string(),
            "GetVibe".to_string(),
            "SetVibe".to_string(),
        ]
    );

    // The composed-in tool is actually callable through the combined
    // server, not just listed: it reaches OtherFixtureService's own
    // backend (FakeOtherBackend::do_other echoes its input), not
    // VibeFixtureService's.
    let mut args = JsonObject::new();
    args.insert("value".to_string(), serde_json::json!("hi"));
    let result = client
        .call_tool(CallToolRequestParams::new("DoOther").with_arguments(args))
        .await
        .expect("call_tool");
    assert_eq!(result.is_error, Some(false));
    let rmcp::model::ContentBlock::Text(text) = &result.content[0] else {
        panic!("expected text content");
    };
    assert_eq!(text.text, r#"{"value":"hi"}"#);
}

/// Each service's own `GetVibe` input schema reflects its own RPC's
/// request message (`GetVibeRequest` has no fields;
/// `OtherRequest` has one), confirming the two same-named tools' schema
/// constants are genuinely distinct package-level items, not one
/// accidentally shared between the two services.
#[test]
fn same_named_tools_across_services_have_independent_schemas() {
    let vibe_schema =
        mcpgen::VibeFixtureServiceMcpServer::<tonic::transport::Channel>::get_vibe_tool()
            .schema_as_json_value();
    let other_schema =
        mcpgen::OtherFixtureServiceMcpServer::<tonic::transport::Channel>::get_vibe_tool()
            .schema_as_json_value();
    assert_ne!(
        vibe_schema, other_schema,
        "VibeFixtureService.GetVibe and OtherFixtureService.GetVibe have different \
         request messages and must not share a schema"
    );
    assert_eq!(vibe_schema["properties"], serde_json::json!({}));
    assert_eq!(
        other_schema["properties"]["value"]["type"],
        serde_json::json!("string")
    );
}

/// A successful tool call: the handler validates, decodes, calls the
/// backend, and returns the pbjson-encoded response as `isError: false`
/// text content (this issue, #7).
#[tokio::test]
async fn handler_success_returns_pbjson_encoded_response() {
    let mut server = mcpgen::VibeFixtureServiceMcpServer::new(serve_vibe_backend().await);
    server.register_default_tools();
    let client = serve(server).await;

    let mut args = JsonObject::new();
    args.insert("vibe".to_string(), serde_json::json!("chill"));
    let result = client
        .call_tool(CallToolRequestParams::new("SetVibe").with_arguments(args))
        .await
        .expect("call_tool");
    assert_eq!(result.is_error, Some(false));
    let rmcp::model::ContentBlock::Text(text) = &result.content[0] else {
        panic!("expected text content");
    };
    // pbjson omits the zero-valued "vibe" field... here it's non-zero, so
    // it's present; this also pins the plain camelCase field name.
    assert_eq!(text.text, r#"{"vibe":"chill"}"#);
}

/// A `None` arguments value is treated as `{}` (an empty object), per the
/// issue's requirement, not rejected or passed through as JSON `null`.
#[tokio::test]
async fn handler_treats_missing_arguments_as_empty_object() {
    let mut server = mcpgen::VibeFixtureServiceMcpServer::new(serve_vibe_backend().await);
    server.register_default_tools();
    let client = serve(server).await;

    // GetVibeRequest has no fields, so "{}" always validates; SetVibe's
    // schema also has no required properties (an empty object validates),
    // so omitting arguments there pins the same behavior for a message
    // that does have fields.
    let result = client
        .call_tool(CallToolRequestParams::new("SetVibe"))
        .await
        .expect("call_tool");
    assert_eq!(result.is_error, Some(false));
    let rmcp::model::ContentBlock::Text(text) = &result.content[0] else {
        panic!("expected text content");
    };
    // SetVibeRequest.vibe defaults to "", echoed back as "" by
    // FakeVibeBackend; pbjson omits a zero-valued string field entirely.
    assert_eq!(text.text, "{}");
}

/// A gRPC error becomes a tool error formatted like grpc-go's
/// `Status.Error()`: `"rpc error: code = NotFound desc = ..."` (this
/// issue, #7's explicit acceptance criterion). The backend is actually
/// called here (unlike the validation/decode failure tests below).
#[tokio::test]
async fn handler_grpc_error_becomes_tool_error_in_grpc_go_format() {
    let mut server = mcpgen::VibeFixtureServiceMcpServer::new(serve_vibe_backend().await);
    server.register_default_tools();
    let client = serve(server).await;

    let mut args = JsonObject::new();
    args.insert("vibe".to_string(), serde_json::json!("missing"));
    let result = client
        .call_tool(CallToolRequestParams::new("SetVibe").with_arguments(args))
        .await
        .expect("call_tool");
    assert_eq!(result.is_error, Some(true));
    let rmcp::model::ContentBlock::Text(text) = &result.content[0] else {
        panic!("expected text content");
    };
    assert_eq!(
        text.text,
        "rpc error: code = NotFound desc = vibe not found"
    );
}

/// A wrong argument type (a number where the schema wants a string) fails
/// schema validation before decoding, and the backend is never called:
/// this issue's explicit "wrong argument type → isError, backend not
/// called" acceptance criterion. Proven by using the `lazy_*` client
/// (never connects): if the backend were called, the test would hang or
/// error on the broken connection instead of returning this message.
#[tokio::test]
async fn handler_wrong_argument_type_is_rejected_without_calling_backend() {
    let mut server = mcpgen::VibeFixtureServiceMcpServer::new(lazy_vibe_client());
    server.register_default_tools();
    let client = serve(server).await;

    let mut args = JsonObject::new();
    args.insert("vibe".to_string(), serde_json::json!(5));
    let result = client
        .call_tool(CallToolRequestParams::new("SetVibe").with_arguments(args))
        .await
        .expect("call_tool");
    assert_eq!(result.is_error, Some(true));
    let rmcp::model::ContentBlock::Text(text) = &result.content[0] else {
        panic!("expected text content");
    };
    assert_eq!(
        text.text,
        r#"invalid arguments: /vibe: 5 is not of type "string""#
    );
}

/// An unknown field (the schema's `additionalProperties: false`) fails
/// schema validation before decoding, and the backend is never called:
/// this issue's explicit "unknown field → isError, backend not called"
/// acceptance criterion, including the snake_case form of a field that
/// only exists on the *response* message (`SetVibeResponse.vibe`, not
/// `SetVibeRequest`), so this can't accidentally pass by decoding into
/// the wrong message.
#[tokio::test]
async fn handler_unknown_field_is_rejected_without_calling_backend() {
    let mut server = mcpgen::VibeFixtureServiceMcpServer::new(lazy_vibe_client());
    server.register_default_tools();
    let client = serve(server).await;

    let mut args = JsonObject::new();
    args.insert("not_a_field".to_string(), serde_json::json!("x"));
    let result = client
        .call_tool(CallToolRequestParams::new("SetVibe").with_arguments(args))
        .await
        .expect("call_tool");
    assert_eq!(result.is_error, Some(true));
    let rmcp::model::ContentBlock::Text(text) = &result.content[0] else {
        panic!("expected text content");
    };
    assert_eq!(
        text.text,
        r#"invalid arguments: : Additional properties are not allowed ('not_a_field' was unexpected)"#
    );
}

/// A freshly constructed server has no tools registered yet (the issue's
/// "no tools registered yet, like Go").
#[tokio::test]
async fn new_server_has_no_tools() {
    let server = mcpgen::VibeFixtureServiceMcpServer::new(lazy_vibe_client());
    let client = serve(server).await;
    let tools = client.list_all_tools().await.expect("list_all_tools");
    assert!(tools.is_empty());
}
