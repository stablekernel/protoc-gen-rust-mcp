# AGENTS.md

Guidance for coding agents (and humans) working in this repository. See
[`docs/GOAL.md`](docs/GOAL.md) for what we're building and what's out of scope,
and [`CONTRIBUTING.md`](CONTRIBUTING.md) for the human process.

## What this is

`protoc-gen-rust-mcp` is a `protoc` plugin, the Rust counterpart of Topeka's Go
plugin [`stablekernel/protoc-gen-go-mcp`](https://github.com/stablekernel/protoc-gen-go-mcp).
For each gRPC service it generates an MCP (Model Context Protocol) server, on
the official Rust MCP SDK ([`rmcp`](https://crates.io/crates/rmcp)), that
exposes each non-streaming RPC as an MCP tool and forwards tool calls to a
[tonic](https://crates.io/crates/tonic) client. Messages are
[prost](https://crates.io/crates/prost) types, and tool arguments and results
are JSON via [pbjson](https://crates.io/crates/pbjson) (protojson-compatible).

**The Go plugin at v0.3.0 is the reference.** When this repo and the Go repo
disagree on tool names, descriptions, input schemas, or `tools/call`
behavior, that is a bug here unless an issue says otherwise. Read the Go
source (`cmd/protoc-gen-go-mcp/`, `examples/gen/example/v1/`) before porting
a piece of it.

Layout (being built by the parity milestone; see the issues):

- `protoc-gen-rust-mcp/`: the plugin crate.
- `examples/protos/example.proto`: the example `VibeService` (a copy of Go's).
- `examples/`: the crate with the committed code generated from it, and the
  `mcp-vibe` demo server.

## Checks

Run all of these before opening a pull request, and paste the commands and
their results in the PR:

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo build --workspace
cargo test --workspace
```

If you changed the generator or the example proto, regenerate the committed
examples and commit the result:

```sh
make generate   # needs protoc 29.3, protoc-gen-prost 0.5.0, protoc-gen-tonic 0.5.0,
                # protoc-gen-prost-serde 0.4.0 on PATH (all in the factory image)
git status      # the regenerated files belong in the same PR
```

Once the golden-file test exists, update it with
`UPDATE_GOLDEN=1 cargo test -p protoc-gen-rust-mcp golden` and review the diff;
an unexpected change in generated code is a bug until explained.

## Conventions

- **Pull request titles must be Conventional Commits** (release-please builds
  the changelog from them): `feat:`, `fix:`, `docs:`, `test:`, `ci:`,
  `refactor:`, `perf:`, `chore:`, `revert:`. Use `feat!:` or `fix!:` and a
  `BREAKING CHANGE:` paragraph in the body for breaking changes to the
  generated API.
- Reference the issue in the PR body (`Closes #N`).
- **Never edit generated files by hand** (anything under `examples/src/gen/`);
  change the generator or the proto and regenerate.
- Don't touch `CHANGELOG.md`, `.release-please-manifest.json` or
  `release-please-config.json`, and don't bump versions: release-please owns
  them (once it's set up).
- Don't change `.github/workflows/**` unless the issue asks for it.
- Keep dependencies minimal and pinned: commit `Cargo.lock`. The generated
  code's runtime dependencies (prost, tonic, pbjson, rmcp, serde_json,
  jsonschema) are part of its public contract; document any change in the PR.
- `pbjson`/`pbjson-types` stay on **0.8.x** to match the `pbjson-build` used by
  `protoc-gen-prost-serde` 0.4.0.
- Generated code must pass `cargo clippy -D warnings` and rustfmt, and its
  public API is public API: document any change to it in the PR.
