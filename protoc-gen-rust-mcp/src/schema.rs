//! Builds the JSON Schema (draft 2020-12) for an RPC's request message, used
//! as an MCP tool's `inputSchema`. Ported from Go's
//! `cmd/protoc-gen-go-mcp/schema.go` (`stablekernel/protoc-gen-go-mcp`
//! `9e072f9`); see that file's doc comments for the reasoning behind each
//! rule, reproduced here where it affects this port.
//!
//! The schema describes exactly what the pbjson-generated serde (this
//! repo's counterpart of Go's `protojson`) accepts for the message. The two
//! mappings agree on every rule below, with two deliberate exceptions
//! documented on [`any_schema`] and [`well_known_type_schema`]'s
//! `google.protobuf.FieldMask` case: pbjson does not special-case
//! `Any` or `FieldMask` the way `protojson` does, and instead encodes both
//! as the plain message they are on the wire.
//!
//! A third, narrower difference (documented on [`float_schema`]) is in
//! float/double **output**, not input: pbjson serializes non-finite values
//! as JSON `null` rather than the `"NaN"`/`"Infinity"`/`"-Infinity"` strings
//! `protojson` emits. The schema describes tool *input*, and both pbjson and
//! protojson accept those three strings on deserialization, so the schema
//! itself does not need to change for this one; it is called out because a
//! client reading a float back out of a tool result should not expect those
//! strings.
//!
//! # Design decisions (same as Go, deliberately stricter than every input
//! the generated serde actually tolerates, to keep the schema clear for LLM
//! clients generating arguments from it)
//!
//! - Enum fields only accept the value's name as a string (e.g.
//!   `"COLOR_RED"`), not the underlying `int32` number, even though the
//!   generated deserializer accepts both.
//! - No field is ever typed to accept JSON `null`, even though the
//!   generated deserializer treats an explicit `null` the same as the field
//!   being absent for most field types.
//! - Property names are the field's lowerCamel JSON name
//!   ([`prost_reflect::FieldDescriptor::json_name`]), never the field's
//!   original snake_case proto name, even though the deserializer also
//!   accepts that.
//! - `uint32`/`int32`/`sint32`/`fixed32`/`sfixed32` fields only accept a
//!   JSON number, never the decimal string the deserializer also accepts
//!   for every integer field (not just the 64-bit ones the generated
//!   serializer itself emits as strings).
//!
//! `format` and `contentEncoding` keywords on string schemas (e.g.
//! `"date-time"`, `"base64"`) are descriptive labels only: the `jsonschema`
//! crate, like most JSON Schema implementations, does not validate against
//! them unless formats are explicitly turned on.
//!
//! [`message_input_schema`] is this module's only public entry point.
//! Nothing in `generator.rs` calls it yet: that is wired up by #6 (tool
//! definitions), which embeds the schema this module builds as an MCP
//! tool's `inputSchema`. Allow the otherwise-unused-code warnings for
//! everything below until then rather than marking individual items.
#![allow(dead_code)]

use std::collections::{BTreeSet, HashMap};

use prost_reflect::{Cardinality, FieldDescriptor, Kind, MessageDescriptor};
use serde_json::{Map, Value, json};

use crate::comments::process_comment_to_string;
use crate::source_info::leading_comments;

/// Full names of the well-known proto message types that get special JSON
/// Schema treatment because the generated serde encodes/decodes them
/// differently from an ordinary message with the same fields.
const WKT_TIMESTAMP: &str = "google.protobuf.Timestamp";
const WKT_DURATION: &str = "google.protobuf.Duration";
const WKT_STRUCT: &str = "google.protobuf.Struct";
const WKT_VALUE: &str = "google.protobuf.Value";
const WKT_LIST_VALUE: &str = "google.protobuf.ListValue";
const WKT_FIELD_MASK: &str = "google.protobuf.FieldMask";
const WKT_EMPTY: &str = "google.protobuf.Empty";
const WKT_DOUBLE_VALUE: &str = "google.protobuf.DoubleValue";
const WKT_FLOAT_VALUE: &str = "google.protobuf.FloatValue";
const WKT_INT64_VALUE: &str = "google.protobuf.Int64Value";
const WKT_UINT64_VALUE: &str = "google.protobuf.UInt64Value";
const WKT_INT32_VALUE: &str = "google.protobuf.Int32Value";
const WKT_UINT32_VALUE: &str = "google.protobuf.UInt32Value";
const WKT_BOOL_VALUE: &str = "google.protobuf.BoolValue";
const WKT_STRING_VALUE: &str = "google.protobuf.StringValue";
const WKT_BYTES_VALUE: &str = "google.protobuf.BytesValue";
const WKT_ANY: &str = "google.protobuf.Any";

/// A schema builder turns [`MessageDescriptor`]s into JSON Schemas that
/// describe exactly what the generated pbjson serde accepts for that
/// message. It detects messages that are part of a reference cycle and
/// represents them with `$defs` + `$ref` instead of infinitely inlining
/// them; other message types are inlined at every occurrence, even when
/// referenced more than once.
#[derive(Default)]
struct SchemaBuilder {
    /// The schema for every message that is either recursive or referenced
    /// more than once, keyed by the message's proto full name. Entries are
    /// populated lazily as messages are visited.
    defs: HashMap<String, Value>,
    /// Messages currently being built, to detect recursion.
    building: BTreeSet<String>,
    /// Which message full names ended up needing a `$ref` (i.e. were part
    /// of a reference cycle), so the final schema knows which `$defs`
    /// entries to keep.
    refs: BTreeSet<String>,
}

