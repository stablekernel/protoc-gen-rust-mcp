//! Golden-file test for the plugin's generated code, the counterpart of
//! Go's `cmd/protoc-gen-go-mcp/golden_test.go` (commit `9e072f9`).
//!
//! It builds a `CodeGeneratorRequest` for `examples/protos/example.proto`
//! without running `protoc`: [`protox`], a pure-Rust protobuf compiler,
//! parses the proto file and builds its descriptor (with comments, via
//! `SourceCodeInfo`) in-process, so this test needs neither a `protoc`
//! binary nor network access. `generator::generate` then runs against that
//! request exactly as it would via `protoc`; its output is already run
//! through `prettyplease` (see `generator.rs`'s `format_rust`), and is
//! then piped through the real `rustfmt` binary too (see
//! [`rustfmt_edition_2024`]), the same final pass `make generate`'s
//! `Makefile` target applies to its `*.mcp.rs` output, because
//! `prettyplease`'s line-wrapping rules don't always agree with
//! `rustfmt`'s own on every construct (#22's acceptance criterion that
//! `rustfmt --check` passes on the committed file). The result is compared
//! byte for byte with the committed
//! `examples/src/gen/examples/v1/examples.v1.mcp.rs`.
//!
//! When a deliberate change to the generator changes the output,
//! regenerate the golden file with:
//!
//! ```sh
//! UPDATE_GOLDEN=1 cargo test -p protoc-gen-rust-mcp golden
//! ```
//!
//! and review the diff; an unexpected change is a bug until explained.
//! This needs `rustfmt` on `PATH`, already required for this repo's own
//! `cargo fmt --all --check` check and pinned as a component in
//! `rust-toolchain.toml`.

use pretty_assertions::assert_eq;
use prost_types::compiler::{CodeGeneratorRequest, Version};

use protoc_gen_rust_mcp::generator;

/// Where `example.proto` (and anything it imports) lives.
const EXAMPLE_PROTO_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../examples/protos");

/// The file to generate, relative to `EXAMPLE_PROTO_DIR`.
const EXAMPLE_PROTO_FILE: &str = "example.proto";

/// The committed golden file this test compares the generator's output
/// against: the same file `make generate` would write for
/// `examples/protos/example.proto`.
const GOLDEN_FILE_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../examples/src/gen/examples/v1/examples.v1.mcp.rs"
);

/// Pins the `protoc` version the generator writes verbatim into the
/// generated file's header comment (the `// - protoc` line; see
/// `header::file_header`). This must match the protoc version recorded in
/// the committed golden file's header for the byte-for-byte comparison to
/// pass; update it (and regenerate, see the module doc comment) if the
/// golden file is ever regenerated with a different protoc version. Must
/// also match `AGENTS.md`'s documented `make generate` protoc version.
const COMPILER_VERSION: Version = Version {
    major: Some(5),
    minor: Some(29),
    patch: Some(3),
    suffix: None,
};

#[test]
fn generated_output_matches_committed_golden_file() {
    let file_descriptor_set = protox::compile([EXAMPLE_PROTO_FILE], [EXAMPLE_PROTO_DIR])
        .expect("compiling example.proto");
    assert!(
        file_descriptor_set
            .file
            .iter()
            .find(|f| f.name.as_deref() == Some(EXAMPLE_PROTO_FILE))
            .expect("example.proto in the FileDescriptorSet")
            .dependency
            .is_empty(),
        "example.proto has no imports; if this changes, this test's request \
         needs its dependencies' descriptors too (protox::compile already \
         includes them via include_imports, so this should stay true)"
    );

    let request = CodeGeneratorRequest {
        file_to_generate: vec![EXAMPLE_PROTO_FILE.to_string()],
        proto_file: file_descriptor_set.file,
        compiler_version: Some(COMPILER_VERSION),
        ..Default::default()
    };

    let response = generator::generate(&request);
    assert_eq!(response.error, None);
    assert_eq!(
        response.file.len(),
        2,
        "expected the standalone examples.v1.mcp.rs file and the \
         insertion-point append to examples.v1.rs"
    );

    let got = response
        .file
        .iter()
        .find(|f| f.name.as_deref() == Some("examples/v1/examples.v1.mcp.rs"))
        .and_then(|f| f.content.as_deref())
        .expect("generator emits examples/v1/examples.v1.mcp.rs");
    let got = rustfmt_edition_2024(got);

    if std::env::var_os("UPDATE_GOLDEN").is_some() {
        std::fs::write(GOLDEN_FILE_PATH, got).expect("writing golden file");
        return;
    }

    let want = std::fs::read_to_string(GOLDEN_FILE_PATH).expect("reading golden file");
    assert_eq!(
        got, want,
        "generated output does not match {GOLDEN_FILE_PATH}; if this change \
         is intentional, run `UPDATE_GOLDEN=1 cargo test -p \
         protoc-gen-rust-mcp golden` and review the diff"
    );
}

/// Pipes `source` through `rustfmt --edition 2024`, the same final
/// formatting pass `make generate`'s `Makefile` target applies to every
/// `*.mcp.rs` file after this plugin runs (see this module's doc
/// comment). Panics if `rustfmt` is missing or rejects `source` (both
/// indicate a bug in the generator or this repo's toolchain setup, not a
/// normal test failure mode).
fn rustfmt_edition_2024(source: &str) -> String {
    use std::io::Write as _;
    use std::process::{Command, Stdio};

    let mut child = Command::new("rustfmt")
        .args(["--edition", "2024", "--emit", "stdout"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawning rustfmt (required on PATH; see rust-toolchain.toml's components)");
    child
        .stdin
        .take()
        .expect("piped stdin")
        .write_all(source.as_bytes())
        .expect("writing to rustfmt's stdin");
    let output = child.wait_with_output().expect("waiting for rustfmt");
    assert!(
        output.status.success(),
        "rustfmt failed on generated code: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("rustfmt output is valid UTF-8")
}
