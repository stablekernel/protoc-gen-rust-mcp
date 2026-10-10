//! End-to-end test of a generated server over real MCP and gRPC (issue
//! #9), the counterpart of Go's `examples/gen/example/v1/e2e_test.go` at
//! `stablekernel/protoc-gen-go-mcp` commit `9e072f9`: a fake `VibeService`
//! backend runs behind a real in-process gRPC connection, the generated
//! `VibeServiceMcpServer` sits in front of it, and the test talks MCP
//! (over an in-memory client/server transport pair, like
//! `examples/tests/handler.rs`) to drive `tools/list` and `tools/call`
//! the same way a real MCP client would.
//!
//! Every rmcp type is confined to this file's harness (`new_test_harness`,
//! `list_tools`, `call_tool`): the test functions below work with plain
//! `ToolInfo`/`ToolResult` values decoded from the wire response, as Go's
//! do, so swapping the MCP client implementation again in the future only
//! changes the harness.
//!
//! This file also carries the issue's **cross-implementation parity
//! fixture**: `tools/list`'s result is asserted JSON-equal to
//! `parity/go-v0.3.0-tools.json`, captured from Go's committed
//! `example_mcp.pb.go` (see `parity/README.md` for exactly how), and
//! `SetVibeDetails`'s `tools/call` result text is asserted JSON-equal to
//! the value Go's generated handler would produce for the same input
//! (`protojson.Marshal` and this crate's pbjson encoding agree on field
//! presence/naming for this message).

use examples::v1::vibe_service_client::VibeServiceClient;
use examples::v1::vibe_service_server::{VibeService, VibeServiceServer};
use examples::v1::*;
use rmcp::ServiceExt;
use rmcp::model::{CallToolRequestParams, JsonObject};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Mutex;
use tonic::transport::server::TcpIncoming;

/// A hand-written `VibeService` that records every request it receives
/// (keyed by RPC name, in receipt order) and returns canned responses or
/// a canned error, so a test can assert on exactly what the generated MCP
/// handler sent it (the counterpart of Go's `fakeVibeService`).
#[derive(Default)]
struct FakeVibeService {
    requests: Mutex<HashMap<&'static str, Vec<RecordedRequest>>>,
    /// If set, `SetVibe` returns this status instead of a response.
    set_vibe_err: Mutex<Option<tonic::Status>>,
}

/// A request recorded by [`FakeVibeService`], as the one field each test
/// below needs to assert on (Go's `fakeVibeService.lastRequest` returns
/// `any` and type-asserts; this enum is this file's equivalent without
/// reaching for `Box<dyn Any>`).
#[derive(Debug)]
enum RecordedRequest {
    SetVibe(SetVibeRequest),
    GetVibe(GetVibeRequest),
    SetVibeDetails(SetVibeDetailsRequest),
    SetVibeArray(SetVibeArrayRequest),
    SetVibeObjects(SetVibeObjectsRequest),
}

impl FakeVibeService {
    fn record(&self, name: &'static str, req: RecordedRequest) {
        self.requests
            .lock()
            .unwrap()
            .entry(name)
            .or_default()
            .push(req);
    }

    /// How many times `name` was called.
    fn call_count(&self, name: &str) -> usize {
        self.requests.lock().unwrap().get(name).map_or(0, Vec::len)
    }
}

#[tonic::async_trait]
impl VibeService for FakeVibeService {
    async fn set_vibe(
        &self,
        request: tonic::Request<SetVibeRequest>,
    ) -> Result<tonic::Response<SetVibeResponse>, tonic::Status> {
        let req = request.into_inner();
        self.record("SetVibe", RecordedRequest::SetVibe(req.clone()));
        if let Some(status) = self.set_vibe_err.lock().unwrap().clone() {
            return Err(status);
        }
        Ok(tonic::Response::new(SetVibeResponse {
            previous_vibe: "chill".to_string(),
            vibe: req.vibe,
        }))
    }