/// Returns the JSON Schema (draft 2020-12) describing the top-level fields
/// of `message`, suitable as an MCP tool's `inputSchema`. See the module
/// doc comment for the design decisions behind the mapping, and
/// [`SchemaBuilder::message_schema`] for the field-by-field and
/// well-known-type rules.
pub fn message_input_schema(message: &MessageDescriptor) -> Value {
    let mut builder = SchemaBuilder::default();
    let mut root = builder.message_schema(message);
    if builder.refs.is_empty() {
        return root;
    }

    // If `message` is itself part of a reference cycle, `message_schema`
    // returned a bare `{"$ref": ...}` for it instead of an object schema
    // (so that other occurrences of `message` can point at a single
    // `$defs` entry). An MCP tool's input schema must itself have
    // `"type": "object"`, so unwrap that one level here: use a copy of
    // `message`'s own `$defs` entry as the root, instead of a `$ref` to
    // it. A copy (rather than the `$defs` entry itself) is required: the
    // root and the `$defs` entry both need a (different) `"$defs"` key
    // below, and reusing the same value for both would make the `$defs`
    // entry contain itself once `"$defs"` is inserted.
    if root.get("$ref").is_some() {
        root = builder
            .defs
            .get(message.full_name())
            .cloned()
            .unwrap_or_default();
    }

    let mut defs = Map::new();
    for name in &builder.refs {
        if let Some(schema) = builder.defs.get(name) {
            defs.insert(name.clone(), schema.clone());
        }
    }
    root.as_object_mut()
        .expect("root schema is always a JSON object")
        .insert("$defs".to_string(), Value::Object(defs));
    root
}

impl SchemaBuilder {
    /// Returns the schema for `message` itself: an object schema with one
    /// property per top-level field, no `"required"` (proto3 has no
    /// required fields), and `additionalProperties: false`.
    ///
    /// Well-known types that the generated serde encodes specially
    /// (Timestamp, Duration, wrappers, Struct, Value, ListValue, FieldMask,
    /// Empty, Any) are special-cased here before falling back to the
    /// generic message-with-fields handling, because none of them is
    /// represented as a JSON object keyed by field name in the same way
    /// (Any is the one exception noted on [`any_schema`], kept here anyway
    /// for consistency with the other well-known types and in case a
    /// future pbjson release special-cases it too).
    fn message_schema(&mut self, message: &MessageDescriptor) -> Value {
        if let Some(schema) = well_known_type_schema(message.full_name()) {
            return schema;
        }

        let full_name = message.full_name().to_string();
        if self.building.contains(&full_name) {
            // Recursion: refer to the (still being built) $defs entry.
            self.refs.insert(full_name.clone());
            return ref_schema(&full_name);
        }
        self.building.insert(full_name.clone());

        // Synthetic oneofs exist only to implement proto3 "optional"; their
        // field is still emitted as a normal (non-required) property
        // below, so no special handling is needed here beyond skipping the
        // "part of oneof" note for them (done in
        // field_schema_with_description).
        let mut properties = Map::new();
        for field in message.fields() {
            properties.insert(
                field.json_name().to_string(),
                self.field_schema_with_description(&field),
            );
        }

        self.building.remove(&full_name);

        let mut schema = Map::new();
        schema.insert("type".to_string(), json!("object"));
        schema.insert("properties".to_string(), Value::Object(properties));
        schema.insert("additionalProperties".to_string(), json!(false));
        let desc = leading_comment(message.path(), message.parent_file_descriptor_proto());
        if !desc.is_empty() {
            schema.insert("description".to_string(), json!(desc));
        }
        let schema = Value::Object(schema);

        // Remember this message's schema under $defs regardless of whether
        // it turns out to be recursive, so a later recursive reference
        // (reached through some other field) can still find it;
        // message_input_schema only keeps the entries actually marked in
        // self.refs in the final output.
        self.defs.insert(full_name.clone(), schema.clone());

        // If this message turned out to be recursive, the first (and
        // every) occurrence should point at the $defs entry instead of
        // inlining it.
        if self.refs.contains(&full_name) {
            return ref_schema(&full_name);
        }
        schema
    }

    /// Builds the schema for a single field and adds a `"description"` from
    /// its leading proto comment (or a mention of the oneof it belongs to,
    /// if it has no comment of its own), per the issue's requirement that
    /// oneof membership be mentioned in the description.
    fn field_schema_with_description(&mut self, field: &FieldDescriptor) -> Value {
        let mut schema = self.field_schema(field);

        let mut desc = leading_comment(
            field.path(),
            field.parent_message().parent_file_descriptor_proto(),
        );
        if let Some(oneof) = field.containing_oneof()
            && !oneof.is_synthetic()
        {
            let note = format!(
                "Part of the \"{}\" oneof: at most one of its fields may be set.",
                oneof.name()
            );
            desc = if desc.is_empty() {
                note
            } else {
                format!("{desc} {note}")
            };
        }
        if !desc.is_empty() {
            schema
                .as_object_mut()
                .expect("field schema is always a JSON object")
                .insert("description".to_string(), json!(desc));
        }
        schema
    }

    /// Returns the schema for a single field's value, handling repeated
    /// fields, maps, enums, messages, and scalars per the table in the
    /// issue.
    fn field_schema(&mut self, field: &FieldDescriptor) -> Value {
        if field.is_map() {
            // For a map field, `field.kind()` is the synthetic map-entry
            // message; its field number 2 ("value", by the map-entry
            // wire-format convention) is the `FieldDescriptor` that
            // actually carries the map's value type, including, for enum
            // or message values, the full `Kind` that a bare
            // `FieldDescriptor::kind()` on the entry's value field alone
            // would not otherwise provide outside of a map context.
            let Kind::Message(entry) = field.kind() else {
                unreachable!("a map field's kind is always its synthetic entry message");
            };
            let value_field = entry.map_entry_value_field();
            return json!({
                "type": "object",
                "additionalProperties": self.kind_schema(&value_field),
            });
        }

        if field.cardinality() == Cardinality::Repeated {
            return json!({
                "type": "array",
                "items": self.kind_schema(field),
            });
        }

        self.kind_schema(field)
    }

