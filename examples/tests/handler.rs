//! End-to-end tests for the generated handler bodies (issue #7's
//! acceptance criteria), against the real example `VibeService` server
//! and a real in-process tonic backend (`FakeVibeService` below), not a
//! never-connecting client: every assertion here exercises the actual
//! validate -> decode -> call -> encode path `protoc-gen-rust-mcp`
//! generates into `examples.v1.mcp.rs`.

use examples::v1::vibe_service_client::VibeServiceClient;
use examples::v1::vibe_service_server::{VibeService, VibeServiceServer};
use examples::v1::*;
use rmcp::model::{CallToolRequestParams, JsonObject};
use rmcp::{ServerHandler, ServiceExt};
use tonic::transport::server::TcpIncoming;

/// A `VibeService` backend: `SetVibe` with `vibe == "missing"` returns a
/// gRPC `NotFound`, exercising the handler's error-formatting path
/// against a real `tonic::Status`; every other RPC echoes enough of its
/// input to prove it actually reached the backend.
struct FakeVibeService;

#[tonic::async_trait]
impl VibeService for FakeVibeService {
    async fn set_vibe(
        &self,
        request: tonic::Request<SetVibeRequest>,
    ) -> Result<tonic::Response<SetVibeResponse>, tonic::Status> {
        let vibe = request.into_inner().vibe;
        if vibe == "missing" {
            return Err(tonic::Status::not_found("vibe not found"));
        }
        Ok(tonic::Response::new(SetVibeResponse {
            previous_vibe: String::new(),
            vibe,
        }))
    }

    async fn get_vibe(
        &self,
        _request: tonic::Request<GetVibeRequest>,
    ) -> Result<tonic::Response<GetVibeResponse>, tonic::Status> {
        Ok(tonic::Response::new(GetVibeResponse {
            vibe: "chill".to_string(),
        }))
    }

    async fn set_vibe_details(
        &self,
        request: tonic::Request<SetVibeDetailsRequest>,
    ) -> Result<tonic::Response<SetVibeResponse>, tonic::Status> {
        let req = request.into_inner();
        Ok(tonic::Response::new(SetVibeResponse {
            previous_vibe: String::new(),
            vibe: req.vibe,
        }))
    }

    async fn set_vibe_array(
        &self,
        _request: tonic::Request<SetVibeArrayRequest>,
    ) -> Result<tonic::Response<SetVibeArrayResponse>, tonic::Status> {
        Err(tonic::Status::unimplemented("not used by this test"))
    }

    async fn set_vibe_objects(
        &self,
        _request: tonic::Request<SetVibeObjectsRequest>,
    ) -> Result<tonic::Response<SetVibeObjectsResponse>, tonic::Status> {
        Err(tonic::Status::unimplemented("not used by this test"))
    }
}

/// Starts a real in-process tonic server for `VibeService` on an
/// OS-assigned ephemeral TCP port, and returns a connected
/// `VibeServiceClient`.
async fn serve_backend() -> VibeServiceClient<tonic::transport::Channel> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind ephemeral port");
    let addr = listener.local_addr().expect("local_addr");
    let incoming = TcpIncoming::from(listener);
    tokio::spawn(
        tonic::transport::Server::builder()
            .add_service(VibeServiceServer::new(FakeVibeService))
            .serve_with_incoming(incoming),
    );
    VibeServiceClient::connect(format!("http://{addr}"))
        .await
        .expect("connect to in-process server")
}

/// A `VibeServiceClient<Channel>` that never actually connects: for
/// tests that must prove the backend is never called (schema/decode
/// failures), where a real connection would mask a bug by letting an
/// unexpectedly-forwarded call fail for an unrelated reason (and where a
/// real call would just hang/error against nothing listening).
fn lazy_client() -> VibeServiceClient<tonic::transport::Channel> {
    let channel = tonic::transport::Endpoint::from_static("http://127.0.0.1:1").connect_lazy();
    VibeServiceClient::new(channel)
}

/// Serves `handler` over an in-memory duplex transport and returns a
/// connected rmcp client (same approach as
/// `protoc-gen-rust-mcp/tests/server_integration.rs`).
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

fn text_of(result: &rmcp::model::CallToolResult) -> &str {
    let rmcp::model::ContentBlock::Text(text) = &result.content[0] else {
        panic!("expected text content");
    };
    &text.text
}

