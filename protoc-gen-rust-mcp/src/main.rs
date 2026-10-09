//! `protoc-gen-rust-mcp`: a `protoc` plugin that generates an MCP (Model
//! Context Protocol) server for each gRPC service in the input, forwarding
//! tool calls to a `tonic` client. See `docs/GOAL.md` for the project goal.
//!
//! Like any `protoc` plugin, this binary reads an encoded
//! `CodeGeneratorRequest` from stdin and writes an encoded
//! `CodeGeneratorResponse` to stdout; `protoc` invokes it as
//! `protoc-gen-rust-mcp` when passed `--rust-mcp_out`.

use std::io::{self, Read, Write};
use std::process::ExitCode;

use prost::Message;
use prost_types::compiler::CodeGeneratorRequest;

use protoc_gen_rust_mcp::generator;

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    if args.any(|a| a == "--version") {
        println!("protoc-gen-rust-mcp {}", env!("CARGO_PKG_VERSION"));
        return ExitCode::SUCCESS;
    }

    if let Err(err) = run() {
        eprintln!("protoc-gen-rust-mcp: {err}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}

fn run() -> io::Result<()> {
    let mut buf = Vec::new();
    io::stdin().read_to_end(&mut buf)?;
    let request = CodeGeneratorRequest::decode(buf.as_slice())
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    let response = generator::generate(&request);

    let mut out = Vec::new();
    response
        .encode(&mut out)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
    io::stdout().write_all(&out)?;
    Ok(())
}
