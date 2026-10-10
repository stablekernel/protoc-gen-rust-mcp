# Cross-implementation parity fixtures

`go-v0.3.0-tools.json` is the `tools/list` result Go's
[`protoc-gen-go-mcp`](https://github.com/stablekernel/protoc-gen-go-mcp)
v0.3.0 (commit `9e072f9`) generates for `examples/protos/example.proto`'s
`VibeService`: one entry per tool, each with its `name`, `description`
and `inputSchema`, in the shape an MCP client would see on the wire from
a real `tools/list` response.

It was captured by extracting those three fields from Go's **committed**
`examples/gen/example/v1/example_mcp.pb.go` at that commit (each tool's
`Name`, `Description` and `InputSchema` literal), not by running Go's
`mcp-vibe` over stdio: no Go toolchain was available in this environment
to `go run` it. The same extraction (and the same source literals) backs
`examples/tests/mcp_tool_parity.rs`'s `cases`, from issue #6/#21; if Go's
example proto or generator output ever changes, update both together.

`examples/tests/e2e.rs`'s
`tools_list_matches_go_v0_3_0_parity_fixture` test asserts the Rust
server's real `tools/list` response is JSON-equal to this fixture
(sorted by tool name, since this generator's registration order differs
from Go's — see that test's doc comment). The same file's
`set_vibe_details_result_matches_go_tools_call_result` test pins the
second parity assertion the issue asks for: `SetVibeDetails`'s
`tools/call` result text for a fixed input, against the value Go's
`protojson.Marshal` would produce for the same backend response (derived
by reading protojson's field-presence rules, for the same reason: no Go
toolchain to run it directly). See that test's doc comment for the exact
reasoning.