    /// Returns the schema for a single (non-repeated, non-map) value
    /// described by `field`: `field` itself for an ordinary or repeated
    /// field, or the value field obtained from
    /// [`prost_reflect::MessageDescriptor::map_entry_value_field`] for a
    /// map field.
    fn kind_schema(&mut self, field: &FieldDescriptor) -> Value {
        match field.kind() {
            Kind::Bool => json!({"type": "boolean"}),
            Kind::String => json!({"type": "string"}),
            Kind::Bytes => json!({"type": "string", "contentEncoding": "base64"}),
            Kind::Int32 | Kind::Sint32 | Kind::Sfixed32 => json!({"type": "integer"}),
            Kind::Uint32 | Kind::Fixed32 => json!({"type": "integer", "minimum": 0}),
            Kind::Int64 | Kind::Sint64 | Kind::Sfixed64 => {
                json!({"type": ["integer", "string"]})
            }
            Kind::Uint64 | Kind::Fixed64 => {
                json!({"type": ["integer", "string"], "minimum": 0})
            }
            Kind::Float | Kind::Double => float_schema(),
            Kind::Enum(enum_desc) => enum_schema(&enum_desc),
            Kind::Message(msg) => self.message_schema(&msg),
        }
    }
}

/// Returns the schema for an enum: a string restricted to the enum's value
/// names, since the generated serde accepts (and emits) enum values as
/// their name.
fn enum_schema(enum_desc: &prost_reflect::EnumDescriptor) -> Value {
    let values: Vec<Value> = enum_desc
        .values()
        .map(|v| Value::String(v.name().to_string()))
        .collect();
    let mut schema = Map::new();
    schema.insert("type".to_string(), json!("string"));
    schema.insert("enum".to_string(), Value::Array(values));
    let desc = leading_comment(enum_desc.path(), enum_desc.parent_file_descriptor_proto());
    if !desc.is_empty() {
        schema.insert("description".to_string(), json!(desc));
    }
    Value::Object(schema)
}

/// Returns the special-cased schema for one of the well-known proto message
/// types that the generated serde encodes/decodes as something other than
/// a plain JSON object of field names, or `None` if `full_name` is not such
/// a type.
fn well_known_type_schema(full_name: &str) -> Option<Value> {
    Some(match full_name {
        WKT_TIMESTAMP => json!({"type": "string", "format": "date-time"}),
        WKT_DURATION => json!({"type": "string"}),
        WKT_STRUCT => json!({"type": "object"}),
        WKT_VALUE => {
            // google.protobuf.Value can hold any JSON value (null, bool,
            // number, string, array or object); an empty schema accepts
            // anything, which is the only accurate representation.
            json!({})
        }
        WKT_LIST_VALUE => json!({"type": "array"}),
        // **pbjson/protojson difference:** protojson encodes a FieldMask as
        // a single comma-joined string of its paths (e.g. `"a,b.c"`).
        // pbjson-types has no special case for FieldMask (unlike Timestamp,
        // Duration, Struct, Value, ListValue and the wrappers, each of
        // which has a hand-written `Serialize`/`Deserialize` impl in
        // `pbjson-types`' source; see its `build.rs`'s `exclude` list,
        // which FieldMask is conspicuously absent from): the generated
        // serde instead treats it as the plain one-field message it is on
        // the wire, serializing it as `{"paths": ["a", "b.c"]}`.
        // `tests::schema_test_message`'s `updateMask` case pins down the
        // current pbjson behavior this is based on.
        WKT_FIELD_MASK => json!({
            "type": "object",
            "properties": {"paths": {"type": "array", "items": {"type": "string"}}},
            "additionalProperties": false,
        }),
        WKT_EMPTY => json!({"type": "object", "additionalProperties": false}),
        WKT_ANY => any_schema(),
        WKT_DOUBLE_VALUE | WKT_FLOAT_VALUE => float_schema(),
        WKT_INT64_VALUE => json!({"type": ["integer", "string"]}),
        WKT_UINT64_VALUE => json!({"type": ["integer", "string"], "minimum": 0}),
        WKT_INT32_VALUE => json!({"type": "integer"}),
        WKT_UINT32_VALUE => json!({"type": "integer", "minimum": 0}),
        WKT_BOOL_VALUE => json!({"type": "boolean"}),
        WKT_STRING_VALUE => json!({"type": "string"}),
        WKT_BYTES_VALUE => json!({"type": "string", "contentEncoding": "base64"}),
        _ => return None,
    })
}

/// Returns a `{"$ref": "#/$defs/<name>"}` schema pointing at the `$defs`
/// entry for the message with the given full name.
fn ref_schema(full_name: &str) -> Value {
    json!({"$ref": format!("#/$defs/{full_name}")})
}

/// Returns the schema for a proto `float`, `double`, `FloatValue` or
/// `DoubleValue` field.
///
/// This describes *input*: the pbjson-generated deserializer accepts a
/// plain JSON number, or one of the strings `"NaN"`, `"Infinity"` and
/// `"-Infinity"` (a plain JSON number cannot represent those three values),
/// matching the protobuf JSON mapping that `protojson` also implements for
/// decoding. Both are accepted, so the schema is `anyOf` the two.
///
/// **pbjson/protojson difference (output, not input):** on the way back
/// out, pbjson's generated `Serialize` does *not* follow that mapping: like
/// plain `serde_json`/Go's `encoding/json`, it encodes NaN and +/-Infinity
/// as JSON `null` (confirmed empirically: `AllScalars{s_double: NaN,
/// s_float: INFINITY}` serializes to `{"sFloat":null,"sDouble":null}`),
/// which is indistinguishable from the field being absent. `protojson`
/// instead emits the three strings this schema also accepts. This does not
/// change the schema above (it describes input, where both accept the
/// strings), but a client reading a non-finite float out of a tool *result*
/// will see `null`, not one of those strings.
fn float_schema() -> Value {
    json!({
        "anyOf": [
            {"type": "number"},
            {"type": "string", "enum": ["NaN", "Infinity", "-Infinity"]},
        ],
    })
}

/// Returns the schema for `google.protobuf.Any`.
///
/// **pbjson/protojson difference:** `protojson` encodes an `Any` as its
/// unpacked JSON representation plus an injected `"@type"` string field
/// naming the packed message's type. pbjson has no such special case (see
/// `pbjson-types`' build script, which does not exclude
/// `google.protobuf.Any` from the messages it generates ordinary
/// `Serialize`/`Deserialize` impls for, unlike every other well-known type
/// this module special-cases): the generated serde instead treats `Any`
/// as the plain two-field message it is on the wire, serializing it as
/// `{"typeUrl": "<string>", "value": "<base64 bytes>"}`. This schema
/// describes that: both fields are optional (proto3), scalar, named by
/// their JSON names, and `additionalProperties` is `false` like any other
/// ordinary message schema. `tests::any_schema_matches_pbjson_encoding`
/// pins down the current pbjson behavior this is based on.
fn any_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "typeUrl": {"type": "string"},
            "value": {"type": "string", "contentEncoding": "base64"},
        },
        "additionalProperties": false,
    })
}