    async fn get_vibe(
        &self,
        request: tonic::Request<GetVibeRequest>,
    ) -> Result<tonic::Response<GetVibeResponse>, tonic::Status> {
        self.record("GetVibe", RecordedRequest::GetVibe(request.into_inner()));
        Ok(tonic::Response::new(GetVibeResponse {
            vibe: "immaculate".to_string(),
        }))
    }

    async fn set_vibe_details(
        &self,
        request: tonic::Request<SetVibeDetailsRequest>,
    ) -> Result<tonic::Response<SetVibeResponse>, tonic::Status> {
        let req = request.into_inner();
        self.record(
            "SetVibeDetails",
            RecordedRequest::SetVibeDetails(req.clone()),
        );
        Ok(tonic::Response::new(SetVibeResponse {
            previous_vibe: "mellow".to_string(),
            vibe: req.vibe,
        }))
    }

    /// Returns a fixed response, independent of `req`, so a test can
    /// assert on the tool's result content without that assertion
    /// trivially passing no matter what the handler actually put there
    /// (see `call_set_vibe_array_sends_request_and_returns_backend_result`).
    async fn set_vibe_array(
        &self,
        request: tonic::Request<SetVibeArrayRequest>,
    ) -> Result<tonic::Response<SetVibeArrayResponse>, tonic::Status> {
        self.record(
            "SetVibeArray",
            RecordedRequest::SetVibeArray(request.into_inner()),
        );
        Ok(tonic::Response::new(SetVibeArrayResponse {
            vibe_array: Some(VibeArray {
                vibe_bools: vec![false],
                ..Default::default()
            }),
        }))
    }

    /// Returns a fixed response, independent of `req`, for the same
    /// reason as `set_vibe_array` above (see
    /// `call_set_vibe_objects_sends_request_and_returns_backend_result`).
    async fn set_vibe_objects(
        &self,
        request: tonic::Request<SetVibeObjectsRequest>,
    ) -> Result<tonic::Response<SetVibeObjectsResponse>, tonic::Status> {
        self.record(
            "SetVibeObjects",
            RecordedRequest::SetVibeObjects(request.into_inner()),
        );
        Ok(tonic::Response::new(SetVibeObjectsResponse {
            vibe_object: vec![SomeVibeObject {
                vibe: "canned".to_string(),
            }],
        }))
    }
}

/// `ToolInfo` is the MCP-client-agnostic shape of a `tools/list` entry:
/// just the JSON an MCP client would see on the wire (Go's `toolInfo`).
struct ToolInfo {
    name: String,
    description: String,
    input_schema: Value,
}

/// `ToolResult` is the MCP-client-agnostic shape of a `tools/call`
/// result: the text of its (single) content item and whether the call
/// was an error (Go's `toolResult`).
struct ToolResult {
    text: String,
    is_error: bool,
}

/// Wires a fake gRPC backend ([`FakeVibeService`]) to the generated MCP
/// server over a real in-process TCP connection, and an MCP client to
/// that server over an in-memory duplex transport, mirroring Go's
/// `newTestHarness`. Returns the backend, shared behind an `Arc` so a
/// test can both assert on its recorded requests and set `SetVibe`'s
/// canned error (`fake.set_vibe_err`) directly, and the connected rmcp
/// client.
async fn new_test_harness() -> (
    std::sync::Arc<FakeVibeService>,
    rmcp::service::RunningService<rmcp::RoleClient, ()>,
) {
    let fake = std::sync::Arc::new(FakeVibeService::default());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind ephemeral port");
    let addr = listener.local_addr().expect("local_addr");
    let incoming = TcpIncoming::from(listener);
    let backend = std::sync::Arc::clone(&fake);
    tokio::spawn(
        tonic::transport::Server::builder()
            .add_service(VibeServiceServer::from_arc(backend))
            .serve_with_incoming(incoming),
    );
    let channel = VibeServiceClient::connect(format!("http://{addr}"))
        .await
        .expect("connect to in-process server");

    let mut mcp_server = VibeServiceMcpServer::new(channel);
    mcp_server.register_default_tools();

    let (server_io, client_io) = tokio::io::duplex(64 * 1024);
    tokio::spawn(async move {
        let running = mcp_server.serve(server_io).await.expect("serve");
        let _ = running.waiting().await;
    });
    let client = ().serve(client_io).await.expect("client connect");

    (fake, client)
}

