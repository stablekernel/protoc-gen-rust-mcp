//! **Parity:** after `register_default_tools()`, `VibeServiceMcpServer`'s
//! five tools (names, descriptions, input schemas) are JSON-equal to the
//! five tools Go's committed `examples/gen/example/v1/example_mcp.pb.go`
//! (`stablekernel/protoc-gen-go-mcp` `9e072f9`) registers. The expected
//! values are copied verbatim from that file (see
//! `protoc-gen-rust-mcp/src/schema.rs`'s
//! `parity_with_go_example_input_schemas` test, which pins the same input
//! schemas against the schema-builder in isolation; this test instead
//! checks the real generated `examples.v1.mcp.rs` end to end). Tool
//! *registration order* is deliberately not checked: this generator keeps
//! tools in a `BTreeMap` (alphabetical by name), unlike Go's
//! insertion-ordered slice, and MCP's `tools/list` does not make ordering
//! part of its contract.

use examples::v1::VibeServiceMcpServer;
use examples::v1::vibe_service_client::VibeServiceClient;
use rmcp::ServerHandler;
use serde_json::Value;

/// A `VibeServiceClient<Channel>` that never connects (`connect_lazy`):
/// this test only inspects registered tool metadata, never calls a tool.
fn lazy_client() -> VibeServiceClient<tonic::transport::Channel> {
    let channel = tonic::transport::Endpoint::from_static("http://127.0.0.1:1").connect_lazy();
    VibeServiceClient::new(channel)
}

