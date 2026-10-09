# Goal

`protoc-gen-rust-mcp` lets an existing gRPC service become an MCP server with
no hand-written glue: run `protoc` with this plugin (alongside the prost,
tonic and pbjson plugins), and each RPC becomes an MCP tool whose input schema
and handler come from the proto definition. AI clients can then call the
service in natural language.

## Current milestone: v0.1.0, parity with protoc-gen-go-mcp 0.3.0 (#2)

Port [`stablekernel/protoc-gen-go-mcp`](https://github.com/stablekernel/protoc-gen-go-mcp)
v0.3.0 (commit `9e072f9`) to Rust:

- **Same tools:** for the same `.proto`, the same tool names, descriptions
  and input schemas (JSON Schema derived from the proto descriptors) as the Go
  plugin generates. The example proto is shared, and a parity fixture
  captured from Go keeps the two in step.
- **Same behavior:** arguments are validated against the schema (with the
  `jsonschema` crate, since rmcp doesn't validate), decoded with pbjson, sent
  through a tonic client, and the response returned as pbjson text; bad
  arguments and gRPC errors become tool errors (`isError: true`), with
  grpc-go's `rpc error: code = … desc = …` text.
- **Stack:** prost + tonic + pbjson for protobuf/gRPC/JSON, and `rmcp` (the
  official Rust MCP SDK) for MCP.
- **Safety nets:** unit tests ported from Go, a golden-file test, an
  end-to-end test, and a CI check that the committed examples are current.
- **Same project shape:** the `mcp-vibe` example, release-please, release
  binaries, and a README that matches Go's.

## Out of scope for this milestone

- Streaming RPCs (skipped by the generator, as in Go).
- MCP resources and prompts.
- Publishing to crates.io, and a `build.rs` (prost-build/tonic-build)
  integration API.