/// Turns a leading proto comment at `path` within `file` into a one-line
/// description, or `""` if there is no comment. A thin wrapper combining
/// [`crate::source_info::leading_comments`] (the `SourceCodeInfo.location`
/// lookup by path) with [`crate::comments::process_comment_to_string`] (the
/// whitespace/marker cleanup), the same two shared helpers #4 introduced for
/// tool and field descriptions elsewhere in the generator.
fn leading_comment(path: &[i32], file: &prost_types::FileDescriptorProto) -> String {
    let raw = leading_comments(file.source_code_info.as_ref(), path);
    process_comment_to_string(raw)
}

/// The pbjson-generated Rust types for `tests/testdata/schemapb/schema.proto`
/// (compiled by `build.rs`), used by the tests below to validate generated
/// schemas against real pbjson-serialized messages: the counterpart of Go's
/// `schema_test.go` validating against `protojson`-marshaled messages.
#[cfg(test)]
#[allow(
    missing_docs,
    clippy::all,
    clippy::pedantic,
    dead_code,
    unreachable_pub
)]
mod schemapb {
    include!(concat!(env!("OUT_DIR"), "/schemapb.rs"));
    include!(concat!(env!("OUT_DIR"), "/schemapb.serde.rs"));
}

#[cfg(test)]
mod tests {
    use super::schemapb;
    use super::*;
    use prost_reflect::DescriptorPool;
    use serde_json::Map as JsonMap;
    use std::path::PathBuf;

    /// Compiles `cmd/protoc-gen-go-mcp/testdata/schemapb/schema.proto`
    /// (vendored under `tests/testdata/schemapb/`) and returns the
    /// `MessageDescriptor` for `message_name`. Uses `protox`, a pure-Rust
    /// protobuf compiler, so `cargo test` never depends on network access
    /// or an installed `protoc`.
    fn load_message(message_name: &str) -> MessageDescriptor {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/testdata/schemapb");
        let fds = protox::compile(["schema.proto"], [dir]).expect("compiling schema.proto");
        let pool = DescriptorPool::from_file_descriptor_set(fds).expect("building descriptor pool");
        pool.get_message_by_name(&format!("schemapb.{message_name}"))
            .unwrap_or_else(|| panic!("message schemapb.{message_name} not found"))
    }

    fn props(schema: &Value) -> &JsonMap<String, Value> {
        schema["properties"]
            .as_object()
            .expect("schema properties should be an object")
    }

    fn with_description(mut schema: Value, desc: &str) -> Value {
        schema
            .as_object_mut()
            .unwrap()
            .insert("description".to_string(), json!(desc));
        schema
    }

    #[test]
    fn all_scalars() {
        let msg = load_message("AllScalars");
        let schema = message_input_schema(&msg);
        let properties = props(&schema);

        let cases: &[(&str, Value)] = &[
            (
                "sString",
                json!({"type": "string", "description": "A string field."}),
            ),
            (
                "sBytes",
                json!({"type": "string", "contentEncoding": "base64", "description": "A bytes field."}),
            ),
            (
                "sBool",
                json!({"type": "boolean", "description": "A bool field."}),
            ),
            (
                "sInt32",
                json!({"type": "integer", "description": "An int32 field."}),
            ),
            (
                "sSint32",
                json!({"type": "integer", "description": "A sint32 field."}),
            ),
            (
                "sSfixed32",
                json!({"type": "integer", "description": "An sfixed32 field."}),
            ),
            (
                "sUint32",
                json!({"type": "integer", "minimum": 0, "description": "A uint32 field."}),
            ),
            (
                "sFixed32",
                json!({"type": "integer", "minimum": 0, "description": "A fixed32 field."}),
            ),
            (
                "sInt64",
                json!({"type": ["integer", "string"], "description": "An int64 field."}),
            ),
            (
                "sSint64",
                json!({"type": ["integer", "string"], "description": "A sint64 field."}),
            ),
            (
                "sSfixed64",
                json!({"type": ["integer", "string"], "description": "An sfixed64 field."}),
            ),
            (
                "sUint64",
                json!({"type": ["integer", "string"], "minimum": 0, "description": "A uint64 field."}),
            ),
            (
                "sFixed64",
                json!({"type": ["integer", "string"], "minimum": 0, "description": "A fixed64 field."}),
            ),
            ("sFloat", with_description(float_schema(), "A float field.")),
            (
                "sDouble",
                with_description(float_schema(), "A double field."),
            ),
        ];
        for (field, want) in cases {
            assert_eq!(&properties[*field], want, "schema for field {field:?}");
        }

        assert_eq!(schema["type"], json!("object"));
        assert_eq!(schema["additionalProperties"], json!(false));
        assert!(
            schema.get("required").is_none(),
            "proto3 messages must not emit \"required\""
        );
        assert_eq!(
            schema["description"],
            json!("AllScalars covers every scalar proto kind in the schema table."),
            "message-level description should come from the message's leading comment"
        );
    }

