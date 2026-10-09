//! Looking up a declaration's leading comment (and, for the file header,
//! its leading *detached* comments) in a `FileDescriptorProto`'s
//! `SourceCodeInfo` by path, the counterpart of Go's
//! `protoreflect.SourceLocations.ByPath` /
//! `FileDescriptor.SourceLocations().ByDescriptor(...)`
//! (`cmd/protoc-gen-go-mcp/mcp.go`'s `genLeadingComments`, and
//! `protogen.go`'s `newMethod`/`newMessage`/etc., at commit `9e072f9`).
//!
//! A `SourceCodeInfo.Location`'s `path` is a sequence of field numbers and
//! repeated-field indexes from the root `FileDescriptorProto` down to the
//! declaration (see the doc comment on `descriptor.proto`'s
//! `SourceCodeInfo` for the full worked example); these constants and
//! helpers build that path for the declarations this plugin cares about
//! (the file's syntax/package for the header, and services, methods,
//! messages, fields and enums for descriptions).

use prost_types::SourceCodeInfo;
use prost_types::source_code_info::Location;

/// `FileDescriptorProto` field numbers used to build a `SourceCodeInfo`
/// path (`descriptor.proto`).
pub mod file_path {
    /// `FileDescriptorProto.package`
    pub const PACKAGE: i32 = 2;
    /// `FileDescriptorProto.message_type`
    pub const MESSAGE_TYPE: i32 = 4;
    /// `FileDescriptorProto.enum_type`
    pub const ENUM_TYPE: i32 = 5;
    /// `FileDescriptorProto.service`
    pub const SERVICE: i32 = 6;
    /// `FileDescriptorProto.syntax`
    pub const SYNTAX: i32 = 12;
}

/// `DescriptorProto` (message) field numbers used to build a
/// `SourceCodeInfo` path.
pub mod message_path {
    /// `DescriptorProto.field`
    pub const FIELD: i32 = 2;
    /// `DescriptorProto.nested_type`
    pub const NESTED_TYPE: i32 = 3;
    /// `DescriptorProto.enum_type`
    pub const ENUM_TYPE: i32 = 4;
}

/// `ServiceDescriptorProto` field numbers used to build a `SourceCodeInfo`
/// path.
pub mod service_path {
    /// `ServiceDescriptorProto.method`
    pub const METHOD: i32 = 2;
}

/// `EnumDescriptorProto` field numbers used to build a `SourceCodeInfo`
/// path.
pub mod enum_path {
    /// `EnumDescriptorProto.value`
    pub const VALUE: i32 = 2;
}

/// Finds the `Location` in `source_code_info` whose `path` is exactly
/// `path`, the counterpart of Go's `SourceLocations.ByPath`. Returns `None`
/// if `source_code_info` is absent (protoc only omits it when run without
/// `--include_source_info`) or has no location for that exact path.
pub fn location_by_path<'a>(
    source_code_info: Option<&'a SourceCodeInfo>,
    path: &[i32],
) -> Option<&'a Location> {
    source_code_info?
        .location
        .iter()
        .find(|loc| loc.path == path)
}

/// The leading comment text for the declaration at `path`, or `""` if
/// there is none (no comment, or no `SourceCodeInfo` at all because protoc
/// ran without `--include_source_info`).
pub fn leading_comments<'a>(source_code_info: Option<&'a SourceCodeInfo>, path: &[i32]) -> &'a str {
    location_by_path(source_code_info, path)
        .and_then(|loc| loc.leading_comments.as_deref())
        .unwrap_or("")
}

/// The leading *detached* comment paragraphs for the declaration at
/// `path`: comment paragraphs that appear before the declaration but are
/// separated from it (and from each other) by a blank line, used for the
/// file header's comment above `syntax = "proto3";` (Go's
/// `genLeadingComments`, which prints each detached paragraph followed by
/// the leading comment itself).
pub fn leading_detached_comments<'a>(
    source_code_info: Option<&'a SourceCodeInfo>,
    path: &[i32],
) -> &'a [String] {
    location_by_path(source_code_info, path)
        .map(|loc| loc.leading_detached_comments.as_slice())
        .unwrap_or(&[])
}

#[cfg(test)]
mod tests {
    use super::*;
    use prost_types::source_code_info::Location;

    fn loc(path: Vec<i32>, leading: Option<&str>) -> Location {
        Location {
            path,
            span: vec![],
            leading_comments: leading.map(str::to_string),
            trailing_comments: None,
            leading_detached_comments: vec![],
        }
    }

    #[test]
    fn finds_leading_comment_by_exact_path() {
        let info = SourceCodeInfo {
            location: vec![
                loc(vec![4, 0], Some(" message comment\n")),
                loc(vec![6, 0, 2, 1], Some(" method comment\n")),
            ],
        };
        assert_eq!(
            leading_comments(Some(&info), &[6, 0, 2, 1]),
            " method comment\n"
        );
        assert_eq!(leading_comments(Some(&info), &[4, 0]), " message comment\n");
    }

    #[test]
    fn missing_path_has_no_comment() {
        let info = SourceCodeInfo {
            location: vec![loc(vec![4, 0], Some(" message comment\n"))],
        };
        assert_eq!(leading_comments(Some(&info), &[6, 0, 2, 0]), "");
    }

    #[test]
    fn no_source_code_info_has_no_comment() {
        assert_eq!(leading_comments(None, &[4, 0]), "");
    }

    #[test]
    fn leading_detached_comments_for_file_header() {
        let mut syntax_loc = loc(vec![file_path::SYNTAX], None);
        syntax_loc.leading_detached_comments = vec![" header paragraph one\n".to_string()];
        let info = SourceCodeInfo {
            location: vec![syntax_loc],
        };
        assert_eq!(
            leading_detached_comments(Some(&info), &[file_path::SYNTAX]),
            [" header paragraph one\n".to_string()]
        );
    }
}
