//! `mcp-vibe`: a demo MCP server for the example `VibeService`
//! (`examples/protos/example.proto`), the counterpart of Go's
//! `cmd/mcp-vibe/` (`stablekernel/protoc-gen-go-mcp` at `9e072f9`).
//!
//! It starts an in-process tonic `VibeService` backend (below), connects a
//! client to it, builds a `VibeServiceMcpServer` with
//! [`register_default_tools`](examples::v1::VibeServiceMcpServer::register_default_tools),
//! and serves MCP over stdio. Nothing writes to stdout except the MCP
//! transport: all logs go to stderr.
//!
//! Install with `cargo install --path examples --bin mcp-vibe` and point an
//! MCP client at the resulting `mcp-vibe` binary over stdio.

use std::sync::Mutex;

use examples::v1::vibe_service_client::VibeServiceClient;
use examples::v1::vibe_service_server::{VibeService, VibeServiceServer};
use examples::v1::{
    GetVibeRequest, GetVibeResponse, SetVibeArrayRequest, SetVibeArrayResponse,
    SetVibeDetailsRequest, SetVibeObjectsRequest, SetVibeObjectsResponse, SetVibeRequest,
    SetVibeResponse, VibeServiceMcpServer,
};
use rmcp::ServiceExt;
use tonic::transport::server::TcpIncoming;

/// A `VibeService` backend holding a single, process-wide vibe, starting
/// at `"initial vibe"`. Only `SetVibe` and `GetVibe` are implemented, as
/// in Go's `vibe.go`; the rest return `Unimplemented`.
struct VibeServiceBackend {
    vibe: Mutex<String>,
}

#[tonic::async_trait]
impl VibeService for VibeServiceBackend {
    async fn set_vibe(
        &self,
        request: tonic::Request<SetVibeRequest>,
    ) -> Result<tonic::Response<SetVibeResponse>, tonic::Status> {
        let new_vibe = request.into_inner().vibe;
        if new_vibe.is_empty() {
            // Go's `vibe.go` returns a plain `fmt.Errorf("vibe cannot be
            // empty")`, which grpc-go's server interceptor reports to the
            // client as `codes.Unknown` (the default for a non-`Status`
            // error), not `InvalidArgument`; `tonic::Status::unknown`
            // matches that.
            return Err(tonic::Status::unknown("vibe cannot be empty"));
        }

        let mut vibe = self.vibe.lock().unwrap_or_else(|e| e.into_inner());
        let previous_vibe = std::mem::replace(&mut *vibe, new_vibe.clone());

        Ok(tonic::Response::new(SetVibeResponse {
            previous_vibe,
            vibe: new_vibe,
        }))
    }

    async fn get_vibe(
        &self,
        _request: tonic::Request<GetVibeRequest>,
    ) -> Result<tonic::Response<GetVibeResponse>, tonic::Status> {
        let vibe = self.vibe.lock().unwrap_or_else(|e| e.into_inner()).clone();
        Ok(tonic::Response::new(GetVibeResponse { vibe }))
    }

    async fn set_vibe_details(
        &self,
        _request: tonic::Request<SetVibeDetailsRequest>,
    ) -> Result<tonic::Response<SetVibeResponse>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn set_vibe_array(
        &self,
        _request: tonic::Request<SetVibeArrayRequest>,
    ) -> Result<tonic::Response<SetVibeArrayResponse>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented"))
    }

    async fn set_vibe_objects(
        &self,
        _request: tonic::Request<SetVibeObjectsRequest>,
    ) -> Result<tonic::Response<SetVibeObjectsResponse>, tonic::Status> {
        Err(tonic::Status::unimplemented("not implemented"))
    }
}

/// Starts an in-process `VibeServiceBackend` on an OS-assigned ephemeral
/// TCP port (`127.0.0.1:0`) and returns a `VibeServiceClient` connected
/// to it, plus a handle that shuts the backend down when dropped or
/// explicitly aborted.
async fn spawn_backend()
-> Result<VibeServiceClient<tonic::transport::Channel>, Box<dyn std::error::Error>> {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    let incoming = TcpIncoming::from(listener);
    let backend = VibeServiceBackend {
        vibe: Mutex::new("initial vibe".to_string()),
    };
    tokio::spawn(async move {
        if let Err(e) = tonic::transport::Server::builder()
            .add_service(VibeServiceServer::new(backend))
            .serve_with_incoming(incoming)
            .await
        {
            eprintln!("mcp-vibe: in-process VibeService backend stopped: {e}");
        }
    });
    let client = VibeServiceClient::connect(format!("http://{addr}")).await?;
    Ok(client)
}

/// Resolves when a termination signal is received: Ctrl+C (`SIGINT` on
/// Unix), or `SIGTERM` on Unix. On non-Unix platforms only Ctrl+C is
/// handled.
async fn shutdown_signal() {
    #[cfg(unix)]
    {
        let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("install SIGTERM handler");
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {}
            _ = sigterm.recv() => {}
        }
    }
    #[cfg(not(unix))]
    {
        let _ = tokio::signal::ctrl_c().await;
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = spawn_backend().await?;

    let mut server = VibeServiceMcpServer::new(client)
        .with_server_info(rmcp::model::Implementation::new("vibe", "0.0.1"));
    server.register_default_tools();

    let running = server.serve(rmcp::transport::io::stdio()).await?;
    eprintln!("mcp-vibe: serving MCP over stdio");
    let cancellation_token = running.cancellation_token();
    tokio::spawn(async move {
        shutdown_signal().await;
        eprintln!("mcp-vibe: shutting down");
        cancellation_token.cancel();
    });

    // Resolves once the transport closes (stdin EOF), or once the spawned
    // task above cancels it on SIGINT/SIGTERM. The process exits right
    // after, so the signal-watcher task above never needs to be joined or
    // aborted explicitly on the stdin-EOF path.
    running.waiting().await?;

    Ok(())
}