    #[test]
    fn schema_test_message() {
        let msg = load_message("SchemaTestMessage");
        let schema = message_input_schema(&msg);

        assert_eq!(schema["type"], json!("object"));
        assert_eq!(schema["additionalProperties"], json!(false));
        assert!(schema.get("required").is_none());

        let properties = props(&schema);

        // repeated scalar
        assert_eq!(
            properties["numbers"],
            json!({"type": "array", "items": {"type": "integer"}, "description": "A repeated scalar field."})
        );

        // map to scalar
        assert_eq!(
            properties["counts"],
            json!({"type": "object", "additionalProperties": {"type": "integer"}, "description": "A map from string to an int32."})
        );

        // map to message
        let named_inners = &properties["namedInners"];
        assert_eq!(named_inners["type"], json!("object"));
        assert_eq!(
            named_inners["description"],
            json!("A map from string to a message.")
        );
        let value_schema = &named_inners["additionalProperties"];
        assert_eq!(value_schema["type"], json!("object"));
        assert!(
            value_schema["properties"]
                .as_object()
                .unwrap()
                .contains_key("name")
        );

        // map to enum
        let color_by_name = &properties["colorByName"];
        assert_eq!(color_by_name["type"], json!("object"));
        let value_schema = &color_by_name["additionalProperties"];
        assert_eq!(value_schema["type"], json!("string"));
        let enum_values: BTreeSet<String> = value_schema["enum"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_str().unwrap().to_string())
            .collect();
        assert_eq!(
            enum_values,
            BTreeSet::from([
                "COLOR_UNSPECIFIED".to_string(),
                "COLOR_RED".to_string(),
                "COLOR_GREEN".to_string(),
                "COLOR_BLUE".to_string(),
            ])
        );

        // nested message
        let inner = &properties["inner"];
        assert_eq!(inner["type"], json!("object"));
        assert_eq!(
            inner["properties"]["name"],
            json!({"type": "string", "description": "A string field on the nested message."})
        );

        // recursive message
        let root = &properties["root"];
        assert_eq!(root["$ref"], json!("#/$defs/schemapb.Node"));
        let defs = schema["$defs"]
            .as_object()
            .expect("$defs should be present");
        // Only schemapb.Node is part of a reference cycle; every other
        // message type (e.g. Inner, reached through both "inner" and the
        // "namedInners" map) must be inlined at each occurrence rather than
        // kept in $defs, per the module doc's "other message types are
        // inlined... even when referenced more than once".
        assert_eq!(
            defs.keys().collect::<Vec<_>>(),
            vec!["schemapb.Node"],
            "$defs should contain only the recursive Node message, not every referenced message"
        );
        let node_schema = defs
            .get("schemapb.Node")
            .expect("$defs should contain the Node schema");
        assert_eq!(node_schema["type"], json!("object"));
        let children_schema = &node_schema["properties"]["children"];
        assert_eq!(children_schema["type"], json!("array"));
        assert_eq!(
            children_schema["items"]["$ref"],
            json!("#/$defs/schemapb.Node")
        );

        // enum
        let favorite_color = &properties["favoriteColor"];
        assert_eq!(favorite_color["type"], json!("string"));

        // repeated enum
        let colors = &properties["colors"];
        assert_eq!(colors["type"], json!("array"));
        assert_eq!(colors["items"]["type"], json!("string"));

        // proto3 optional field is a normal, non-required property
        assert_eq!(
            properties["nickname"],
            json!({"type": "string", "description": "A proto3 optional field: never required."})
        );

        // oneof fields are normal properties with the exact "part of
        // oneof" note text appended to their own description.
        const CONTACT_ONEOF_NOTE: &str =
            "Part of the \"contact\" oneof: at most one of its fields may be set.";
        assert_eq!(
            properties["email"],
            json!({
                "type": "string",
                "description": format!("Email address. {CONTACT_ONEOF_NOTE}"),
            })
        );
        assert_eq!(
            properties["phone"],
            json!({
                "type": "string",
                "description": format!("Phone number. {CONTACT_ONEOF_NOTE}"),
            })
        );

        let wkt_cases: &[(&str, Value)] = &[
            (
                "createdAt",
                json!({"type": "string", "format": "date-time"}),
            ),
            ("ttl", json!({"type": "string"})),
            ("nicknameWrapper", json!({"type": "string"})),
            ("countWrapper", json!({"type": "integer"})),
            ("flagWrapper", json!({"type": "boolean"})),
            (
                "blobWrapper",
                json!({"type": "string", "contentEncoding": "base64"}),
            ),
            ("metadata", json!({"type": "object"})),
            ("anyValue", json!({})),
            ("anyList", json!({"type": "array"})),
            (
                "updateMask",
                json!({
                    "type": "object",
                    "properties": {"paths": {"type": "array", "items": {"type": "string"}}},
                    "additionalProperties": false,
                }),
            ),
            (
                "nothing",
                json!({"type": "object", "additionalProperties": false}),
            ),
            ("scoreWrapper", float_schema()),
            ("ratioWrapper", float_schema()),
            ("bigCountWrapper", json!({"type": ["integer", "string"]})),
            (
                "bigUnsignedWrapper",
                json!({"type": ["integer", "string"], "minimum": 0}),
            ),
            (
                "smallUnsignedWrapper",
                json!({"type": "integer", "minimum": 0}),
            ),
            ("anyPayload", any_schema()),
        ];
        for (field, want) in wkt_cases {
            assert_eq!(&properties[*field], want, "schema for field {field:?}");
        }
    }

    /// Locks in that [`any_schema`] forbids unknown properties: unlike
    /// `protojson`'s `Any` handling (which this module deliberately does
    /// not replicate, see [`any_schema`]'s doc comment), pbjson encodes
    /// `Any` as the plain two-field message it is on the wire, so,
    /// consistent with every other ordinary message schema this module
    /// emits, `additionalProperties` is `false`.
    #[test]
    fn any_schema_forbids_additional_properties() {
        let schema = any_schema();
        assert_eq!(schema["additionalProperties"], json!(false));
    }

    /// Pins down the actual pbjson wire encoding of `google.protobuf.Any`
    /// that [`any_schema`]'s doc comment describes, and that it validates
    /// against the schema this module generates for it.
    #[test]
    fn any_schema_matches_pbjson_encoding() {
        let any = pbjson_types::Any {
            type_url: "type.googleapis.com/schemapb.Inner".to_string(),
            value: b"\x01\x02\x03".to_vec().into(),
        };
        let instance = serde_json::to_value(&any).expect("pbjson serialization");
        assert_eq!(
            instance,
            json!({"typeUrl": "type.googleapis.com/schemapb.Inner", "value": "AQID"}),
            "pbjson must encode Any as its plain typeUrl/value fields, not protojson's @type + unpacked payload"
        );

        let validator = jsonschema::validator_for(&any_schema()).expect("valid schema");
        assert!(
            validator.is_valid(&instance),
            "any_schema should accept pbjson's actual Any encoding"
        );
    }

