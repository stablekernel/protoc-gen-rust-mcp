//! `protoc-gen-rust-mcp --version` prints the crate version and exits
//! without reading stdin, as `protoc` expects of a well-behaved plugin.

use std::process::Command;

#[test]
fn version_flag_prints_version_and_exits() {
    let output = Command::new(env!("CARGO_BIN_EXE_protoc-gen-rust-mcp"))
        .arg("--version")
        .output()
        .expect("failed to run protoc-gen-rust-mcp");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert_eq!(
        stdout.trim(),
        format!("protoc-gen-rust-mcp {}", env!("CARGO_PKG_VERSION"))
    );
}
