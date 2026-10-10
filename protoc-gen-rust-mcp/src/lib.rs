//! `protoc-gen-rust-mcp`'s code generation library: turning a
//! `CodeGeneratorRequest` into a `CodeGeneratorResponse` that defines, for
//! each gRPC service in the input, an MCP (Model Context Protocol) server
//! that forwards tool calls to a `tonic` client. See `docs/GOAL.md` for the
//! project goal, and `src/main.rs` for the `protoc` plugin binary that
//! drives this library over stdin/stdout.

pub mod comments;
pub mod field_mask;
pub mod generator;
pub mod header;
pub mod rust_literal;
pub mod schema;
pub mod server;
pub mod source_info;