    /// Checks that the special `"NaN"`/`"Infinity"`/`"-Infinity"` string
    /// encodings for float/double actually round-trip through the
    /// pbjson-generated deserializer (not just the schema in isolation),
    /// for a plain `float`/`double` field (`AllScalars`) and for the
    /// `FloatValue`/`DoubleValue` wrappers (`SchemaTestMessage`), then
    /// validates the resulting JSON against each message's generated
    /// schema. The counterpart of Go's `TestSchemaValidatesNonFiniteFloats`.
    #[test]
    fn schema_validates_non_finite_floats() {
        let scalars_msg = load_message("AllScalars");
        let scalars_schema = message_input_schema(&scalars_msg);
        let scalars_validator = jsonschema::validator_for(&scalars_schema).expect("valid schema");

        let test_msg = load_message("SchemaTestMessage");
        let test_schema = message_input_schema(&test_msg);
        let test_validator = jsonschema::validator_for(&test_schema).expect("valid schema");

        for s in ["NaN", "Infinity", "-Infinity"] {
            let scalars_json = json!({"sFloat": s, "sDouble": s});
            let scalars: schemapb::AllScalars = serde_json::from_value(scalars_json.clone())
                .unwrap_or_else(|e| {
                    panic!("pbjson should deserialize {s:?} for a plain float/double field: {e}")
                });
            assert!(scalars.s_float.is_nan() || scalars.s_float.is_infinite());
            assert!(scalars.s_double.is_nan() || scalars.s_double.is_infinite());
            assert!(
                scalars_validator.is_valid(&scalars_json),
                "AllScalars schema should accept {s:?} for sFloat/sDouble"
            );

            let wrapper_json = json!({"scoreWrapper": s, "ratioWrapper": s});
            let wrappers: schemapb::SchemaTestMessage =
                serde_json::from_value(wrapper_json.clone()).unwrap_or_else(|e| {
                    panic!(
                        "pbjson should deserialize {s:?} for a DoubleValue/FloatValue wrapper: {e}"
                    )
                });
            assert!(wrappers.score_wrapper.is_some());
            assert!(wrappers.ratio_wrapper.is_some());
            assert!(
                test_validator.is_valid(&wrapper_json),
                "SchemaTestMessage schema should accept {s:?} for scoreWrapper/ratioWrapper"
            );
        }
    }

    #[test]
    fn recursive_root_message() {
        let msg = load_message("Node");
        let schema = message_input_schema(&msg);

        assert_eq!(
            schema["type"],
            json!("object"),
            "root schema must have type \"object\", not a $ref"
        );
        assert!(
            schema.get("$ref").is_none(),
            "root schema must not itself be a $ref"
        );

        let properties = props(&schema);
        let children_schema = &properties["children"];
        assert_eq!(children_schema["type"], json!("array"));
        assert_eq!(
            children_schema["items"]["$ref"],
            json!("#/$defs/schemapb.Node")
        );

        let defs = schema["$defs"]
            .as_object()
            .expect("$defs should be present for a recursive root message");
        let node_schema = defs
            .get("schemapb.Node")
            .expect("$defs should contain the Node schema");
        assert_eq!(node_schema["type"], json!("object"));

        let validator = jsonschema::validator_for(&schema).expect("valid schema");
        let three = schemapb::Node {
            value: "root".to_string(),
            children: vec![schemapb::Node {
                value: "child".to_string(),
                children: vec![schemapb::Node {
                    value: "grandchild".to_string(),
                    children: vec![],
                }],
            }],
        };
        assert_proto_validates_against_schema(&validator, &three);
    }

    /// Checks that an empty (all-default) `SchemaTestMessage` validates
    /// against its own generated schema, per the issue's property-test
    /// acceptance criterion.
    #[test]
    fn schema_validates_empty_message() {
        let msg = load_message("SchemaTestMessage");
        let schema = message_input_schema(&msg);
        let validator = jsonschema::validator_for(&schema).expect("valid schema");

        let empty = schemapb::SchemaTestMessage::default();
        assert_proto_validates_against_schema(&validator, &empty);
    }

    /// Generates a number of randomly populated `SchemaTestMessage` values
    /// and checks that each one, pbjson-serialized, validates against the
    /// schema generated for `SchemaTestMessage`, per the issue's
    /// property-test acceptance criterion.
    #[test]
    fn schema_validates_populated_messages() {
        let msg = load_message("SchemaTestMessage");
        let schema = message_input_schema(&msg);
        let validator = jsonschema::validator_for(&schema).expect("valid schema");

        let mut rng = rand::rngs::StdRng::seed_from_u64(1);
        for _ in 0..50 {
            let m = random_schema_test_message(&mut rng, 0);
            assert_proto_validates_against_schema(&validator, &m);
        }
    }

    /// Asserts that `message`, pbjson-serialized, validates against
    /// `validator`: the counterpart of Go's `assertProtoValidatesAgainstSchema`
    /// validating a `protojson`-marshaled message.
    fn assert_proto_validates_against_schema<T: serde::Serialize + std::fmt::Debug>(
        validator: &jsonschema::Validator,
        message: &T,
    ) {
        let instance = serde_json::to_value(message).expect("pbjson serialization");
        if let Err(err) = validator.validate(&instance) {
            panic!("schema validation failed: {err}\ninstance: {instance}\nmessage: {message:?}");
        }
    }

    use rand::{Rng, RngExt, SeedableRng};

    fn random_string(rng: &mut impl Rng) -> String {
        const LETTERS: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFG";
        let n = rng.random_range(0..8usize);
        (0..n)
            .map(|_| LETTERS[rng.random_range(0..LETTERS.len())] as char)
            .collect()
    }

    fn random_color(rng: &mut impl Rng) -> i32 {
        rng.random_range(0..4)
    }

