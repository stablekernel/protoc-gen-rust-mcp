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
    include!(concat!(env!("OUT_DIR"), "/mcpgen.mcp.rs"));
}

use mcpgen::other_fixture_service_client::OtherFixtureServiceClient;
use mcpgen::vibe_fixture_service_client::VibeFixtureServiceClient;
use rmcp::model::{CallToolRequestParams, JsonObject};
use rmcp::{ServerHandler, ServiceExt};
use tonic::transport::Endpoint;

/// A `VibeFixtureServiceClient<Channel>`/`OtherFixtureServiceClient<Channel>`
/// that never actually connects (`connect_lazy`): every tool call in this
/// test is either a stub (handler bodies are #7) or an override that
/// never touches the client, so a real gRPC server is unnecessary.
fn lazy_vibe_client() -> VibeFixtureServiceClient<tonic::transport::Channel> {
    let channel = Endpoint::from_static("http://127.0.0.1:1").connect_lazy();
    VibeFixtureServiceClient::new(channel)
}

fn lazy_other_client() -> OtherFixtureServiceClient<tonic::transport::Channel> {
    let channel = Endpoint::from_static("http://127.0.0.1:1").connect_lazy();
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
    let mut server = mcpgen::VibeFixtureServiceMcpServer::new(lazy_vibe_client());
    server.register_default_tools();

    // Cloned before the override is registered, so the clone's own `tools`
    // map is irrelevant: `get_vibe_handler` only reads `self.client`, not
    // `self.tools`, so this purely exercises the public wrapper delegating
    // to the same stub behavior as the default registration.
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
    // Same "not implemented" stub behavior as calling the default handler
    // directly, proving the override actually delegated rather than
    // short-circuiting.
    assert_eq!(result.is_error, Some(true));
    let rmcp::model::ContentBlock::Text(text) = &result.content[0] else {
        panic!("expected text content");
    };
    assert_eq!(text.text, "GetVibe is not implemented");
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
    let mut vibe_server = mcpgen::VibeFixtureServiceMcpServer::new(lazy_vibe_client());
    vibe_server.register_default_tools();

    let mut other_server = mcpgen::OtherFixtureServiceMcpServer::new(lazy_other_client());
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
    // server, not just listed (its stub handler still returns the "not
    // implemented" tool error, since handler bodies are #7).
    let result = client
        .call_tool(CallToolRequestParams::new("DoOther"))
        .await
        .expect("call_tool");
    assert_eq!(result.is_error, Some(true));
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

/// Every stub handler returns an `isError: true` "not implemented" tool
/// error (handler bodies are #7), named after its RPC.
#[tokio::test]
async fn stub_handler_returns_not_implemented_tool_error() {
    let mut server = mcpgen::VibeFixtureServiceMcpServer::new(lazy_vibe_client());
    server.register_default_tools();
    let client = serve(server).await;

    let result = client
        .call_tool(CallToolRequestParams::new("GetVibe"))
        .await
        .expect("call_tool");
    assert_eq!(result.is_error, Some(true));
    let rmcp::model::ContentBlock::Text(text) = &result.content[0] else {
        panic!("expected text content");
    };
    assert_eq!(text.text, "GetVibe is not implemented");
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
