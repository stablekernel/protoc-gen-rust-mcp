//! Compiles `tests/testdata/schemapb/schema.proto` into pbjson-generated
//! Rust types, for `src/schema.rs`'s tests to validate against: the
//! counterpart of Go's `schema_test.go` loading the same proto with
//! `protocompile` and comparing against `protojson`-marshaled messages.
//!
//! Runs unconditionally (not only for `cargo test`) because Cargo does not
//! give build scripts a reliable "are we testing" signal, but it only
//! compiles one small proto file with no network access (`protox` is a
//! pure-Rust compiler and all its imports here are the bundled well-known
//! types), so the extra cost on a plain `cargo build` is negligible.
//!
//! The dependencies used here (`protox`, `prost-build`, `pbjson-build`) are
//! declared under `[build-dependencies]`, not `[dependencies]`: they are
//! not part of the plugin binary itself.

use std::path::PathBuf;

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

    let out_dir = PathBuf::from(std::env::var("OUT_DIR")?);
    let descriptor_bytes = {
        use protox::prost::Message;
        fds.encode_to_vec()
    };

    pbjson_build::Builder::new()
        .register_descriptors(&descriptor_bytes)?
        .build(&[".schemapb"])?;

    // pbjson_build::Builder::build writes <package>.serde.rs next to
    // prost-build's own output in OUT_DIR; nothing further to do here.
    let _ = out_dir;
    Ok(())
}
