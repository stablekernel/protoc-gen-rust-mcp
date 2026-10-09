//! Parity test: for every RPC in `examples/protos/example.proto`, the
//! description `comments::process_comment_to_string` (falling back to
//! `comments::camel_to_space`) produces from the method's leading comment
//! must equal the `Description` Go's committed
//! `examples/gen/example/v1/example_mcp.pb.go` generates for the same
//! method (`protoc-gen-go-mcp` v0.3.0, commit `9e072f9`). This exercises
//! the real `SourceCodeInfo` protoc produces for the shared example proto,
//! not hand-built fixtures, via `protox` (so `cargo test` doesn't need a
//! `protoc` binary or network access).

use protoc_gen_rust_mcp::comments::{camel_to_space, process_comment_to_string};
use protoc_gen_rust_mcp::source_info::{file_path, leading_comments, service_path};

const EXAMPLE_PROTO_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../examples/protos");

/// The method descriptions Go's committed `example_mcp.pb.go` generates,
/// in service-declaration order (`VibeService`'s methods in
/// `example.proto`).
const GO_DESCRIPTIONS: &[(&str, &str)] = &[
    (
        "SetVibe",
        "This is a block comment with multiple lines to test block handling \"Hello World\", a `backtick`, and a path like C:\\vibes\\new",
    ),
    ("GetVibe", "Get Vibe of the server"),
    ("SetVibeDetails", "Set vibe details"),
    ("SetVibeArray", "Set the vibe arrays"),
    ("SetVibeObjects", "Set multiple vibe objects"),
];

fn method_description(leading: &str, go_name: &str) -> String {
    if leading.is_empty() {
        camel_to_space(go_name)
    } else {
        process_comment_to_string(leading)
    }
}

#[test]
fn method_descriptions_match_go_committed_output() {
    let file_descriptor_set =
        protox::compile(["example.proto"], [EXAMPLE_PROTO_DIR]).expect("compiling example.proto");
    let file = file_descriptor_set
        .file
        .iter()
        .find(|f| f.name.as_deref() == Some("example.proto"))
        .expect("example.proto in the FileDescriptorSet");
    let service = file
        .service
        .first()
        .expect("VibeService is the only service in example.proto");
    assert_eq!(service.name.as_deref(), Some("VibeService"));

    assert_eq!(
        service.method.len(),
        GO_DESCRIPTIONS.len(),
        "example.proto's method count drifted from the Go parity fixture; update GO_DESCRIPTIONS"
    );

    for (index, method) in service.method.iter().enumerate() {
        let method_name = method.name.as_deref().unwrap_or_default();
        let (expected_name, expected_description) = GO_DESCRIPTIONS[index];
        assert_eq!(
            method_name, expected_name,
            "method order drifted from the Go parity fixture"
        );

        // [FileDescriptorProto.service, 0 (VibeService is the file's only
        // service), ServiceDescriptorProto.method, index]: see
        // `source_info`'s doc comment for how a SourceCodeInfo path is
        // built from the root FileDescriptorProto down to a declaration.
        let path = [file_path::SERVICE, 0, service_path::METHOD, index as i32];
        let leading = leading_comments(file.source_code_info.as_ref(), &path);
        let description = method_description(leading, method_name);

        assert_eq!(
            description, expected_description,
            "description for {method_name} did not match Go's committed output"
        );
    }
}
