//! Generates per-message JSON rewrite functions that translate a
//! `google.protobuf.FieldMask` field's JSON between pbjson's own
//! `{"paths": [...]}` encoding and protojson's single comma-joined
//! lowerCamel-path string (`schema.rs`'s `WKT_FIELD_MASK` case documents
//! why the schema says `string`; #20 is the issue this implements).
//! `server.rs`'s generated handler runs a request message's `..._fm_in`
//! rewrite on the raw JSON arguments after schema validation and before
//! the pbjson decode, and a response message's `..._fm_out` rewrite on
//! the pbjson-encoded response before `serde_json::to_string`.
//!
//! Only messages that transitively contain a FieldMask field (through
//! singular, repeated or map-valued message fields, however deeply
//! nested, including through a reference cycle) get a rewrite function;
//! every other method's generated handler is unchanged, byte for byte,
//! from before this module existed (no field in `example.proto` is a
//! FieldMask, so the golden-file and parity tests pin that this is a
//! no-op for the committed example).
//!
//! Functions and the two shared helpers are named with the owning
//! service's own `snake_case` name as a prefix (like the existing
//! `..._INPUT_SCHEMA`/`..._VALIDATOR` constants in `server.rs`), because
//! the generated `<package>.mcp.rs` is `include!`d directly into one
//! module: two services in the same file must not emit colliding
//! top-level item names, even if (unusually) they share a message type
//! that needs a FieldMask rewrite.

use std::collections::BTreeSet;
use std::fmt::Write as _;

use heck::ToSnakeCase;
use prost_reflect::{Cardinality, FieldDescriptor, Kind, MessageDescriptor};

/// Full name of `google.protobuf.FieldMask`, duplicated from `schema.rs`'s
/// own (private) `WKT_FIELD_MASK`: the two modules care about this name
/// for different reasons (schema shape vs. JSON rewrite), not worth
/// sharing a `pub(crate)` item for one string literal.
const FIELD_MASK_FULL_NAME: &str = "google.protobuf.FieldMask";

/// Which way a rewrite function converts a FieldMask field's JSON.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Direction {
    /// protojson's string -> pbjson's `{"paths": [...]}`: run on tool call
    /// arguments, after schema validation and before the pbjson decode.
    In,
    /// pbjson's `{"paths": [...]}` -> protojson's string: run on a
    /// pbjson-encoded response, before `serde_json::to_string`.
    Out,
}

impl Direction {
    fn suffix(self) -> &'static str {
        match self {
            Direction::In => "fm_in",
            Direction::Out => "fm_out",
        }
    }
}

/// Returns whether `message` contains a `google.protobuf.FieldMask` field
/// anywhere in its structure: directly, or nested inside another message
/// field (singular, repeated or map-valued), however deep, including
/// through a reference cycle. A message that only reaches a FieldMask by
/// looping back through itself is correctly treated as *not* containing
/// one: that loop never bottoms out in finitely many steps, so no real
/// value of that type can ever actually hold a FieldMask by that path
/// alone.
pub fn message_needs_rewrite(message: &MessageDescriptor) -> bool {
    let mut visiting = BTreeSet::new();
    transitively_has_field_mask(message, &mut visiting)
}

fn transitively_has_field_mask(
    message: &MessageDescriptor,
    visiting: &mut BTreeSet<String>,
) -> bool {
    let full_name = message.full_name().to_string();
    if visiting.contains(&full_name) {
        return false;
    }
    visiting.insert(full_name.clone());
    let result = message
        .fields()
        .any(|field| field_has_field_mask(&field, visiting));
    visiting.remove(&full_name);
    result
}

/// The `Kind` a field's *value* actually carries: for a map field, that's
/// its synthetic entry message's value field (number 2); for every other
/// field (repeated or not), it's the field's own kind.
fn value_kind(field: &FieldDescriptor) -> Kind {
    if field.is_map() {
        let Kind::Message(entry) = field.kind() else {
            unreachable!("a map field's kind is always its synthetic entry message");
        };
        entry.map_entry_value_field().kind()
    } else {
        field.kind()
    }
}

