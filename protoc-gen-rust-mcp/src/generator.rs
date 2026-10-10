//! The plugin's code generation entry point: `generate` turns a
//! `CodeGeneratorRequest` into a `CodeGeneratorResponse`, one generated file
//! per proto *package* that both is marked for generation (appears in
//! `file_to_generate`) and declares at least one service, following
//! prost's per-package naming (e.g. `examples.v1.mcp.rs`). Streaming RPCs
//! and files without services are skipped, mirroring the Go plugin
//! (`cmd/protoc-gen-go-mcp/main.go`).
//!
//! The generated `<package>.mcp.rs` is written standalone (so `--rust-
//! mcp_out` can target the same directory as `--prost_out` without
//! clobbering its output) *and* an `include!("<package>.mcp.rs");` line is
//! appended to the main `<package>.rs` file at its `module` insertion
//! point, the same convention `protoc-gen-tonic` and
//! `protoc-gen-prost-serde` use to splice their own output into prost's
//! main file. `make generate` runs this plugin after those two, so the
//! insertion point already exists in the file on disk by the time this
//! append lands (protoc applies every generator's insertions to the
//! already-written file regardless of order, per `plugin.proto`'s
//! `insertion_point` doc comment).

use prost_reflect::DescriptorPool;
use prost_types::FileDescriptorProto;
use prost_types::compiler::CodeGeneratorRequest;
use prost_types::compiler::CodeGeneratorResponse;
use prost_types::compiler::Version;
use prost_types::compiler::code_generator_response::File;

use crate::header::file_header;
use crate::server;

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
/// proto package in `file_to_generate` that declares a service (streaming
/// RPCs and files without services are skipped, same as the Go plugin).
/// Factored out of `main` so tests can call it directly on a hand-built
/// `CodeGeneratorRequest`, without needing `protoc` or a
/// `protoc-gen-rust-mcp` binary.
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

    if files_to_generate.is_empty() {
        return CodeGeneratorResponse {
            error: None,
            supported_features: Some(FEATURE_PROTO3_OPTIONAL),
            file: Vec::new(),
        };
    }

    // A DescriptorPool needs every file in the request (not just the ones
    // being generated) so cross-file message references resolve; prost-
    // reflect panics on duplicate files, so include each only once, by
    // name, same as `request.proto_file` itself already is (protoc never
    // repeats a file in one request).
    let file_descriptor_set = prost_types::FileDescriptorSet {
        file: request.proto_file.clone(),
    };
    let pool = DescriptorPool::from_file_descriptor_set(file_descriptor_set)
        .expect("protoc always sends a well-formed, internally consistent FileDescriptorSet");

    // Group the files to generate by package: prost emits one Rust module
    // per package (merging every .proto file that shares one), and this
    // plugin's own output must land in that same module.
    let mut packages: Vec<String> = Vec::new();
    for f in &files_to_generate {
        let package = f.package.clone().unwrap_or_default();
        if !packages.contains(&package) {
            packages.push(package);
        }
    }

    let mut file = Vec::new();
    for package in &packages {
        file.extend(generate_package(
            &pool,
            package,
            request.compiler_version.as_ref(),
        ));
    }

    CodeGeneratorResponse {
        error: None,
        supported_features: Some(FEATURE_PROTO3_OPTIONAL),
        file,
    }
}

/// Generates the standalone `<package>.mcp.rs` file plus the
/// `include!(...)` appended to `<package>.rs`'s `module` insertion point,
/// for every service declared in `package` (across however many of its
/// `.proto` files are in the pool). Returns an empty `Vec` if `package` has
/// no services (shouldn't happen: callers only pass packages whose files
/// matched `!f.service.is_empty()`, but a package can in principle split
/// its services across sibling files).
fn generate_package(
    pool: &DescriptorPool,
    package: &str,
    compiler_version: Option<&Version>,
) -> Vec<File> {
    let services: Vec<_> = pool
        .services()
        .filter(|s| s.package_name() == package)
        .collect();
    if services.is_empty() {
        return Vec::new();
    }

    let output_dir = if package.is_empty() {
        String::new()
    } else {
        package.replace('.', "/") + "/"
    };
    let mcp_filename = format!("{package}.mcp.rs");
    let main_filename = format!("{package}.rs");

    // Every service's generated code (`server::generate_service`'s raw,
    // not-yet-formatted output), concatenated, then run through
    // `prettyplease` *before* the header is prepended: the same order
    // prost-build itself uses for its own `// This file is @generated by
    // prost-build.` banner (`Config::format`/`add_generated_modules`),
    // because `syn::parse_file` would otherwise swallow the header's
    // plain `//` line comments (`syn` only preserves comments that are
    // doc comments or attached to an AST node it round-trips, neither of
    // which a file's very first lines are).
    let mut body = String::new();
    for (i, service) in services.iter().enumerate() {
        if i > 0 {
            body.push('\n');
        }
        body.push_str(&server::generate_service(service));
    }

    // Starts with the same DO-NOT-EDIT banner and versions block every
    // other generated file in this crate starts with (#4's `file_header`),
    // the counterpart of Go's `example_mcp.pb.go` header.
    let mut content = file_header(env!("CARGO_PKG_VERSION"), compiler_version).join("\n");
    content.push_str("\n\n");
    content.push_str(&format_rust(&body));

    vec![
        File {
            name: Some(format!("{output_dir}{mcp_filename}")),
            content: Some(content),
            ..Default::default()
        },
        File {
            name: Some(format!("{output_dir}{main_filename}")),
            insertion_point: Some("module".to_string()),
            content: Some(format!("include!(\"{mcp_filename}\");\n")),
            ..Default::default()
        },
    ]
}

/// Formats a generated Rust source fragment with `prettyplease`, the same
/// formatter `prost-build` and `tonic-build` use for their own output (see
/// `prost_build::Config::format`, which this generator previously had no
/// counterpart for, and `tonic_build::manual::Method`'s codegen): `syn`
/// parses `rust` as a complete file (every item `server::generate_service`
/// emits — `static`s, a type alias, a struct, its two `impl` blocks — is
/// valid at file scope on its own, since the generated `.mcp.rs` is
/// `include!`d directly into the package's module), then `prettyplease`
/// re-renders it at the usual ~100-column width, replacing the
/// hand-written `format!` calls' own ad hoc indentation (#22: those often
/// produced 100+-column lines, like the embedded-schema `static`s' single
/// line each).
///
/// Panics if `rust` does not parse as a `syn::File`: every caller passes
/// this generator's own output, so a parse failure here means a bug in
/// `server::generate_service`, not malformed user input — the same
/// contract `prost_build::Config::format`'s own `syn::parse_file(buf)
/// .unwrap()` relies on.
fn format_rust(rust: &str) -> String {
    let file: syn::File = syn::parse_str(rust)
        .unwrap_or_else(|e| panic!("generated code is not valid Rust: {e}\n\n{rust}"));
    prettyplease::unparse(&file)
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