/// Sends a `tools/list` request and returns the tools the server
/// reports (Go's `testHarness.listTools`).
async fn list_tools(client: &rmcp::service::RunningService<rmcp::RoleClient, ()>) -> Vec<ToolInfo> {
    client
        .list_all_tools()
        .await
        .expect("list_all_tools")
        .into_iter()
        .map(|tool| ToolInfo {
            name: tool.name.to_string(),
            description: tool.description.as_deref().unwrap_or_default().to_string(),
            input_schema: tool.schema_as_json_value(),
        })
        .collect()
}

/// Sends a `tools/call` request for `name` with `args` as the top-level
/// arguments object and returns its result as plain text and an
/// `is_error` flag (Go's `testHarness.callTool`).
async fn call_tool(
    client: &rmcp::service::RunningService<rmcp::RoleClient, ()>,
    name: &'static str,
    args: JsonObject,
) -> ToolResult {
    let result = client
        .call_tool(CallToolRequestParams::new(name).with_arguments(args))
        .await
        .expect("tools/call should not be a protocol-level error");
    assert_eq!(result.content.len(), 1, "expected exactly one content item");
    let rmcp::model::ContentBlock::Text(text) = &result.content[0] else {
        panic!("expected a text content item, got {:?}", result.content[0]);
    };
    ToolResult {
        text: text.text.clone(),
        is_error: result.is_error.unwrap_or(false),
    }
}

fn args(pairs: &[(&str, serde_json::Value)]) -> JsonObject {
    let mut m = JsonObject::new();
    for (k, v) in pairs {
        m.insert((*k).to_string(), v.clone());
    }
    m
}

/// The counterpart of Go's `TestToolsList`: `tools/list` reports all
/// five tools, each with its description and the request message's
/// fields as the top-level input schema properties (not nested under a
/// property named after the request message).
#[tokio::test]
async fn tools_list_reports_all_five_tools_with_descriptions_and_schemas() {
    let (_fake, client) = new_test_harness().await;

    let tools = list_tools(&client).await;
    let mut names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
    names.sort_unstable();
    assert_eq!(
        names,
        vec![
            "GetVibe",
            "SetVibe",
            "SetVibeArray",
            "SetVibeDetails",
            "SetVibeObjects",
        ]
    );

    let by_name: HashMap<&str, &ToolInfo> = tools.iter().map(|t| (t.name.as_str(), t)).collect();

    let set_vibe = by_name["SetVibe"];
    assert_eq!(
        set_vibe.description,
        "This is a block comment with multiple lines to test block handling \"Hello World\", \
         a `backtick`, and a path like C:\\vibes\\new"
    );
    assert_eq!(set_vibe.input_schema["type"], "object");
    assert_eq!(set_vibe.input_schema["additionalProperties"], false);
    assert_eq!(
        set_vibe.input_schema["properties"],
        serde_json::json!({
            "vibe": {
                "description": "The vibe of the server to be set. Must match \\d+ or a `code` like \"chill\", and must not contain a literal newline.",
                "type": "string",
            }
        })
    );
    assert!(
        set_vibe.input_schema["properties"]
            .get("SetVibeRequest")
            .is_none()
    );

    let get_vibe = by_name["GetVibe"];
    assert_eq!(get_vibe.description, "Get Vibe of the server");
    assert_eq!(get_vibe.input_schema["type"], "object");
    assert_eq!(get_vibe.input_schema["properties"], serde_json::json!({}));

    let set_vibe_details = by_name["SetVibeDetails"];
    assert_eq!(set_vibe_details.description, "Set vibe details");
    assert_eq!(set_vibe_details.input_schema["type"], "object");
    assert!(set_vibe_details.input_schema["properties"]["vibe"].is_object());
    assert!(set_vibe_details.input_schema["properties"]["vibeScalar"].is_object());

    let set_vibe_array = by_name["SetVibeArray"];
    assert_eq!(set_vibe_array.description, "Set the vibe arrays");
    assert_eq!(set_vibe_array.input_schema["type"], "object");
    assert!(set_vibe_array.input_schema["properties"]["vibeArray"].is_object());

    let set_vibe_objects = by_name["SetVibeObjects"];
    assert_eq!(set_vibe_objects.description, "Set multiple vibe objects");
    assert_eq!(set_vibe_objects.input_schema["type"], "object");
    assert!(set_vibe_objects.input_schema["properties"]["vibeObject"].is_object());
}