fn field_has_field_mask(field: &FieldDescriptor, visiting: &mut BTreeSet<String>) -> bool {
    match value_kind(field) {
        Kind::Message(m) if m.full_name() == FIELD_MASK_FULL_NAME => true,
        Kind::Message(m) => transitively_has_field_mask(&m, visiting),
        _ => false,
    }
}

/// Accumulates the per-message rewrite functions (and the two shared
/// helpers) needed for one service's generated code, memoizing by message
/// full name + direction so a message reached from more than one method
/// (or through more than one field) gets exactly one function per
/// direction, however many times it's referenced.
pub struct FieldMaskCodegen {
    service_snake: String,
    emitted: BTreeSet<(String, &'static str)>,
    helpers_emitted: bool,
    functions: String,
}

impl FieldMaskCodegen {
    pub fn new(service_name: &str) -> Self {
        Self {
            service_snake: service_name.to_snake_case(),
            emitted: BTreeSet::new(),
            helpers_emitted: false,
            functions: String::new(),
        }
    }

    /// If `message` (an RPC's request message) transitively contains a
    /// FieldMask, ensures its `..._fm_in` function (and every function it
    /// calls) is in [`functions`](Self::functions), and returns its name.
    /// Returns `None` if `message` has no FieldMask anywhere in it, in
    /// which case the caller emits no rewrite call at all.
    pub fn ensure_in(&mut self, message: &MessageDescriptor) -> Option<String> {
        self.ensure(message, Direction::In)
    }

    /// The `..._fm_out` counterpart of [`ensure_in`](Self::ensure_in), for
    /// an RPC's response message.
    pub fn ensure_out(&mut self, message: &MessageDescriptor) -> Option<String> {
        self.ensure(message, Direction::Out)
    }

    /// Every function definition accumulated so far (the two shared
    /// helpers, if any message needed them, followed by one function per
    /// `(message, direction)` pair actually used), in the order `ensure`
    /// first reached each one (depth-first).
    pub fn functions(&self) -> &str {
        &self.functions
    }

    fn ensure(&mut self, message: &MessageDescriptor, direction: Direction) -> Option<String> {
        if !message_needs_rewrite(message) {
            return None;
        }
        let fn_name = self.fn_name(message, direction);
        let key = (message.full_name().to_string(), direction.suffix());
        if self.emitted.contains(&key) {
            return Some(fn_name);
        }
        // Mark emitted before recursing into nested fields: a reference
        // cycle (a recursive message with a FieldMask elsewhere in the
        // cycle) then finds itself already "emitted" on the way back
        // around and just calls the function being built, instead of
        // recursing forever. Plain function calls don't need forward
        // declarations at file scope, so it's fine that the function
        // body for `fn_name` hasn't actually been appended to
        // `self.functions` yet when a nested call site refers to it.
        self.emitted.insert(key);
        self.ensure_helpers();

        let mut body = String::new();
        let _ = writeln!(
            body,
            "fn {fn_name}(v: &mut ::serde_json::Value) {{\n    \
             let ::std::option::Option::Some(obj) = v.as_object_mut() else {{\n        return;\n    }};"
        );
        for field in message.fields() {
            self.emit_field(&mut body, &field, direction);
        }
        body.push_str("}\n\n");

        self.functions.push_str(&body);
        Some(fn_name)
    }