    fn random_node(rng: &mut impl Rng, depth: usize, max_depth: usize) -> schemapb::Node {
        let mut n = schemapb::Node {
            value: random_string(rng),
            children: vec![],
        };
        if depth < max_depth {
            let count = rng.random_range(0..3usize);
            for _ in 0..count {
                n.children.push(random_node(rng, depth + 1, max_depth));
            }
        }
        n
    }

    /// Generates a randomly populated `SchemaTestMessage`, the counterpart
    /// of Go's `randomSchemaTestMessage`. The oneof is left unset about a
    /// third of the time (a valid, common state the schema must also
    /// accept), and the well-known-type fields are populated on about half
    /// of the generated messages, so the property test above also exercises
    /// their special-cased schemas, not just the zero-value/absent case.
    fn random_schema_test_message(rng: &mut impl Rng, depth: usize) -> schemapb::SchemaTestMessage {
        use schemapb::schema_test_message::Contact;

        let mut m = schemapb::SchemaTestMessage {
            name: random_string(rng),
            favorite_color: random_color(rng),
            ..Default::default()
        };
        for _ in 0..rng.random_range(0..4usize) {
            m.numbers.push(rng.random());
        }
        if rng.random_range(0..2) == 0 {
            m.counts.insert("a".to_string(), rng.random());
            m.counts.insert("b".to_string(), rng.random());
        }
        if depth < 2 && rng.random_range(0..2) == 0 {
            m.named_inners.insert(
                "x".to_string(),
                schemapb::Inner {
                    name: random_string(rng),
                },
            );
        }
        if rng.random_range(0..2) == 0 {
            m.color_by_name
                .insert("favorite".to_string(), random_color(rng));
        }
        if depth < 2 && rng.random_range(0..2) == 0 {
            m.inner = Some(schemapb::Inner {
                name: random_string(rng),
            });
        }
        if depth < 2 {
            m.root = Some(random_node(rng, depth + 1, 2));
        }
        for _ in 0..rng.random_range(0..3usize) {
            m.colors.push(random_color(rng));
        }
        if rng.random_range(0..2) == 0 {
            m.nickname = Some(random_string(rng));
        }
        m.contact = match rng.random_range(0..3u8) {
            0 => Some(Contact::Email(random_string(rng))),
            1 => Some(Contact::Phone(random_string(rng))),
            _ => None,
        };

        if rng.random_range(0..2) == 0 {
            m.created_at = Some(pbjson_types::Timestamp {
                seconds: rng.random_range(0..2_000_000_000i64),
                nanos: 0,
            });
            m.ttl = Some(pbjson_types::Duration {
                seconds: 0,
                nanos: rng.random_range(0..1_000_000_000i32),
            });
            m.nickname_wrapper = Some(pbjson_types::StringValue {
                value: random_string(rng),
            });
            m.count_wrapper = Some(pbjson_types::Int32Value {
                value: rng.random(),
            });
            m.flag_wrapper = Some(pbjson_types::BoolValue {
                value: rng.random_range(0..2) == 0,
            });
            m.blob_wrapper = Some(pbjson_types::BytesValue {
                value: random_string(rng).into_bytes().into(),
            });
            let mut fields = std::collections::HashMap::new();
            fields.insert(
                "k".to_string(),
                pbjson_types::Value {
                    kind: Some(pbjson_types::value::Kind::StringValue(random_string(rng))),
                },
            );
            m.metadata = Some(pbjson_types::Struct { fields });
            m.any_value = Some(pbjson_types::Value {
                kind: Some(pbjson_types::value::Kind::StringValue(random_string(rng))),
            });
            m.any_list = Some(pbjson_types::ListValue {
                values: vec![
                    pbjson_types::Value {
                        kind: Some(pbjson_types::value::Kind::StringValue(random_string(rng))),
                    },
                    pbjson_types::Value {
                        kind: Some(pbjson_types::value::Kind::NumberValue(
                            rng.random_range(0..1000) as f64,
                        )),
                    },
                ],
            });
            m.update_mask = Some(pbjson_types::FieldMask {
                paths: vec!["name".to_string(), "root.value".to_string()],
            });
            m.nothing = Some(pbjson_types::Empty {});
            m.score_wrapper = Some(pbjson_types::DoubleValue {
                value: rng.random(),
            });
            m.ratio_wrapper = Some(pbjson_types::FloatValue {
                value: rng.random::<f32>(),
            });
            m.big_count_wrapper = Some(pbjson_types::Int64Value {
                value: rng.random(),
            });
            m.big_unsigned_wrapper = Some(pbjson_types::UInt64Value {
                value: rng.random(),
            });
            m.small_unsigned_wrapper = Some(pbjson_types::UInt32Value {
                value: rng.random(),
            });
            m.any_payload = Some(pbjson_types::Any {
                type_url: "type.googleapis.com/schemapb.Inner".to_string(),
                value: Vec::new().into(),
            });
        }
        m
    }

    /// Compiles `examples/protos/example.proto` (the repo's shared parity
    /// fixture) and returns the `MessageDescriptor` for `message_name` in
    /// the `examples.v1` package.
    fn load_example_message(message_name: &str) -> MessageDescriptor {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("examples/protos");
        let fds = protox::compile(["example.proto"], [dir]).expect("compiling example.proto");
        let pool = DescriptorPool::from_file_descriptor_set(fds).expect("building descriptor pool");
        pool.get_message_by_name(&format!("examples.v1.{message_name}"))
            .unwrap_or_else(|| panic!("message examples.v1.{message_name} not found"))
    }