/// The counterpart of Go's `TestCallSetVibe`: `SetVibe` reaches the
/// backend with the given argument, exactly once, and the result is the
/// backend's pbjson-encoded response.
#[tokio::test]
async fn call_set_vibe_sends_request_and_returns_backend_result() {
    let (fake, client) = new_test_harness().await;

    let result = call_tool(
        &client,
        "SetVibe",
        args(&[("vibe", serde_json::json!("radical"))]),
    )
    .await;
    assert!(!result.is_error);
    assert_eq!(fake.call_count("SetVibe"), 1);

    let reqs = fake.requests.lock().unwrap();
    let Some(RecordedRequest::SetVibe(req)) = reqs.get("SetVibe").and_then(|v| v.last()) else {
        panic!("fake backend did not receive a SetVibeRequest");
    };
    assert_eq!(req.vibe, "radical");
    drop(reqs);

    let body: HashMap<String, Value> = serde_json::from_str(&result.text).expect("valid JSON");
    assert_eq!(body["previousVibe"], "chill");
    assert_eq!(body["vibe"], "radical");
}

/// The counterpart of Go's `TestCallGetVibe`.
#[tokio::test]
async fn call_get_vibe_sends_request_and_returns_backend_result() {
    let (fake, client) = new_test_harness().await;

    let result = call_tool(&client, "GetVibe", JsonObject::new()).await;
    assert!(!result.is_error);

    let reqs = fake.requests.lock().unwrap();
    assert!(
        matches!(
            reqs.get("GetVibe").and_then(|v| v.last()),
            Some(RecordedRequest::GetVibe(_))
        ),
        "fake backend did not receive a GetVibeRequest"
    );
    drop(reqs);

    let body: HashMap<String, Value> = serde_json::from_str(&result.text).expect("valid JSON");
    assert_eq!(body["vibe"], "immaculate");
}

/// The counterpart of Go's `TestCallSetVibeDetails`. This input/output
/// pair is also this issue's second parity assertion: see
/// `set_vibe_details_result_matches_go_tools_call_result` below.
#[tokio::test]
async fn call_set_vibe_details_sends_request_and_returns_backend_result() {
    let (fake, client) = new_test_harness().await;

    let result = call_tool(
        &client,
        "SetVibeDetails",
        args(&[
            ("vibe", serde_json::json!("groovy")),
            (
                "vibeScalar",
                serde_json::json!({
                    "vibeBool": true,
                    "vibeDouble": 3.5,
                    "vibeInt32": 42,
                }),
            ),
        ]),
    )
    .await;
    assert!(!result.is_error, "{}", result.text);

    let reqs = fake.requests.lock().unwrap();
    let Some(RecordedRequest::SetVibeDetails(req)) =
        reqs.get("SetVibeDetails").and_then(|v| v.last())
    else {
        panic!("fake backend did not receive a SetVibeDetailsRequest");
    };
    assert_eq!(req.vibe, "groovy");
    let scalar = req.vibe_scalar.as_ref().expect("vibeScalar should be set");
    assert!(scalar.vibe_bool);
    assert_eq!(scalar.vibe_double, 3.5);
    assert_eq!(scalar.vibe_int32, 42);
    drop(reqs);

    let body: HashMap<String, Value> = serde_json::from_str(&result.text).expect("valid JSON");
    assert_eq!(body["vibe"], "groovy");
}

