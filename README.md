protoc-gen-rust-mcp
-----------
This is a [Topeka](#topeka) plugin for the [protoc compiler](https://grpc.io/docs/protoc-installation/) that generates a [model-context-protocol(MCP)](https://modelcontextprotocol.io/introduction) server based on a [protocol buffer](https://protobuf.dev/) definition. Conceptually, this allows an AI model to use existing [gRPC](https://grpc.io/) codebases with natural language, allowing for rapid prototyping and usage of LLM capabilities for protobuf based codebases.

This is the Rust counterpart of Topeka's Go plugin
[`stablekernel/protoc-gen-go-mcp`](https://github.com/stablekernel/protoc-gen-go-mcp):
for the same `.proto`, it generates the same tool names, descriptions and
input schemas, on [prost](https://crates.io/crates/prost),
[tonic](https://crates.io/crates/tonic) and [pbjson](https://crates.io/crates/pbjson)
instead of Go's protobuf/gRPC/protojson stack, and the official
[Rust MCP SDK](https://crates.io/crates/rmcp) (`rmcp`) instead of Go's
official MCP SDK.

#### Prerequisites
- The [Rust toolchain](https://www.rust-lang.org/tools/install), pinned in
  [`rust-toolchain.toml`](./rust-toolchain.toml) (checked against the
  factory image's installed `rustc --version`/`cargo --version` when that
  file was added; `rustup` picks up the pin automatically once you're in
  this repository).
- [protoc](https://grpc.io/docs/protoc-installation/) 29.3 (the version
  this plugin is tested against; see [`Makefile`](./Makefile) and
  [`AGENTS.md`](./AGENTS.md)).
- [`protoc-gen-prost`](https://crates.io/crates/protoc-gen-prost) 0.5.0,
  [`protoc-gen-tonic`](https://crates.io/crates/protoc-gen-tonic) 0.5.0 and
  [`protoc-gen-prost-serde`](https://crates.io/crates/protoc-gen-prost-serde)
  0.4.0 on `PATH`, the three `protoc` plugins that generate the
  prost/tonic/pbjson code this plugin's own output sits next to:
  ```sh
  cargo install --locked protoc-gen-prost@0.5.0 protoc-gen-tonic@0.5.0 protoc-gen-prost-serde@0.4.0
  ```

The generated code itself (not this plugin) depends on these crates at
runtime; regenerating with different versions changes the generated API,
so document any change in your PR, same as this repository's own
[`AGENTS.md`](./AGENTS.md) asks of its committed example:

| crate | version | why |
| --- | --- | --- |
| [`prost`](https://crates.io/crates/prost), [`prost-types`](https://crates.io/crates/prost-types) | 0.14 | message types the generated tool arguments/results decode into |
| [`tonic`](https://crates.io/crates/tonic) | 0.14 | the gRPC client the generated handler forwards tool calls to |
| [`tonic-prost`](https://crates.io/crates/tonic-prost) | 0.14 | tonic's prost codec, needed by `protoc-gen-tonic`'s own output |
| [`pbjson`](https://crates.io/crates/pbjson), [`pbjson-types`](https://crates.io/crates/pbjson-types) | **0.8.x** (pinned, to match the `pbjson-build` `protoc-gen-prost-serde` 0.4.0 uses) | the protojson-compatible JSON (de)serialization tool arguments/results use |
| [`rmcp`](https://crates.io/crates/rmcp), features `server` + whatever transport you serve over (e.g. `transport-io` for stdio, `transport-streamable-http-server` for Streamable HTTP) | 3.5 | the official Rust MCP SDK the generated server implements `ServerHandler` against |
| [`serde_json`](https://crates.io/crates/serde_json) | 1 | the `JsonObject`/`Value` types tool arguments and results pass through |
| [`jsonschema`](https://crates.io/crates/jsonschema), `default-features = false` (every generated schema is self-contained, so no HTTP/file `$ref` resolution is needed) | 0.58 | validates a tool call's raw arguments against its input schema before decoding, the validation Go's SDK does and `rmcp` itself does not |
| [`futures`](https://crates.io/crates/futures) | 0.3 | `BoxFuture`, the generated tool handler's return type |

See [`examples/Cargo.toml`](./examples/Cargo.toml) for the exact feature
sets the committed example server (`mcp-vibe`) needs on top of this.

#### Running the plugin
Check out the [`Makefile`](./Makefile) for explicit command usage. Use
`make generate` to regenerate the example MCP server from the
[proto file](./examples/protos/example.proto) with the real `protoc`
pipeline (`protoc-gen-prost`, `protoc-gen-tonic`, `protoc-gen-prost-serde`,
then this plugin).

Outside `make generate`, invoke `protoc` directly, pointing `--rust-
mcp_out` at the same directory as the other three plugins' output (this
plugin's generated `<package>.mcp.rs` is spliced into prost's own
`<package>.rs` via an `include!`, the same convention
`protoc-gen-tonic`/`protoc-gen-prost-serde` use for their own output, so
they must share a directory and run against the same `--proto_path`):

```sh
protoc \
  --proto_path=examples/protos \
  --prost_out=examples/src/gen \
  --tonic_out=examples/src/gen \
  --prost-serde_out=examples/src/gen \
  --plugin=protoc-gen-rust-mcp=target/debug/protoc-gen-rust-mcp \
  --rust-mcp_out=examples/src/gen \
  examples/protos/example.proto
```

The result is one `<package>.rs` per proto package (prost's own output,
with the other three plugins' `include!` lines spliced in at its
`// @@protoc_insertion_point(module)` marker) plus this plugin's own
standalone `<package>.mcp.rs`. A crate that vendors generated code
`include!`s just the top-level `<package>.rs` (see
[`examples/src/lib.rs`](./examples/src/lib.rs)):

```rust,ignore
pub mod v1 {
    include!("gen/examples/v1/examples.v1.rs");
}
```

#### Debugging the plugin
Like any `protoc` plugin, `protoc-gen-rust-mcp` reads an encoded
`CodeGeneratorRequest` on stdin and writes an encoded
`CodeGeneratorResponse` to stdout, so you can't attach a debugger to it
while `protoc` is driving it (there's no interactive terminal to attach
through, and `protoc` doesn't wait around). Instead, save the request
once and replay it as many times as you like outside `protoc`:

```sh
protoc --proto_path=examples/protos \
  --plugin=protoc-gen-debug=./protoc-gen-debug \
  --debug_out=. examples/protos/example.proto
```

[`protoc-gen-debug`](./protoc-gen-debug) is a tiny script that saves
whatever `protoc` sends it on stdin to `request.bin` in the current
directory (the counterpart of Go's own `protoc-gen-debug`, which instead
starts `dlv --headless`; Rust's debuggers don't have a direct equivalent
of that, hence the two-step version here). `protoc` reports an error
afterwards (`--debug_out` never gets a real `CodeGeneratorResponse` back),
but the request is already saved by then.

Replay it directly:

```sh
cargo build -p protoc-gen-rust-mcp
./target/debug/protoc-gen-rust-mcp < request.bin > /dev/null
```

or under a real debugger. `rustup` installs the
[`rust-lldb`](https://doc.rust-lang.org/rustc/debugging-support-tools.html)/`rust-gdb`
wrapper scripts (pretty-printer-aware front ends for a real `lldb`/`gdb`)
alongside `rustc` by default, so if your machine has `lldb` or `gdb`
itself installed (neither is in the minimal factory image this repo's
`AGENTS.md` checks run in — install one to use this locally):

```sh
rust-lldb -- ./target/debug/protoc-gen-rust-mcp < request.bin
# or: rust-gdb -- ./target/debug/protoc-gen-rust-mcp < request.bin
```

or in VS Code with the
[CodeLLDB](https://marketplace.visualstudio.com/items?itemName=vadimcn.vscode-lldb)
extension (which bundles its own LLDB, so it needs no separate `lldb`
install) and the committed [`.vscode/launch.json`](./.vscode/launch.json)
(`Debug protoc-gen-rust-mcp (replay saved request)`), which builds the
plugin and redirects `request.bin` to its stdin for you.

#### Testing the example
Install the example `mcp-vibe` server:
```sh
cargo install --path examples --bin mcp-vibe
```
Add the `mcp-vibe` server to your mcp servers:
```json
{
  "mcpServers": {
    "vibe": {
      "command": "mcp-vibe"
    }
  }
}
```
and run a client with above mcp server attached (eg. claude desktop)

#### What the plugin generates
A generated `<Service>McpServer<T>` wraps a tonic `<Service>Client<T>` and
implements [`rmcp::ServerHandler`](https://docs.rs/rmcp/latest/rmcp/trait.ServerHandler.html)
directly (see [`protoc-gen-rust-mcp/src/server.rs`](./protoc-gen-rust-mcp/src/server.rs)'s
module doc comment for why it's hand-implemented rather than built on
`rmcp`'s own `ToolRouter`). Once you've called `register_default_tools()`
(or registered individual tools, see below), serve it the way any
`rmcp::ServerHandler` is served. The two most common transports (from
[`examples/tests/wiring.rs`](./examples/tests/wiring.rs), compiled by
`cargo test` — see that file's doc comment):

```rust,ignore
// client is an examples::v1::vibe_service_client::VibeServiceClient for
// the backend gRPC service, e.g. VibeServiceClient::new(channel) for a
// tonic::transport::Channel.

// Over stdio, for CLI-launched MCP clients (see
// examples/src/bin/mcp-vibe.rs for the full, runnable version of this).
let mut server = VibeServiceMcpServer::new(client)
    .with_server_info(rmcp::model::Implementation::new("vibe", "0.0.1"));
server.register_default_tools();

let running = server.serve(rmcp::transport::io::stdio()).await?;
running.waiting().await?;
```

```rust,ignore
// Over Streamable HTTP, for network clients: rmcp's StreamableHttpService
// is a tower::Service, wired into a hyper server here via hyper-util's
// TowerToHyperService adapter (the same pairing rmcp's own
// examples/servers/src/counter_hyper_streamable_http.rs uses). client is
// the same VibeServiceClient as above; the service factory closure below
// builds a fresh VibeServiceMcpServer per session.
let service = TowerToHyperService::new(StreamableHttpService::new(
    move || {
        let mut server = VibeServiceMcpServer::new(client.clone())
            .with_server_info(rmcp::model::Implementation::new("vibe", "0.0.1"));
        server.register_default_tools();
        Ok::<_, std::io::Error>(server)
    },
    LocalSessionManager::default().into(),
    Default::default(),
));

let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await?;
loop {
    let (stream, _) = listener.accept().await?;
    let io = TokioIo::new(stream);
    let service = service.clone();
    tokio::spawn(async move {
        let _ = Builder::new(TokioExecutor::default())
            .serve_connection(io, service)
            .await;
    });
}
```

Both snippets are kept compiling in
[`examples/tests/wiring.rs`](./examples/tests/wiring.rs) (`serve_stdio`,
`serve_streamable_http`), so they stay in sync with the generated API;
`cargo build --tests`/`cargo test` cover that file on every run of the
[checks](./AGENTS.md) (it is a `tests/*.rs` file, so plain `cargo build`
skips it).

#### How tool arguments map to the request message
Each tool's `inputSchema` is the [JSON Schema](https://json-schema.org/)
for its RPC's request message, and the generated handler decodes incoming
arguments with the pbjson-generated `Deserialize`
(`serde_json::from_value`), the counterpart of Go's
[`protojson.Unmarshal`](https://pkg.go.dev/google.golang.org/protobuf/encoding/protojson#Unmarshal).
Schema validation runs first (see
[`protoc-gen-rust-mcp/src/server.rs`](./protoc-gen-rust-mcp/src/server.rs)'s
module doc comment): the raw arguments are checked against a
`jsonschema::Validator` compiled from the same schema embedded in the
tool's `inputSchema`, the validation Go's SDK does via `mcp.AddTool` and
`rmcp` itself does not. On failure every failing instance path and reason
is listed in the tool error text (e.g. ``invalid arguments: /vibe: 5 is
not of type "string"``), and the backend is never called. That means
tool arguments follow pbjson's mapping (matching protojson on every point
documented below), not the proto field names or Rust struct field names:

- **Field names are the field's JSON name** (lowerCamel by default, e.g. a
  proto field `previous_vibe` is the JSON key `previousVibe`), not the
  original snake_case proto name. The schema's `additionalProperties:
  false` rejects an unrecognized key — including the snake_case form —
  as a tool error rather than silently ignoring or coercing it (e.g.
  ``invalid arguments: : Additional properties are not allowed
  ('previous_vibe' was unexpected)``).
- **64-bit integers (`int64`, `uint64`, `sint64`, `fixed64`, `sfixed64`)
  are accepted as either a JSON number or a decimal string** (`"type":
  ["integer", "string"]`), matching protojson's own encoding of them as
  strings (to avoid precision loss in JSON's number type). All other
  integer kinds are plain JSON numbers.
- **Enum fields are their value's name as a string** (e.g. `"VIBE_GOOD"`),
  not the underlying numeric value, matching protojson's default enum
  encoding.
- **`bytes` fields are base64-encoded strings**, per protojson's `bytes`
  mapping.
- **`float`/`double` fields also accept the strings `"NaN"`, `"Infinity"`
  and `"-Infinity"`** (in addition to a JSON number), matching protojson's
  encoding of those non-finite values (a plain JSON number can't
  represent them) — see **Differences from protoc-gen-go-mcp** below for
  the one place this stops matching protojson: non-finite values in a
  tool *result*.
- Tool call results are pbjson-encoded the same way, so a result's keys
  and enum/int64 representations follow the same rules — **including
  that pbjson, like protojson, omits zero-valued fields** (an empty
  string, `0`, `false`, an unset enum, etc.) from the result entirely,
  rather than emitting them explicitly.
- **A gRPC error from the backend becomes a tool error** (`isError:
  true`) whose text matches grpc-go's `Status.Error()` format:
  ``rpc error: code = NotFound desc = vibe not found`` (`tonic::Code`'s
  `Debug` impl happens to render the same CamelCase names grpc-go uses,
  confirmed against `tonic::Code` 0.14.6).

#### Differences from protoc-gen-go-mcp
This plugin's generated code uses pbjson, not protojson, for tool
arguments and results. The two agree on every point in the previous
section, with these documented exceptions:

- **`google.protobuf.Any`** is pbjson-shaped, not protojson-shaped:
  `{"typeUrl": "...", "value": "<base64>"}` (the plain two-field message
  `Any` is on the wire), not protojson's unpacked-message-plus-injected-
  `"@type"` representation. pbjson has no special case for `Any` (see
  [`protoc-gen-rust-mcp/src/schema.rs`](./protoc-gen-rust-mcp/src/schema.rs)'s
  `any_schema` and
  [`protoc-gen-rust-mcp/src/server.rs`](./protoc-gen-rust-mcp/src/server.rs)'s
  module doc comment for exactly what pbjson emits).
- **Non-finite floats (`NaN`/`Infinity`/`-Infinity`) in tool *results*
  come out as JSON `null`**, not the strings protojson emits. Both
  pbjson and protojson *accept* all three spellings as tool call
  *input* (see the previous section's float/double rule); this
  divergence is output-only. See `schema.rs`'s `float_schema` and
  [`examples/tests/handler.rs`](./examples/tests/handler.rs)'s
  `non_finite_floats_in_result_are_encoded_as_json_null`.
- **No [editions](https://protobuf.dev/editions/overview/) support**:
  this plugin only advertises `FEATURE_PROTO3_OPTIONAL`, not
  `FEATURE_SUPPORTS_EDITIONS` (see `generator.rs`'s
  `FEATURE_PROTO3_OPTIONAL` doc comment for why — in short, neither
  `prost-reflect` nor `protoc-gen-prost` support editions files yet, so
  this stack can't generate editions code regardless of what this
  plugin advertises). `protoc` rejects an editions input file with a
  clear error instead of this plugin panicking on it.

One more well-known type gets special JSON handling, but it's made to
*match* Go rather than diverge from it: **`google.protobuf.FieldMask`**'s
schema is `{"type": "string"}` and its wire representation is protojson's
comma-joined lowerCamel-path string, even though pbjson's own
`Serialize`/`Deserialize` for it would otherwise be `{"paths": [...]}`.
Generated per-message code rewrites a FieldMask field's JSON between the
two shapes, between schema validation and the pbjson decode/encode (see
[`protoc-gen-rust-mcp/src/field_mask.rs`](./protoc-gen-rust-mcp/src/field_mask.rs)).

#### Philosophical Notes
The plugin uses the existing code generation for protocol buffers and gRPC servers and builds upon that base, using and reusing parts where necessary. This gives us a healthy amount of code reuse while allowing us to control what we expose to end users. We want this plugin to provide sane, out-of-the-box functionality while allowing for easy extension.

How is this achieved?

The code is broken into composable parts:

1. The `protoc-gen-rust-mcp` plugin generates [default tools](https://modelcontextprotocol.io/docs/concepts/tools) based on the request parameters for any given RPC.
eg:
```proto
message SetVibeRequest {
  string vibe = 1;
}

message SetVibeResponse {
  string previous_vibe = 1;
  string vibe = 2;
}

service VibeService {
  // Set Vibe
  rpc SetVibe(SetVibeRequest) returns (SetVibeResponse) {}
}
```
This snippet defines the `SetVibe` RPC, which takes a `SetVibeRequest` message and contains a definition of the request parameter `SetVibeRequest` message. The plugin generates the following tool by default:
```rust,ignore
// VIBE_SERVICE_SET_VIBE_INPUT_SCHEMA is a package-level constant holding
// the JSON Schema derived from SetVibeRequest.
impl<T> VibeServiceMcpServer<T> {
    pub fn set_vibe_tool() -> rmcp::model::Tool {
        rmcp::model::Tool::new(
            "SetVibe",
            "Set Vibe",
            VIBE_SERVICE_SET_VIBE_INPUT_SCHEMA.clone(),
        )
    }
}
```
This tool can be subsequently registered with the server to make the RPC available to the model.

2. The `protoc-gen-rust-mcp` plugin generates a default handler that interacts with a generated gRPC client for interaction with this server to parse the tool call arguments into a defined gRPC request leveraging a generated client.

```rust,ignore
impl<T> VibeServiceMcpServer<T> {
    async fn call_set_vibe(
        mut client: vibe_service_client::VibeServiceClient<T>,
        args: Option<rmcp::model::JsonObject>,
    ) -> rmcp::model::CallToolResult {
        // ... validate args against the schema, decode, call the client, encode the response
    }
}
```

3. These two pieces are combined upon registration to provide the LLM with knowledge of the RPC method and how to use them. `register_default_tools` builds each tool's handler from a clone of the server's own client, captured once per registration rather than once per call:
```rust,ignore
impl<T> VibeServiceMcpServer<T> {
    pub fn register_default_tools(&mut self) -> &mut Self {
        // ...other tools registered above
        {
            let client = self.client.clone();
            self.register_tool(Self::set_vibe_tool(), move |args| {
                Self::call_set_vibe(client.clone(), args)
            });
        }
        // ...other tools registered below
        self
    }
}
```
Every registered tool's arguments are validated against its input schema before the handler's decode step runs (see **How tool arguments map to the request message** above) — this plugin's own generated code does that validation itself, since (unlike Go's SDK) `rmcp` does not validate incoming arguments against a tool's schema.

#### Topeka
[Topeka](https://topeka.ai) is an open source project that provides code-generators for [Model-Context-Protocol (MCP)](https://modelcontextprotocol.io/introduction).
It is designed to facilitate the usage of MCP seamlessly against existing gRPC based applications. This is done via
leveraging code generation using the [protoc compiler](https://grpc.io/docs/protoc-installation/) and installing the relevant Topeka plugin.

The plugins follow [Semantic Versioning](https://semver.org/) and any plugin prior to 1.0.0 releases ARE still subject to breaking changes. Please note, this is
applied to the generated servers, not the plugins themselves, which do not provide public APIs. This project reserves the right to change how code generation is achieved,
while maintaining stable MCP server APIs.

#### Maintainers
[Stable Kernel](https://stablekernel.com) is the primary maintainer of this project and sponsor of the plugins, though we welcome outside contributions.

[Stable Kernel](https://stablekernel.com) is a digital transformation company building solutions that power LLM enablement for growing businesses. We have a track record of helping our partners solve their biggest challenges on their digital journey, whether they need insights or implementation. Every day, millions of people rely on software that we developed, and our custom software development and technology services have been trusted by some of the most innovative Fortune 500 companies in the world. 