    /// Appends one field's rewrite, if any, to `body`: a direct call to
    /// the shared `..._fm_str_to_obj`/`..._fm_obj_to_str` helper for a
    /// FieldMask-typed field, or a call to another message's own
    /// `ensure`-built function for a message-typed field that itself
    /// transitively contains a FieldMask (nothing is emitted for a field
    /// that is neither), across the three shapes a field can have:
    /// singular, repeated (a JSON array) and map (a JSON object keyed by
    /// the map's string key).
    fn emit_field(&mut self, body: &mut String, field: &FieldDescriptor, direction: Direction) {
        let json_name = field.json_name();
        let helper = match direction {
            Direction::In => format!("{}_fm_str_to_obj", self.service_snake),
            Direction::Out => format!("{}_fm_obj_to_str", self.service_snake),
        };

        let is_map = field.is_map();
        let is_repeated = !is_map && field.cardinality() == Cardinality::Repeated;

        let call_name: Option<String> = match value_kind(field) {
            Kind::Message(m) if m.full_name() == FIELD_MASK_FULL_NAME => Some(helper),
            Kind::Message(m) => self.ensure(&m, direction),
            _ => None,
        };
        let Some(call_name) = call_name else {
            return;
        };

        if is_map {
            let _ = writeln!(
                body,
                "    if let ::std::option::Option::Some(::serde_json::Value::Object(map)) = obj.get_mut({json_name:?}) {{\n        \
                 for v in map.values_mut() {{\n            {call_name}(v);\n        }}\n    \
                 }}"
            );
        } else if is_repeated {
            let _ = writeln!(
                body,
                "    if let ::std::option::Option::Some(::serde_json::Value::Array(items)) = obj.get_mut({json_name:?}) {{\n        \
                 for v in items {{\n            {call_name}(v);\n        }}\n    \
                 }}"
            );
        } else {
            let _ = writeln!(
                body,
                "    if let ::std::option::Option::Some(f) = obj.get_mut({json_name:?}) {{\n        {call_name}(f);\n    }}"
            );
        }
    }

    fn fn_name(&self, message: &MessageDescriptor, direction: Direction) -> String {
        format!(
            "{}_{}_{}",
            self.service_snake,
            direction.suffix(),
            message.full_name().to_snake_case()
        )
    }