/// The counterpart of Go's `TestCallSetVibeArray`.
#[tokio::test]
async fn call_set_vibe_array_sends_request_and_returns_backend_result() {
    let (fake, client) = new_test_harness().await;

    let result = call_tool(
        &client,
        "SetVibeArray",
        args(&[(
            "vibeArray",
            serde_json::json!({"vibeBools": [true, false, true]}),
        )]),
    )
    .await;
    assert!(!result.is_error);

    let reqs = fake.requests.lock().unwrap();
    let Some(RecordedRequest::SetVibeArray(req)) = reqs.get("SetVibeArray").and_then(|v| v.last())
    else {
        panic!("fake backend did not receive a SetVibeArrayRequest");
    };
    let vibe_array = req
        .vibe_array
        .as_ref()
        .expect("the vibe_array message field itself should be populated");
    assert_eq!(vibe_array.vibe_bools, vec![true, false, true]);
    drop(reqs);

    // Assert the result content exactly, against the fake backend's fixed
    // response (see FakeVibeService::set_vibe_array), so a handler that
    // writes the wrong value (or drops the field) is caught instead of
    // just checking the key exists.
    let body: Value = serde_json::from_str(&result.text).expect("valid JSON");
    assert_eq!(body["vibeArray"], serde_json::json!({"vibeBools": [false]}));
}

/// The counterpart of Go's `TestCallSetVibeObjects`.
#[tokio::test]
async fn call_set_vibe_objects_sends_request_and_returns_backend_result() {
    let (fake, client) = new_test_harness().await;

    let result = call_tool(
        &client,
        "SetVibeObjects",
        args(&[("vibeObject", serde_json::json!([{"vibe": "one"}]))]),
    )
    .await;
    assert!(!result.is_error);

    let reqs = fake.requests.lock().unwrap();
    let Some(RecordedRequest::SetVibeObjects(req)) =
        reqs.get("SetVibeObjects").and_then(|v| v.last())
    else {
        panic!("fake backend did not receive a SetVibeObjectsRequest");
    };
    assert_eq!(
        req.vibe_object,
        vec![SomeVibeObject {
            vibe: "one".to_string()
        }]
    );
    drop(reqs);

    // Assert the result content exactly, against the fake backend's fixed
    // response (see FakeVibeService::set_vibe_objects), for the same
    // reason as call_set_vibe_array above.
    let body: Value = serde_json::from_str(&result.text).expect("valid JSON");
    assert_eq!(body["vibeObject"], serde_json::json!([{"vibe": "canned"}]));
}

/// The counterpart of Go's `TestCallSetVibe_GRPCError`: a gRPC error
/// becomes `isError: true` with grpc-go's `Status.Error()` text
/// (`"rpc error: code = <Code> desc = <message>"`), asserted on the
/// actual wire text so a change to the gRPC code (not just the message)
/// is caught.
#[tokio::test]
async fn call_set_vibe_grpc_error_becomes_tool_error() {
    let (fake, client) = new_test_harness().await;
    *fake.set_vibe_err.lock().unwrap() = Some(tonic::Status::not_found("vibe not found"));

    let result = call_tool(
        &client,
        "SetVibe",
        args(&[("vibe", serde_json::json!("nope"))]),
    )
    .await;

    assert!(result.is_error);
    assert_eq!(
        result.text,
        "rpc error: code = NotFound desc = vibe not found"
    );
}

/// The counterpart of Go's `TestCallSetVibe_InvalidArgumentType`: a wrong
/// argument type (a number where the schema wants a string) fails schema
/// validation before decoding, and the backend is never called.
#[tokio::test]
async fn call_set_vibe_invalid_argument_type_never_reaches_backend() {
    let (fake, client) = new_test_harness().await;

    let result = call_tool(&client, "SetVibe", args(&[("vibe", serde_json::json!(42))])).await;

    assert!(result.is_error);
    assert_eq!(
        fake.call_count("SetVibe"),
        0,
        "invalid arguments must never reach the backend"
    );
}

