//! Dev-only test fixtures for `protoc-gen-rust-mcp` (`publish = false`,
//! not a dependency of the plugin itself): compiles
//! `testdata/schemapb/schema.proto` and `testdata/mcpgen/fixture.proto`
//! (via `build.rs`, with `protox`, `pbjson-build` and `tonic-prost-build`)
//! and runs `protoc_gen_rust_mcp::server::generate_service` against the
//! latter, so `protoc-gen-rust-mcp`'s own tests can validate against real
//! pbjson-generated types and a real tonic-generated client/server,
//! without the plugin crate itself needing a `build.rs` or those
//! build-time dependencies (#22).
//!
//! `protoc-gen-rust-mcp` depends on this crate as a dev-dependency; this
//! crate depends on `protoc-gen-rust-mcp` as a normal (build-)dependency
//! to run its generator, which is possible here precisely because this
//! crate is not the one being built when `generate_service` runs.

/// The pbjson-generated Rust types for `testdata/schemapb/schema.proto`,
/// used to validate generated schemas against real pbjson-serialized
/// messages: the counterpart of Go's `schema_test.go` validating against
/// `protojson`-marshaled messages.
#[allow(
    missing_docs,
    clippy::all,
    clippy::pedantic,
    dead_code,
    unreachable_pub
)]
pub mod schemapb {
    include!(concat!(env!("OUT_DIR"), "/schemapb.rs"));
    include!(concat!(env!("OUT_DIR"), "/schemapb.serde.rs"));
}

/// The real tonic-generated client/server code, pbjson-generated
/// `Serialize`/`Deserialize` impls, and this generator's own output for
/// `testdata/mcpgen/fixture.proto` (all three produced by `build.rs`),
/// exercised end-to-end by `protoc-gen-rust-mcp/tests/server_integration.rs`
/// against a real tonic-generated client, the way `examples/` does for
/// the real, committed `VibeService` example.
#[allow(
    dead_code,
    missing_docs,
    clippy::all,
    clippy::pedantic,
    unreachable_pub
)]
pub mod mcpgen {
    include!(concat!(env!("OUT_DIR"), "/mcpgen.rs"));
    include!(concat!(env!("OUT_DIR"), "/mcpgen.serde.rs"));
    include!(concat!(env!("OUT_DIR"), "/mcpgen.mcp.rs"));
}