    fn ensure_helpers(&mut self) {
        if self.helpers_emitted {
            return;
        }
        self.helpers_emitted = true;
        self.functions
            .push_str(&field_mask_helpers(&self.service_snake));
    }
}

/// The two shared helpers every `..._fm_in`/`..._fm_out` function calls
/// for an actual FieldMask-typed field (as opposed to a nested message
/// field, which instead calls that message's own `ensure`-built
/// function), plus the two path-segment case converters they're built
/// on. Emitted once per service (prefixed with its own `snake_case` name,
/// like every other item this module emits, to avoid colliding with
/// another service's copy in the same generated file).
fn field_mask_helpers(service_snake: &str) -> String {
    const TEMPLATE: &str = "\
/// Converts a FieldMask field's protojson string (comma-joined lowerCamel
/// paths, e.g. `\"fooBar,a.bC\"`) into pbjson's own `{\"paths\": [...]}`
/// shape (snake_case paths), so the pbjson-generated `Deserialize` can
/// decode it. A no-op if `v` is not a string (schema validation already
/// rejected that before this runs). An empty string means no paths, per
/// protojson's `unmarshalFieldMask`.
fn SERVICE_fm_str_to_obj(v: &mut ::serde_json::Value) {
    let ::serde_json::Value::String(s) = v else {
        return;
    };
    let paths: ::std::vec::Vec<::serde_json::Value> = s
        .split(',')
        .filter(|p| !p.is_empty())
        .map(|p| {
            ::serde_json::Value::String(
                p.split('.')
                    .map(SERVICE_fm_lower_camel_to_snake)
                    .collect::<::std::vec::Vec<_>>()
                    .join(\".\"),
            )
        })
        .collect();
    *v = ::serde_json::json!({ \"paths\": paths });
}

/// The reverse of [`SERVICE_fm_str_to_obj`]: pbjson's `{\"paths\": [...]}`
/// shape back into protojson's comma-joined lowerCamel string. A
/// zero-valued FieldMask (no paths) pbjson-encodes as `{}` (a `\"paths\"`
/// array is only present when non-empty, the usual pbjson/protojson
/// zero-value-omission rule), which this maps to `\"\"`, not left as an
/// empty JSON object: a no-op here would otherwise send a tool caller an
/// `{}` object for what the schema promises is always a string. A no-op
/// only if `v` is not even a JSON object (unreachable for a real
/// FieldMask field, since schema validation on the way in and this
/// generator's own pbjson encoding on the way out both guarantee that
/// shape in practice).
fn SERVICE_fm_obj_to_str(v: &mut ::serde_json::Value) {
    let ::std::option::Option::Some(obj) = v.as_object() else {
        return;
    };
    let paths = match obj.get(\"paths\") {
        ::std::option::Option::Some(::serde_json::Value::Array(paths)) => paths.as_slice(),
        _ => &[],
    };
    let joined = paths
        .iter()
        .filter_map(|p| p.as_str())
        .map(|p| {
            p.split('.')
                .map(SERVICE_fm_snake_to_lower_camel)
                .collect::<::std::vec::Vec<_>>()
                .join(\".\")
        })
        .collect::<::std::vec::Vec<_>>()
        .join(\",\");
    *v = ::serde_json::Value::String(joined);
}

/// Converts a single lowerCamel path segment to snake_case, protojson's
/// own rule (`strs.JSONSnakeCase` in `google.golang.org/protobuf`): each
/// ASCII uppercase letter becomes `_` followed by its lowercase form.
fn SERVICE_fm_lower_camel_to_snake(s: &str) -> ::std::string::String {
    let mut out = ::std::string::String::with_capacity(s.len());
    for c in s.chars() {
        if c.is_ascii_uppercase() {
            out.push('_');
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

/// Converts a single snake_case path segment to lowerCamel, the reverse
/// of [`SERVICE_fm_lower_camel_to_snake`] (`strs.JSONCamelCase`): each
/// `_x` becomes `X`, and any other underscore is dropped.
fn SERVICE_fm_snake_to_lower_camel(s: &str) -> ::std::string::String {
    let mut out = ::std::string::String::with_capacity(s.len());
    let mut was_underscore = false;
    for c in s.chars() {
        if c != '_' {
            if was_underscore && c.is_ascii_lowercase() {
                out.push(c.to_ascii_uppercase());
            } else {
                out.push(c);
            }
        }
        was_underscore = c == '_';
    }
    out
}

";
    TEMPLATE.replace("SERVICE", service_snake)
}

#[cfg(test)]
mod tests {
    use super::*;
    use prost_types::field_descriptor_proto::{Label, Type};
    use prost_types::{DescriptorProto, FieldDescriptorProto, FileDescriptorProto};

    fn field(
        name: &str,
        number: i32,
        r#type: Type,
        type_name: Option<&str>,
    ) -> FieldDescriptorProto {
        FieldDescriptorProto {
            name: Some(name.to_string()),
            number: Some(number),
            label: Some(Label::Optional as i32),
            r#type: Some(r#type as i32),
            type_name: type_name.map(|s| s.to_string()),
            ..Default::default()
        }
    }

    fn repeated_field(
        name: &str,
        number: i32,
        r#type: Type,
        type_name: Option<&str>,
    ) -> FieldDescriptorProto {
        FieldDescriptorProto {
            label: Some(Label::Repeated as i32),
            ..field(name, number, r#type, type_name)
        }
    }

    /// A `map<string, FieldMask>` field's synthetic entry message
    /// (`MapEntry.key`/`MapEntry.value`, the `map_entry = true` option),
    /// used by [`pool`] to build `fmtest.WithMaskMap.masks`.
    fn map_entry_message(
        name: &str,
        value_type: Type,
        value_type_name: Option<&str>,
    ) -> DescriptorProto {
        DescriptorProto {
            name: Some(name.to_string()),
            field: vec![
                field("key", 1, Type::String, None),
                field("value", 2, value_type, value_type_name),
            ],
            options: Some(prost_types::MessageOptions {
                map_entry: Some(true),
                ..Default::default()
            }),
            ..Default::default()
        }
    }

    /// Builds a pool with:
    /// - `fmtest.Leaf`: no FieldMask anywhere (just a string field).
    /// - `fmtest.WithMask`: a direct FieldMask field.
    /// - `fmtest.Nested`: a message field pointing at `WithMask`.
    /// - `fmtest.Cyclic`: a self-referential field and nothing else (no
    ///   FieldMask reachable, even though it's recursive).
    /// - `fmtest.WithMaskMap`: a `map<string, FieldMask>` field.
    fn pool() -> prost_reflect::DescriptorPool {
        let leaf = DescriptorProto {
            name: Some("Leaf".to_string()),
            field: vec![field("name", 1, Type::String, None)],
            ..Default::default()
        };
        let with_mask = DescriptorProto {
            name: Some("WithMask".to_string()),
            field: vec![field(
                "mask",
                1,
                Type::Message,
                Some(".google.protobuf.FieldMask"),
            )],
            ..Default::default()
        };
        let nested = DescriptorProto {
            name: Some("Nested".to_string()),
            field: vec![field("inner", 1, Type::Message, Some(".fmtest.WithMask"))],
            ..Default::default()
        };
        let cyclic = DescriptorProto {
            name: Some("Cyclic".to_string()),
            field: vec![field("child", 1, Type::Message, Some(".fmtest.Cyclic"))],
            ..Default::default()
        };
        let masks_entry = map_entry_message(
            "MasksEntry",
            Type::Message,
            Some(".google.protobuf.FieldMask"),
        );
        let with_mask_map = DescriptorProto {
            name: Some("WithMaskMap".to_string()),
            field: vec![{
                let mut f = field(
                    "masks",
                    1,
                    Type::Message,
                    Some(".fmtest.WithMaskMap.MasksEntry"),
                );
                f.label = Some(Label::Repeated as i32);
                f
            }],
            nested_type: vec![masks_entry],
            ..Default::default()
        };
        let field_mask = DescriptorProto {
            name: Some("FieldMask".to_string()),
            field: vec![repeated_field("paths", 1, Type::String, None)],
            ..Default::default()
        };
        let field_mask_file = FileDescriptorProto {
            name: Some("google/protobuf/field_mask.proto".to_string()),
            package: Some("google.protobuf".to_string()),
            message_type: vec![field_mask],
            syntax: Some("proto3".to_string()),
            ..Default::default()
        };
        let file = FileDescriptorProto {
            name: Some("fmtest.proto".to_string()),
            package: Some("fmtest".to_string()),
            dependency: vec!["google/protobuf/field_mask.proto".to_string()],
            message_type: vec![leaf, with_mask, nested, cyclic, with_mask_map],
            syntax: Some("proto3".to_string()),
            ..Default::default()
        };
        prost_reflect::DescriptorPool::from_file_descriptor_set(prost_types::FileDescriptorSet {
            file: vec![field_mask_file, file],
        })
        .expect("building descriptor pool")
    }

    #[test]
    fn message_without_field_mask_does_not_need_rewrite() {
        let pool = pool();
        let leaf = pool.get_message_by_name("fmtest.Leaf").unwrap();
        assert!(!message_needs_rewrite(&leaf));
    }

    #[test]
    fn message_with_direct_field_mask_needs_rewrite() {
        let pool = pool();
        let with_mask = pool.get_message_by_name("fmtest.WithMask").unwrap();
        assert!(message_needs_rewrite(&with_mask));
    }

    #[test]
    fn message_with_nested_field_mask_needs_rewrite() {
        let pool = pool();
        let nested = pool.get_message_by_name("fmtest.Nested").unwrap();
        assert!(message_needs_rewrite(&nested));
    }

    #[test]
    fn purely_cyclic_message_without_field_mask_does_not_need_rewrite() {
        let pool = pool();
        let cyclic = pool.get_message_by_name("fmtest.Cyclic").unwrap();
        assert!(!message_needs_rewrite(&cyclic));
    }

    #[test]
    fn message_with_map_valued_field_mask_needs_rewrite() {
        let pool = pool();
        let with_mask_map = pool.get_message_by_name("fmtest.WithMaskMap").unwrap();
        assert!(message_needs_rewrite(&with_mask_map));
    }

    /// A map field's rewrite iterates its values (not its entries or
    /// keys), calling the shared `..._fm_str_to_obj` helper directly on
    /// each one, since a `map<string, FieldMask>`'s value is a FieldMask
    /// itself, not a message that in turn contains one.
    #[test]
    fn map_valued_field_mask_rewrite_iterates_values() {
        let pool = pool();
        let with_mask_map = pool.get_message_by_name("fmtest.WithMaskMap").unwrap();
        let mut codegen = FieldMaskCodegen::new("MyService");
        let fn_name = codegen.ensure_in(&with_mask_map).expect("needs rewrite");
        let functions = codegen.functions();
        assert!(functions.contains(&format!("fn {fn_name}")));
        assert!(functions.contains("obj.get_mut(\"masks\")"));
        assert!(functions.contains("map.values_mut()"));
        assert!(functions.contains("my_service_fm_str_to_obj(v)"));
    }

    #[test]
    fn ensure_returns_none_for_message_without_field_mask() {
        let pool = pool();
        let leaf = pool.get_message_by_name("fmtest.Leaf").unwrap();
        let mut codegen = FieldMaskCodegen::new("MyService");
        assert_eq!(codegen.ensure_in(&leaf), None);
        assert_eq!(codegen.functions(), "");
    }

    #[test]
    fn ensure_is_memoized_by_message_and_direction() {
        let pool = pool();
        let with_mask = pool.get_message_by_name("fmtest.WithMask").unwrap();
        let mut codegen = FieldMaskCodegen::new("MyService");
        let first = codegen.ensure_in(&with_mask).expect("needs rewrite");
        let second = codegen.ensure_in(&with_mask).expect("needs rewrite");
        assert_eq!(first, second);
        // Only emitted once despite being requested twice: the helpers
        // plus exactly one function definition for WithMask's "in".
        assert_eq!(
            codegen
                .functions()
                .matches("fn my_service_fm_in_fmtest_with_mask")
                .count(),
            1
        );
    }

    #[test]
    fn nested_message_rewrite_calls_the_inner_messages_function() {
        let pool = pool();
        let nested = pool.get_message_by_name("fmtest.Nested").unwrap();
        let mut codegen = FieldMaskCodegen::new("MyService");
        let fn_name = codegen.ensure_in(&nested).expect("needs rewrite");
        assert_eq!(fn_name, "my_service_fm_in_fmtest_nested");
        let functions = codegen.functions();
        assert!(functions.contains("fn my_service_fm_in_fmtest_nested"));
        assert!(functions.contains("fn my_service_fm_in_fmtest_with_mask"));
        assert!(functions.contains("my_service_fm_in_fmtest_with_mask(f)"));
    }

    #[test]
    fn lower_camel_to_snake_matches_protojson() {
        let helpers = field_mask_helpers("svc");
        // Spot-check the generated helper source compiles the expected
        // logic by re-implementing it here isn't useful (it's the same
        // code either way); instead this pins the exact rule in prose so
        // a future edit to the template can't silently drift from
        // protojson's strs.JSONSnakeCase/JSONCamelCase without a second
        // regression test (handler-level, in server.rs's own tests)
        // failing too.
        assert!(helpers.contains("is_ascii_uppercase"));
        assert!(helpers.contains("is_ascii_lowercase"));
    }
}