/// Success: the result JSON matches protojson conventions (lowerCamelCase
/// field names, zero-valued fields omitted).
#[tokio::test]
async fn success_result_matches_protojson_conventions() {
    let mut server = VibeServiceMcpServer::new(serve_backend().await);
    server.register_default_tools();
    let client = serve(server).await;

    let mut args = JsonObject::new();
    args.insert("vibe".to_string(), serde_json::json!("chill"));
    let result = client
        .call_tool(CallToolRequestParams::new("SetVibe").with_arguments(args))
        .await
        .expect("call_tool");
    assert_eq!(result.is_error, Some(false));
    // previousVibe is zero-valued ("") and omitted; vibe is present.
    assert_eq!(text_of(&result), r#"{"vibe":"chill"}"#);
}

/// A gRPC `NotFound` becomes `isError: true` with grpc-go's
/// `"rpc error: code = NotFound desc = ..."` text.
#[tokio::test]
async fn grpc_not_found_becomes_tool_error_in_grpc_go_format() {
    let mut server = VibeServiceMcpServer::new(serve_backend().await);
    server.register_default_tools();
    let client = serve(server).await;

    let mut args = JsonObject::new();
    args.insert("vibe".to_string(), serde_json::json!("missing"));
    let result = client
        .call_tool(CallToolRequestParams::new("SetVibe").with_arguments(args))
        .await
        .expect("call_tool");
    assert_eq!(result.is_error, Some(true));
    assert_eq!(
        text_of(&result),
        "rpc error: code = NotFound desc = vibe not found"
    );
}

/// A wrong argument type fails schema validation before decoding; the
/// backend is never called (proven by using `lazy_client`, which would
/// hang/error on a real call attempt rather than returning this specific
/// validation message).
#[tokio::test]
async fn wrong_argument_type_is_rejected_without_calling_backend() {
    let mut server = VibeServiceMcpServer::new(lazy_client());
    server.register_default_tools();
    let client = serve(server).await;

    let mut args = JsonObject::new();
    args.insert("vibe".to_string(), serde_json::json!(5));
    let result = client
        .call_tool(CallToolRequestParams::new("SetVibe").with_arguments(args))
        .await
        .expect("call_tool");
    assert_eq!(result.is_error, Some(true));
    assert_eq!(
        text_of(&result),
        r#"invalid arguments: /vibe: 5 is not of type "string""#
    );
}

/// An unknown field is rejected by `additionalProperties: false` before
/// decoding, including the snake_case form of a field that exists only
/// on the *response* message (`SetVibeResponse.previous_vibe`), proving
/// this isn't accidentally accepted via the wrong message's schema.
#[tokio::test]
async fn unknown_field_including_snake_case_is_rejected_without_calling_backend() {
    let mut server = VibeServiceMcpServer::new(lazy_client());
    server.register_default_tools();
    let client = serve(server).await;

    let mut args = JsonObject::new();
    args.insert("previous_vibe".to_string(), serde_json::json!("x"));
    let result = client
        .call_tool(CallToolRequestParams::new("SetVibe").with_arguments(args))
        .await
        .expect("call_tool");
    assert_eq!(result.is_error, Some(true));
    assert_eq!(
        text_of(&result),
        r#"invalid arguments: : Additional properties are not allowed ('previous_vibe' was unexpected)"#
    );
}

/// An int64 field is accepted both as a JSON number and as a decimal
/// string (pbjson's generated `Deserialize` accepts both for a 64-bit
/// field; the schema's `type: ["integer", "string"]` reflects that), and
/// an enum is accepted by its name. `SetVibeDetails` echoes the vibe
/// string back through `SetVibeResponse`, so a successful, non-error
/// result proves the whole message (including vibeScalar) decoded.
#[tokio::test]
async fn int64_accepted_as_number_or_string_and_enum_by_name() {
    let mut server = VibeServiceMcpServer::new(serve_backend().await);
    server.register_default_tools();
    let client = serve(server).await;

    for vibe_int64 in [serde_json::json!(42), serde_json::json!("42")] {
        let mut args = JsonObject::new();
        args.insert("vibe".to_string(), serde_json::json!("details"));
        args.insert(
            "vibeScalar".to_string(),
            serde_json::json!({
                "vibeInt64": vibe_int64,
                "vibeEnum": ["VIBE_GOOD"],
            }),
        );
        let result = client
            .call_tool(CallToolRequestParams::new("SetVibeDetails").with_arguments(args))
            .await
            .expect("call_tool");
        assert_eq!(
            result.is_error,
            Some(false),
            "vibeInt64 = {vibe_int64:?} should validate and decode: {}",
            text_of(&result)
        );
        assert_eq!(text_of(&result), r#"{"vibe":"details"}"#);
    }
}

/// `NaN`/`Infinity`/`-Infinity` are accepted as float/double *input*
/// (both the schema and pbjson's generated deserializer special-case
/// these three strings; see `schema.rs`'s `float_schema`), going through
/// the real handler's validate-then-decode path rather than the
/// schema/deserializer in isolation.
#[tokio::test]
async fn non_finite_floats_are_accepted_as_input() {
    let mut server = VibeServiceMcpServer::new(serve_backend().await);
    server.register_default_tools();
    let client = serve(server).await;

    for s in ["NaN", "Infinity", "-Infinity"] {
        let mut args = JsonObject::new();
        args.insert("vibe".to_string(), serde_json::json!("details"));
        args.insert(
            "vibeScalar".to_string(),
            serde_json::json!({"vibeDouble": s, "vibeFloat": s}),
        );
        let result = client
            .call_tool(CallToolRequestParams::new("SetVibeDetails").with_arguments(args))
            .await
            .expect("call_tool");
        assert_eq!(
            result.is_error,
            Some(false),
            "{s:?} should validate and decode: {}",
            text_of(&result)
        );
    }
}
