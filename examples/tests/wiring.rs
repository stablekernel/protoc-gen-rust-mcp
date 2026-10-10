//! This file backs the README's "What the plugin generates" section:
//! `cargo build --tests`/`cargo test` compile it (it is a `tests/*.rs`
//! file, so plain `cargo build` on the `examples` crate skips it), so the
//! wiring snippets shown there cannot silently drift from code that
//! actually compiles against the generated API. It is the counterpart of
//! Go's `examples/gen/example/v1/example_test.go` (`Example_wiring`,
//! `stablekernel/protoc-gen-go-mcp` at `9e072f9`).
//!
//! Like Go's `Example_wiring`, the two functions below (`serve_stdio`,
//! `serve_streamable_http`) are never called: both would block forever
//! (an MCP server waiting on its transport), so neither is an `#[test]`.
//! `#[allow(dead_code)]` silences the resulting warning; compiling them
//! is the whole point.

#![allow(dead_code)]

use examples::v1::VibeServiceMcpServer;
use examples::v1::vibe_service_client::VibeServiceClient;
use rmcp::ServiceExt;

/// Over stdio, for CLI-launched MCP clients (see
/// `examples/src/bin/mcp-vibe.rs` for the full, runnable version of
/// this).
async fn serve_stdio(
    client: VibeServiceClient<tonic::transport::Channel>,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut server = VibeServiceMcpServer::new(client)
        .with_server_info(rmcp::model::Implementation::new("vibe", "0.0.1"));
    server.register_default_tools();

    let running = server.serve(rmcp::transport::io::stdio()).await?;
    running.waiting().await?;
    Ok(())
}

/// Over Streamable HTTP, for network clients: `rmcp`'s
/// `StreamableHttpService` is a `tower::Service`, wired into a hyper
/// server here with `hyper-util`'s `TowerToHyperService` adapter (the
/// same pairing `rmcp`'s own
/// `examples/servers/src/counter_hyper_streamable_http.rs` uses).
/// `client` is the same `VibeServiceClient` as above; a fresh
/// `VibeServiceMcpServer` is built per session (the service factory
/// closure below), matching Go's one-`*mcp.Server`-per-request `rmcp`
/// equivalent.
async fn serve_streamable_http(
    client: VibeServiceClient<tonic::transport::Channel>,
) -> Result<(), Box<dyn std::error::Error>> {
    use hyper_util::rt::{TokioExecutor, TokioIo};
    use hyper_util::server::conn::auto::Builder;
    use hyper_util::service::TowerToHyperService;
    use rmcp::transport::streamable_http_server::StreamableHttpService;
    use rmcp::transport::streamable_http_server::session::local::LocalSessionManager;

    let service = TowerToHyperService::new(StreamableHttpService::new(
        move || {
            let mut server = VibeServiceMcpServer::new(client.clone())
                .with_server_info(rmcp::model::Implementation::new("vibe", "0.0.1"));
            server.register_default_tools();
            Ok::<_, std::io::Error>(server)
        },
        LocalSessionManager::default().into(),
        Default::default(),
    ));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await?;
    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let service = service.clone();
        tokio::spawn(async move {
            let _ = Builder::new(TokioExecutor::default())
                .serve_connection(io, service)
                .await;
        });
    }
}