/// The counterpart of Go's `TestCallSetVibe_UnknownField`: an
/// unrecognized field fails schema validation (`additionalProperties:
/// false`) rather than reaching the backend.
#[tokio::test]
async fn call_set_vibe_unknown_field_never_reaches_backend() {
    let (fake, client) = new_test_harness().await;

    let result = call_tool(
        &client,
        "SetVibe",
        args(&[
            ("vibe", serde_json::json!("radical")),
            ("unknownField", serde_json::json!("surprise")),
        ]),
    )
    .await;

    assert!(result.is_error);
    assert_eq!(
        fake.call_count("SetVibe"),
        0,
        "invalid arguments must never reach the backend"
    );
}

/// **Parity fixture (part 1 of 2):** `tools/list`'s result is JSON-equal
/// to `parity/go-v0.3.0-tools.json`, captured from Go's committed
/// `example_mcp.pb.go` (`stablekernel/protoc-gen-go-mcp` `9e072f9`; see
/// `parity/README.md` for exactly how). Tool
/// *registration order* is deliberately not checked here (this
/// generator keeps tools in a `BTreeMap`, alphabetical by name, unlike
/// Go's insertion-ordered slice; `examples/tests/mcp_tool_parity.rs`
/// already pins each tool's own fields against the same Go source, in
/// Go's order), so this test sorts both sides by name before comparing.
#[tokio::test]
async fn tools_list_matches_go_v0_3_0_parity_fixture() {
    let (_fake, client) = new_test_harness().await;

    let mut got: Vec<Value> = list_tools(&client)
        .await
        .into_iter()
        .map(|t| {
            serde_json::json!({
                "name": t.name,
                "description": t.description,
                "inputSchema": t.input_schema,
            })
        })
        .collect();
    got.sort_by_key(|t| t["name"].as_str().unwrap().to_string());

    let fixture: Value = serde_json::from_str(include_str!("parity/go-v0.3.0-tools.json"))
        .expect("parsing parity fixture");
    let mut want: Vec<Value> = fixture["tools"].as_array().expect("tools array").clone();
    want.sort_by_key(|t| t["name"].as_str().unwrap().to_string());

    assert_eq!(got, want, "tools/list must be JSON-equal to Go v0.3.0's");
}

/// **Parity fixture (part 2 of 2):** the `tools/call` result text for
/// `SetVibeDetails` with this test's input is JSON-equal to what Go
/// v0.3.0's generated handler produces for the same input
/// (`protojson.Marshal` on the backend's `SetVibeResponse{PreviousVibe:
/// "mellow", Vibe: "groovy"}`, both fields non-zero so protojson, like
/// pbjson, includes both: `{"previousVibe":"mellow","vibe":"groovy"}`;
/// confirmed by reading protojson's marshaling rules, since no Go
/// toolchain was available to run this directly — see this test file's
/// module doc comment and the PR description for details). Uses the same
/// input as `call_set_vibe_details_sends_request_and_returns_backend_result`
/// above, against this crate's own `FakeVibeService`, whose
/// `SetVibeDetails` returns `PreviousVibe: "mellow"` the same way Go's
/// `fakeVibeService` does.
#[tokio::test]
async fn set_vibe_details_result_matches_go_tools_call_result() {
    let (_fake, client) = new_test_harness().await;

    let result = call_tool(
        &client,
        "SetVibeDetails",
        args(&[
            ("vibe", serde_json::json!("groovy")),
            (
                "vibeScalar",
                serde_json::json!({
                    "vibeBool": true,
                    "vibeDouble": 3.5,
                    "vibeInt32": 42,
                }),
            ),
        ]),
    )
    .await;
    assert!(!result.is_error, "{}", result.text);

    let got: Value = serde_json::from_str(&result.text).expect("valid JSON");
    let want: Value =
        serde_json::from_str(r#"{"previousVibe":"mellow","vibe":"groovy"}"#).expect("valid JSON");
    assert_eq!(got, want);
}
