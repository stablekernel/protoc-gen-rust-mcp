//! The plugin's code generation entry point: `generate` turns a
//! `CodeGeneratorRequest` into a `CodeGeneratorResponse`, one generated file
//! per `.proto` file that both is marked for generation (appears in
//! `file_to_generate`) and declares at least one service. Streaming RPCs and
//! files without services are skipped, mirroring the Go plugin
//! (`cmd/protoc-gen-go-mcp/main.go`).

use prost_types::FileDescriptorProto;
use prost_types::compiler::CodeGeneratorRequest;
use prost_types::compiler::code_generator_response::File;

/// `protobuf/compiler/plugin.proto`'s `CodeGeneratorResponse`, extended with
/// the `minimum_edition` and `maximum_edition` fields (tags 3 and 4) that
/// `prost-types` 0.14 doesn't expose on its own `CodeGeneratorResponse`
/// (it vendors an older copy of `plugin.proto`). The wire format is
/// unchanged, so this still round-trips through `protoc`.
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct CodeGeneratorResponse {
    #[prost(string, optional, tag = "1")]
    pub error: Option<String>,
    #[prost(uint64, optional, tag = "2")]
    pub supported_features: Option<u64>,
    #[prost(int32, optional, tag = "3")]
    pub minimum_edition: Option<i32>,
    #[prost(int32, optional, tag = "4")]
    pub maximum_edition: Option<i32>,
    #[prost(message, repeated, tag = "15")]
    pub file: Vec<File>,
}

/// `CodeGeneratorResponse::Feature` bitmask values
/// (`protobuf/compiler/plugin.proto`).
const FEATURE_PROTO3_OPTIONAL: u64 = 1;
const FEATURE_SUPPORTS_EDITIONS: u64 = 2;

/// `Edition` enum values (`protobuf/descriptor.proto`) this plugin declares
/// support for.
const EDITION_PROTO2: i32 = 998;
const EDITION_2023: i32 = 1000;

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
        supported_features: Some(FEATURE_PROTO3_OPTIONAL | FEATURE_SUPPORTS_EDITIONS),
        minimum_edition: Some(EDITION_PROTO2),
        maximum_edition: Some(EDITION_2023),
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
    fn no_services_generates_no_files_and_sets_features_and_editions() {
        let request = CodeGeneratorRequest {
            file_to_generate: vec!["no_service.proto".to_string()],
            proto_file: vec![file("no_service.proto", vec![])],
            ..Default::default()
        };

        let response = generate(&request);

        assert!(response.file.is_empty());
        assert_eq!(
            response.supported_features,
            Some(FEATURE_PROTO3_OPTIONAL | FEATURE_SUPPORTS_EDITIONS)
        );
        assert_eq!(response.minimum_edition, Some(EDITION_PROTO2));
        assert_eq!(response.maximum_edition, Some(EDITION_2023));
        assert_eq!(response.error, None);
    }

    #[test]
    fn empty_request_generates_no_files() {
        let request = CodeGeneratorRequest::default();
        let response = generate(&request);
        assert!(response.file.is_empty());
        assert_eq!(
            response.supported_features,
            Some(FEATURE_PROTO3_OPTIONAL | FEATURE_SUPPORTS_EDITIONS)
        );
    }
}
