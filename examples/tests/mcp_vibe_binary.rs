//! Spawns the real `mcp-vibe` binary as a subprocess and drives it over
//! stdio with a real MCP client (issue #10's acceptance criteria):
//! `initialize`, `notifications/initialized`, `tools/list`, and a
//! `tools/call` of `SetVibe` then `GetVibe`. Unlike `examples/tests/handler.rs`
//! (which exercises the generated handler in-process), this is the only
//! test that runs the actual `mcp-vibe` process end to end, proving stdio
//! framing, the in-process gRPC backend wiring, and clean shutdown all
//! work together.

use rmcp::ServiceExt;
use rmcp::model::{CallToolRequestParams, JsonObject};
use rmcp::transport::{ConfigureCommandExt, TokioChildProcess};

fn text_of(result: &rmcp::model::CallToolResult) -> &str {
    let rmcp::model::ContentBlock::Text(text) = &result.content[0] else {
        panic!("expected text content");
    };
    &text.text
}

#[tokio::test]
async fn initialize_list_tools_and_call_set_then_get_vibe() {
    let transport = TokioChildProcess::new(
        tokio::process::Command::new(env!("CARGO_BIN_EXE_mcp-vibe")).configure(|_cmd| {}),
    )
    .expect("spawn mcp-vibe");

    let client = ().serve(transport).await.expect("initialize handshake");

    // `initialize` + `notifications/initialized` already happened as part
    // of `serve`'s handshake; `peer_info` exposes the result.
    let info = client.peer_info().expect("peer info after initialize");
    let server_info = info
        .server_info
        .as_ref()
        .expect("server_info present after initialize");
    assert_eq!(server_info.name, "vibe");
    assert_eq!(server_info.version, "0.0.1");

    let tools = client.list_all_tools().await.expect("tools/list");
    let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_ref()).collect();
    assert!(
        tool_names.contains(&"SetVibe"),
        "expected SetVibe among {tool_names:?}"
    );
    assert!(
        tool_names.contains(&"GetVibe"),
        "expected GetVibe among {tool_names:?}"
    );

    let mut args = JsonObject::new();
    args.insert("vibe".to_string(), serde_json::json!("chill"));
    let set_result = client
        .call_tool(CallToolRequestParams::new("SetVibe").with_arguments(args))
        .await
        .expect("tools/call SetVibe");
    assert_eq!(set_result.is_error, Some(false));
    assert_eq!(
        text_of(&set_result),
        r#"{"previousVibe":"initial vibe","vibe":"chill"}"#
    );

    let get_result = client
        .call_tool(CallToolRequestParams::new("GetVibe"))
        .await
        .expect("tools/call GetVibe");
    assert_eq!(get_result.is_error, Some(false));
    assert_eq!(text_of(&get_result), r#"{"vibe":"chill"}"#);

    client.cancel().await.expect("clean shutdown");
}