    /// **Parity:** for each request message in `example.proto`, the schema
    /// this module builds is JSON-equal (comparing parsed values, not
    /// bytes, since Go's `encoding/json` HTML-escapes `<`, `>` and `&`) to
    /// the `InputSchema` embedded in Go's committed
    /// `examples/gen/example/v1/example_mcp.pb.go`
    /// (`stablekernel/protoc-gen-go-mcp` `9e072f9`). The expected values
    /// below are copied verbatim from that file's `json.RawMessage`
    /// literals.
    #[test]
    fn parity_with_go_example_input_schemas() {
        let cases: &[(&str, &str)] = &[
            (
                "SetVibeRequest",
                r#"{"additionalProperties":false,"description":"The request to set the vibe of the server","properties":{"vibe":{"description":"The vibe of the server to be set. Must match \\d+ or a `code` like \"chill\", and must not contain a literal newline.","type":"string"}},"type":"object"}"#,
            ),
            (
                "GetVibeRequest",
                r#"{"additionalProperties":false,"description":"The request to get the vibe of the server","properties":{},"type":"object"}"#,
            ),
            (
                "SetVibeDetailsRequest",
                r#"{"additionalProperties":false,"description":"The detailed vibe of the server","properties":{"vibe":{"description":"The vibe of the string to be set","type":"string"},"vibeScalar":{"additionalProperties":false,"description":"The details of the vibe","properties":{"vibeBool":{"description":"The details of the vibe bool","type":"boolean"},"vibeBytes":{"contentEncoding":"base64","description":"the details of the vibe bytes","type":"string"},"vibeDouble":{"anyOf":[{"type":"number"},{"enum":["NaN","Infinity","-Infinity"],"type":"string"}],"description":"The details of the vibe double"},"vibeEnum":{"description":"The details of the vibe string","items":{"description":"The details of the vibe string","enum":["VIBE_UNSET","VIBE_GOOD"],"type":"string"},"type":"array"},"vibeFixed32":{"description":"The details of the vibe fixed32","minimum":0,"type":"integer"},"vibeFixed64":{"description":"The details of the vibe fixed64","minimum":0,"type":["integer","string"]},"vibeFloat":{"anyOf":[{"type":"number"},{"enum":["NaN","Infinity","-Infinity"],"type":"string"}],"description":"the details of the vibe float"},"vibeInt32":{"description":"The details of the vibe int32","type":"integer"},"vibeInt64":{"description":"The details of the vibe int64","type":["integer","string"]},"vibeSfixed32":{"description":"The details of the vibe sfixed32","type":"integer"},"vibeSfixed64":{"description":"The details of the vibe sfixed64","type":["integer","string"]},"vibeSint32":{"description":"The details of the vibe sint32","type":"integer"},"vibeSint64":{"description":"The details of the vibe sint64","type":["integer","string"]},"vibeUint32":{"description":"The details of the vibe uint32","minimum":0,"type":"integer"},"vibeUint64":{"description":"The details of the vibe uint64","minimum":0,"type":["integer","string"]}},"type":"object"}},"type":"object"}"#,
            ),
            (
                "SetVibeArrayRequest",
                r#"{"additionalProperties":false,"description":"The vibe array request","properties":{"vibeArray":{"additionalProperties":false,"description":"The details of the vibe array","properties":{"vibeBools":{"description":"The details of the vibe bool array","items":{"type":"boolean"},"type":"array"},"vibeByteses":{"description":"the details of the vibe bytes array","items":{"contentEncoding":"base64","type":"string"},"type":"array"},"vibeDoubles":{"description":"The details of the vibe double array","items":{"anyOf":[{"type":"number"},{"enum":["NaN","Infinity","-Infinity"],"type":"string"}]},"type":"array"},"vibeFixed32s":{"description":"The details of the vibe fixed32 array","items":{"minimum":0,"type":"integer"},"type":"array"},"vibeFixed64s":{"description":"The details of the vibe fixed64 array","items":{"minimum":0,"type":["integer","string"]},"type":"array"},"vibeFloats":{"description":"the details of the vibe float array","items":{"anyOf":[{"type":"number"},{"enum":["NaN","Infinity","-Infinity"],"type":"string"}]},"type":"array"},"vibeInt32s":{"description":"The details of the vibe int32 array","items":{"type":"integer"},"type":"array"},"vibeInt64s":{"description":"The details of the vibe int64 array","items":{"type":["integer","string"]},"type":"array"},"vibeSfixed32s":{"description":"The details of the vibe sfixed32 array","items":{"type":"integer"},"type":"array"},"vibeSfixed64s":{"description":"The details of the vibe sfixed64 array","items":{"type":["integer","string"]},"type":"array"},"vibeSint32s":{"description":"The details of the vibe sint32 array","items":{"type":"integer"},"type":"array"},"vibeSint64s":{"description":"The details of the vibe sint64 array","items":{"type":["integer","string"]},"type":"array"},"vibeUint32s":{"description":"The details of the vibe uint32 array","items":{"minimum":0,"type":"integer"},"type":"array"},"vibeUint64s":{"description":"The details of the vibe uint64 array","items":{"minimum":0,"type":["integer","string"]},"type":"array"}},"type":"object"}},"type":"object"}"#,
            ),
            (
                "SetVibeObjectsRequest",
                r#"{"additionalProperties":false,"description":"The request to set multiple vibe objects on the server","properties":{"vibeObject":{"description":"The details of the vibe","items":{"additionalProperties":false,"description":"The vibe object of the server","properties":{"vibe":{"description":"The vibe of the server","type":"string"}},"type":"object"},"type":"array"}},"type":"object"}"#,
            ),
        ];

        for (message_name, go_schema_json) in cases {
            let msg = load_example_message(message_name);
            let got = message_input_schema(&msg);
            let want: Value =
                serde_json::from_str(go_schema_json).expect("parsing Go's expected schema JSON");
            assert_eq!(&got, &want, "schema for examples.v1.{message_name}");
        }

        // The comparisons above parse both sides into `serde_json::Value`,
        // which is only meaningful because this module builds schemas
        // without the `preserve_order` feature (see the module doc
        // comment): keys sort, so two schemas with the same keys and
        // values always compare equal regardless of insertion order. Pin
        // one case's *serialized* string too, so a future workspace-wide
        // enabling of `preserve_order` (#6 embeds this schema as tool
        // input schema text, not just a `Value`) that reintroduces
        // nondeterministic key order is caught here rather than silently
        // producing a tool schema whose key order differs run to run.
        let get_vibe_schema = message_input_schema(&load_example_message("GetVibeRequest"));
        assert_eq!(
            serde_json::to_string(&get_vibe_schema).unwrap(),
            r#"{"additionalProperties":false,"description":"The request to get the vibe of the server","properties":{},"type":"object"}"#,
            "serialized schema key order should be sorted/deterministic"
        );
    }
}
