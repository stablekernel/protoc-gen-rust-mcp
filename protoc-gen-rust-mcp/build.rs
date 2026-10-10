//! Compiles `tests/testdata/schemapb/schema.proto` into pbjson-generated
//! Rust types, for `src/schema.rs`'s tests to validate against: the
//! counterpart of Go's `schema_test.go` loading the same proto with
//! `protocompile` and comparing against `protojson`-marshaled messages.
//! Also compiles `tests/testdata/mcpgen/fixture.proto` with real
//! `tonic-prost-build`, and runs this crate's own `server::generate_service`
//! against it, so `tests/server_integration.rs` can exercise the generated
//! MCP server code against a real tonic-generated client (`#[path]`-
//! included below, since a build script cannot depend on the library
//! crate it is building).
//!
//! Runs unconditionally (not only for `cargo test`) because Cargo does not
//! give build scripts a reliable "are we testing" signal, but it only
//! compiles two small protos with no network access (`protox` is a
//! pure-Rust compiler and all its imports here are the bundled well-known
//! types), so the extra cost on a plain `cargo build` is negligible.
//!
//! The dependencies used here (`protox`, `prost-build`, `pbjson-build`,
//! `tonic-prost-build`) are declared under `[build-dependencies]`, not
//! `[dependencies]`: they are not part of the plugin binary itself.

use std::path::PathBuf;

// Re-included by #[path] (a build script can't depend on the library
// crate it builds); this standalone compilation only exercises a subset
// of each module's public API, so `#[allow(dead_code)]` the rest instead
// of marking individual unused items in the real `src/` files.
#[allow(dead_code)]
#[path = "src/comments.rs"]
mod comments;
#[allow(dead_code)]
#[path = "src/rust_literal.rs"]
mod rust_literal;
#[path = "src/schema.rs"]
mod schema;
#[allow(dead_code)]
#[path = "src/server.rs"]
mod server;
#[allow(dead_code)]
#[path = "src/source_info.rs"]
mod source_info;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/testdata/schemapb");
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
/// build` and `pbjson-build` (so `tests/server_integration.rs` exercises
/// this generator's output against a real tonic-generated client and
/// pbjson-generated `Serialize`/`Deserialize` impls, not a hand-written
/// stand-in — the generated handler body, #7, decodes/encodes arguments
/// with those impls), then runs `server::generate_service` (this crate's
/// own generator, `#[path]`-included above) against the same descriptors
/// and writes the result as `mcpgen.mcp.rs` into `OUT_DIR`, so the test
/// can `include!` all three files the way `examples/src/gen/` does for
/// the real example.
fn generate_mcpgen_fixture() -> Result<(), Box<dyn std::error::Error>> {
    let proto_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/testdata/mcpgen");
    println!("cargo:rerun-if-changed={}", proto_dir.display());

    let fds = protox::compile(["fixture.proto"], [&proto_dir])?;

    tonic_prost_build::configure().compile_fds(fds.clone())?;

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
        content.push_str(&server::generate_service(&service));
        content.push('\n');
    }

    let out_dir = PathBuf::from(std::env::var("OUT_DIR")?);
    std::fs::write(out_dir.join("mcpgen.mcp.rs"), content)?;

    Ok(())
}