#[tokio::test]
async fn register_default_tools_matches_go_committed_tools() {
    let mut server = VibeServiceMcpServer::new(lazy_client());
    server.register_default_tools();

    let get = |name: &str| {
        server
            .get_tool(name)
            .unwrap_or_else(|| panic!("tool {name} should be registered"))
    };

    // (name, description, input schema), in Go's `RegisterDefaultTools`
    // order; the input schema JSON is copied verbatim from Go's committed
    // example_mcp.pb.go.
    let cases: &[(&str, &str, &str)] = &[
        (
            "SetVibe",
            "This is a block comment with multiple lines to test block handling \"Hello World\", a `backtick`, and a path like C:\\vibes\\new",
            r#"{"additionalProperties":false,"description":"The request to set the vibe of the server","properties":{"vibe":{"description":"The vibe of the server to be set. Must match \\d+ or a `code` like \"chill\", and must not contain a literal newline.","type":"string"}},"type":"object"}"#,
        ),
        (
            "GetVibe",
            "Get Vibe of the server",
            r#"{"additionalProperties":false,"description":"The request to get the vibe of the server","properties":{},"type":"object"}"#,
        ),
        (
            "SetVibeDetails",
            "Set vibe details",
            r#"{"additionalProperties":false,"description":"The detailed vibe of the server","properties":{"vibe":{"description":"The vibe of the string to be set","type":"string"},"vibeScalar":{"additionalProperties":false,"description":"The details of the vibe","properties":{"vibeBool":{"description":"The details of the vibe bool","type":"boolean"},"vibeBytes":{"contentEncoding":"base64","description":"the details of the vibe bytes","type":"string"},"vibeDouble":{"anyOf":[{"type":"number"},{"enum":["NaN","Infinity","-Infinity"],"type":"string"}],"description":"The details of the vibe double"},"vibeEnum":{"description":"The details of the vibe string","items":{"description":"The details of the vibe string","enum":["VIBE_UNSET","VIBE_GOOD"],"type":"string"},"type":"array"},"vibeFixed32":{"description":"The details of the vibe fixed32","minimum":0,"type":"integer"},"vibeFixed64":{"description":"The details of the vibe fixed64","minimum":0,"type":["integer","string"]},"vibeFloat":{"anyOf":[{"type":"number"},{"enum":["NaN","Infinity","-Infinity"],"type":"string"}],"description":"the details of the vibe float"},"vibeInt32":{"description":"The details of the vibe int32","type":"integer"},"vibeInt64":{"description":"The details of the vibe int64","type":["integer","string"]},"vibeSfixed32":{"description":"The details of the vibe sfixed32","type":"integer"},"vibeSfixed64":{"description":"The details of the vibe sfixed64","type":["integer","string"]},"vibeSint32":{"description":"The details of the vibe sint32","type":"integer"},"vibeSint64":{"description":"The details of the vibe sint64","type":["integer","string"]},"vibeUint32":{"description":"The details of the vibe uint32","minimum":0,"type":"integer"},"vibeUint64":{"description":"The details of the vibe uint64","minimum":0,"type":["integer","string"]}},"type":"object"}},"type":"object"}"#,
        ),
        (
            "SetVibeArray",
            "Set the vibe arrays",
            r#"{"additionalProperties":false,"description":"The vibe array request","properties":{"vibeArray":{"additionalProperties":false,"description":"The details of the vibe array","properties":{"vibeBools":{"description":"The details of the vibe bool array","items":{"type":"boolean"},"type":"array"},"vibeByteses":{"description":"the details of the vibe bytes array","items":{"contentEncoding":"base64","type":"string"},"type":"array"},"vibeDoubles":{"description":"The details of the vibe double array","items":{"anyOf":[{"type":"number"},{"enum":["NaN","Infinity","-Infinity"],"type":"string"}]},"type":"array"},"vibeFixed32s":{"description":"The details of the vibe fixed32 array","items":{"minimum":0,"type":"integer"},"type":"array"},"vibeFixed64s":{"description":"The details of the vibe fixed64 array","items":{"minimum":0,"type":["integer","string"]},"type":"array"},"vibeFloats":{"description":"the details of the vibe float array","items":{"anyOf":[{"type":"number"},{"enum":["NaN","Infinity","-Infinity"],"type":"string"}]},"type":"array"},"vibeInt32s":{"description":"The details of the vibe int32 array","items":{"type":"integer"},"type":"array"},"vibeInt64s":{"description":"The details of the vibe int64 array","items":{"type":["integer","string"]},"type":"array"},"vibeSfixed32s":{"description":"The details of the vibe sfixed32 array","items":{"type":"integer"},"type":"array"},"vibeSfixed64s":{"description":"The details of the vibe sfixed64 array","items":{"type":["integer","string"]},"type":"array"},"vibeSint32s":{"description":"The details of the vibe sint32 array","items":{"type":"integer"},"type":"array"},"vibeSint64s":{"description":"The details of the vibe sint64 array","items":{"type":["integer","string"]},"type":"array"},"vibeUint32s":{"description":"The details of the vibe uint32 array","items":{"minimum":0,"type":"integer"},"type":"array"},"vibeUint64s":{"description":"The details of the vibe uint64 array","items":{"minimum":0,"type":["integer","string"]},"type":"array"}},"type":"object"}},"type":"object"}"#,
        ),
        (
            "SetVibeObjects",
            "Set multiple vibe objects",
            r#"{"additionalProperties":false,"description":"The request to set multiple vibe objects on the server","properties":{"vibeObject":{"description":"The details of the vibe","items":{"additionalProperties":false,"description":"The vibe object of the server","properties":{"vibe":{"description":"The vibe of the server","type":"string"}},"type":"object"},"type":"array"}},"type":"object"}"#,
        ),
    ];

    assert_eq!(
        cases.len(),
        5,
        "example.proto's tool count drifted from the Go parity fixture"
    );

    for (name, description, schema_json) in cases {
        let tool = get(name);
        assert_eq!(tool.name.as_ref(), *name);
        assert_eq!(
            tool.description.as_deref(),
            Some(*description),
            "description for {name}"
        );
        let want: Value =
            serde_json::from_str(schema_json).expect("parsing Go's expected schema JSON");
        let got = tool.schema_as_json_value();
        assert_eq!(got, want, "input schema for {name}");
    }

    // Names only (this also doubles as a quick sanity check on the exact
    // tool set, via `into_tools`, rather than just each one individually
    // via `get_tool`).
    let mut names: Vec<String> = server
        .into_tools()
        .map(|(tool, _)| tool.name.to_string())
        .collect();
    names.sort_unstable();
    assert_eq!(
        names,
        vec![
            "GetVibe".to_string(),
            "SetVibe".to_string(),
            "SetVibeArray".to_string(),
            "SetVibeDetails".to_string(),
            "SetVibeObjects".to_string(),
        ]
    );
}
