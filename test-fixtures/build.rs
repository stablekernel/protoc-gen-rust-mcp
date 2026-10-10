//! Compiles `tests/testdata/schemapb/schema.proto` into pbjson-generated
//! Rust types, for `protoc_gen_rust_mcp::schema`'s tests to validate
//! against: the counterpart of Go's `schema_test.go` loading the same
//! proto with `protocompile` and comparing against `protojson`-marshaled
//! messages. Also compiles `tests/testdata/mcpgen/fixture.proto` with real
//! `tonic-prost-build`, and runs `protoc_gen_rust_mcp::server::generate_service`
//! against it, so `protoc-gen-rust-mcp/tests/server_integration.rs` can
//! exercise the generated MCP server code against a real tonic-generated
//! client.
//!
//! This crate exists so `protoc-gen-rust-mcp` itself has no build.rs: a
//! plain `cargo build`/`cargo install` of the plugin no longer pulls in
//! `protox`, `prost-build`, `pbjson-build` or `tonic-prost-build`, or
//! compiles these fixtures at all (#22). Unlike the old
//! `protoc-gen-rust-mcp/build.rs`, this one can depend on
//! `protoc-gen-rust-mcp` as a normal (build-)dependency instead of
//! `#[path]`-including its modules, because this crate is not the one
//! being built.
//!
//! Runs unconditionally (not only for `cargo test`) because Cargo does not
//! give build scripts a reliable "are we testing" signal, but it only
//! compiles two small protos with no network access (`protox` is a
//! pure-Rust compiler and all its imports here are the bundled well-known
//! types), so the cost is negligible, and this whole crate only runs for
//! `cargo test --workspace` / `cargo test -p protoc-gen-rust-mcp` anyway
//! (nothing outside tests depends on it).

use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata/schemapb");
    println!("cargo:rerun-if-changed={}", proto_dir.display());

    let fds = protox::compile(["schema.proto"], [&proto_dir])?;

    let mut config = prost_build::Config::new();
    // Route google.protobuf well-known types to pbjson-types (which has
    // the Serialize/Deserialize impls the schema tests need) instead of
    // prost-build's default of prost-types (which has none).
    config.compile_well_known_types();
    config.extern_path(".google.protobuf", "::pbjson_types");
    config.compile_fds(fds.clone())?;

    let descriptor_bytes = {
        use protox::prost::Message;
        fds.encode_to_vec()
    };

    pbjson_build::Builder::new()
        .register_descriptors(&descriptor_bytes)?
        .build(&[".schemapb"])?;

    // pbjson_build::Builder::build writes <package>.serde.rs next to
    // prost-build's own output in OUT_DIR; nothing further to do here.

    generate_mcpgen_fixture()?;

    Ok(())
}

/// Compiles `tests/testdata/mcpgen/fixture.proto` with real `tonic-prost-
/// build` and `pbjson-build` (so `server_integration.rs` exercises this
/// generator's output against a real tonic-generated client and
/// pbjson-generated `Serialize`/`Deserialize` impls, not a hand-written
/// stand-in — the generated handler body, #7, decodes/encodes arguments
/// with those impls), then runs `protoc_gen_rust_mcp::server::generate_service`
/// (a normal dependency; see this file's module doc comment) against the
/// same descriptors and writes the result as `mcpgen.mcp.rs` into
/// `OUT_DIR`, so the test can `include!` all three files the way
/// `examples/src/gen/` does for the real example.
fn generate_mcpgen_fixture() -> Result<(), Box<dyn std::error::Error>> {
    let proto_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata/mcpgen");
    println!("cargo:rerun-if-changed={}", proto_dir.display());

    let fds = protox::compile(["fixture.proto"], [&proto_dir])?;

    // Route google.protobuf well-known types (fixture.proto imports
    // FieldMask, for #20's UpdateVibe RPC) to pbjson-types, same as
    // schema.proto's own config above: tonic-prost-build's default of
    // prost-types has no Serialize/Deserialize impls, which
    // pbjson_build::Builder::build below needs to generate FieldMask's
    // own (unused directly, since #20 rewrites its JSON before pbjson
    // ever sees it) serde impl, and which this fixture's generated
    // mcpgen.mcp.rs needs for UpdateVibeRequest/Response to compile at
    // all (prost-types has no Serialize/Deserialize either way).
    tonic_prost_build::configure()
        .compile_well_known_types(true)
        .extern_path(".google.protobuf", "::pbjson_types")
        .compile_fds(fds.clone())?;

    let descriptor_bytes = {
        use protox::prost::Message;
        fds.encode_to_vec()
    };
    pbjson_build::Builder::new()
        .register_descriptors(&descriptor_bytes)?
        .build(&[".mcpgen"])?;

    use prost_reflect::DescriptorPool;
    let pool = DescriptorPool::from_file_descriptor_set(fds)?;

    let mut content = String::from("// @generated for tests/server_integration.rs\n");
    for service in pool.services() {
        content.push_str(&protoc_gen_rust_mcp::server::generate_service(&service));
        content.push('\n');
    }

    let out_dir = PathBuf::from(std::env::var("OUT_DIR")?);
    std::fs::write(out_dir.join("mcpgen.mcp.rs"), content)?;

    Ok(())
}
