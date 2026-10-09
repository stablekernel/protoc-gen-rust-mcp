//! The plugin's code generation entry point: `generate` turns a
//! `CodeGeneratorRequest` into a `CodeGeneratorResponse`, one generated file
//! per `.proto` file that both is marked for generation (appears in
//! `file_to_generate`) and declares at least one service. Streaming RPCs and
//! files without services are skipped, mirroring the Go plugin
//! (`cmd/protoc-gen-go-mcp/main.go`).

use prost_types::FileDescriptorProto;
use prost_types::compiler::CodeGeneratorRequest;
use prost_types::compiler::CodeGeneratorResponse;
use prost_types::compiler::code_generator_response::File;

/// `CodeGeneratorResponse::Feature` bitmask values
/// (`protobuf/compiler/plugin.proto`). This plugin does **not** advertise
/// `FEATURE_SUPPORTS_EDITIONS`: `prost-reflect` 0.16.5 only builds
/// `DescriptorPool`s for proto2/proto3 files (it panics on an `edition =
/// "2023";` file's `FileDescriptorProto`, confirmed empirically against a
/// real protoc 29.3 request, at `descriptor/error.rs:782`), and
/// `protoc-gen-prost` 0.5.0 itself only advertises `PROTO3_OPTIONAL`, so this
/// stack cannot generate editions code regardless. Leaving editions
/// unadvertised makes `protoc` reject an editions input file with a clear
/// error instead of this plugin panicking on it. This is a documented
/// divergence from the Go plugin.
const FEATURE_PROTO3_OPTIONAL: u64 = 1;

/// Runs code generation against `request`, producing one generated file per
/// `.proto` file in `file_to_generate` that declares a service (streaming
/// RPCs and files without services are skipped, same as the Go plugin).
/// Factored out of `main` so tests can call it directly on a hand-built
/// `CodeGeneratorRequest`, without needing `protoc` or a
/// `protoc-gen-rust-mcp` binary.
///
/// This scaffold doesn't generate any file content yet: `generate_file`
/// below is a stub for the codegen issues building on it to fill in.
pub fn generate(request: &CodeGeneratorRequest) -> CodeGeneratorResponse {
    let files_to_generate: Vec<&FileDescriptorProto> = request
        .proto_file
        .iter()
        .filter(|f| {
            f.name
                .as_deref()
                .is_some_and(|name| request.file_to_generate.iter().any(|g| g == name))
        })
        .filter(|f| !f.service.is_empty())
        .collect();

    let file = files_to_generate
        .into_iter()
        .filter_map(generate_file)
        .collect();

    CodeGeneratorResponse {
        error: None,
        supported_features: Some(FEATURE_PROTO3_OPTIONAL),
        file,
    }
}

/// Generates the MCP server code for one `.proto` file known to declare a
/// service. Returns `None` for now; later issues give this real content.
fn generate_file(_proto_file: &FileDescriptorProto) -> Option<File> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use prost_types::ServiceDescriptorProto;

    fn file(name: &str, services: Vec<ServiceDescriptorProto>) -> FileDescriptorProto {
        FileDescriptorProto {
            name: Some(name.to_string()),
            service: services,
            ..Default::default()
        }
    }

    #[test]
    fn no_services_generates_no_files_and_sets_features() {
        let request = CodeGeneratorRequest {
            file_to_generate: vec!["no_service.proto".to_string()],
            proto_file: vec![file("no_service.proto", vec![])],
            ..Default::default()
        };

        let response = generate(&request);

        assert!(response.file.is_empty());
        assert_eq!(response.supported_features, Some(FEATURE_PROTO3_OPTIONAL));
        assert_eq!(response.error, None);
    }

    #[test]
    fn empty_request_generates_no_files() {
        let request = CodeGeneratorRequest::default();
        let response = generate(&request);
        assert!(response.file.is_empty());
        assert_eq!(response.supported_features, Some(FEATURE_PROTO3_OPTIONAL));
    }
}
