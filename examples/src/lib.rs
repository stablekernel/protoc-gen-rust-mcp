//! Generated code for the example `VibeService`
//! (`examples/protos/example.proto`), committed so the parity checks have a
//! stable target. Regenerate with `make generate` after changing the proto
//! or the generator; never edit `src/gen/` by hand.

pub mod v1 {
    // protoc-gen-prost-serde 0.4.0 emits `write!(f, "...", &FIELDS)` for enum
    // visitors; FIELDS is a `&[&str]` so the extra reference is redundant,
    // but we don't control that generated code and must not edit it by hand.
    #![allow(clippy::useless_borrows_in_formatting)]
    include!("gen/examples/v1/examples.v1.rs");
}
