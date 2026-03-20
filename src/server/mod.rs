pub mod handler;

use std::path::PathBuf;
use std::sync::Arc;

use axum::{routing::get, Router};
use rmcp::transport::streamable_http_server::{
    session::local::LocalSessionManager, tower::StreamableHttpService, StreamableHttpServerConfig,
};
use tokio_util::sync::CancellationToken;

use crate::error::Result;
use crate::skills::discovery;
use crate::skills::parser;

use self::handler::SkillsServer;

/// Build a SkillsServer from skill search paths.
pub fn build_server(paths: &[PathBuf]) -> Result<SkillsServer> {
    let skill_dirs = discovery::discover_skills(paths)?;
    let mut skills = Vec::new();

    for dir in &skill_dirs {
        let source = dir.parent().and_then(|p| p.to_str()).unwrap_or("unknown");
        match parser::parse_skill(dir, source) {
            Ok(skill) => {
                eprintln!("[sxmc] Loaded skill: {}", skill.name);
                skills.push(skill);
            }
            Err(e) => {
                eprintln!("[sxmc] Warning: failed to parse {}: {}", dir.display(), e);
            }
        }
    }

    eprintln!(
        "[sxmc] Loaded {} skills with {} tools and {} resources",
        skills.len(),
        skills.iter().map(|s| s.scripts.len()).sum::<usize>(),
        skills.iter().map(|s| s.references.len()).sum::<usize>(),
    );

    Ok(SkillsServer::new(skills))
}

/// Run the MCP server over stdio.
pub async fn serve_stdio(paths: &[PathBuf]) -> Result<()> {
    let server = build_server(paths)?;
    let transport = rmcp::transport::stdio();

    let service = rmcp::ServiceExt::serve(server, transport)
        .await
        .map_err(|e| crate::error::SxmcError::McpError(e.to_string()))?;

    service
        .waiting()
        .await
        .map_err(|e| crate::error::SxmcError::McpError(e.to_string()))?;

    Ok(())
}

fn build_streamable_http_service(
    paths: Arc<Vec<PathBuf>>,
    cancellation_token: CancellationToken,
) -> StreamableHttpService<SkillsServer, LocalSessionManager> {
    StreamableHttpService::new(
        move || build_server(&paths).map_err(std::io::Error::other),
        Default::default(),
        StreamableHttpServerConfig {
            stateful_mode: true,
            json_response: false,
            cancellation_token,
            ..Default::default()
        },
    )
}

fn build_http_router(paths: Arc<Vec<PathBuf>>, cancellation_token: CancellationToken) -> Router {
    let service = build_streamable_http_service(paths, cancellation_token);

    Router::new()
        .route(
            "/",
            get(|| async { "sxmc streamable HTTP MCP server\nEndpoint: /mcp\n" }),
        )
        .nest_service("/mcp", service)
}

/// Run the MCP server over streamable HTTP.
pub async fn serve_http(paths: &[PathBuf], host: &str, port: u16) -> Result<()> {
    let bind_addr = format!("{host}:{port}");
    let listener = tokio::net::TcpListener::bind(&bind_addr)
        .await
        .map_err(|e| crate::error::SxmcError::Other(format!("Failed to bind {bind_addr}: {e}")))?;
    let local_addr = listener
        .local_addr()
        .map_err(|e| crate::error::SxmcError::Other(format!("Failed to read local addr: {e}")))?;
    let cancellation_token = CancellationToken::new();
    let router = build_http_router(Arc::new(paths.to_vec()), cancellation_token.clone());

    eprintln!(
        "[sxmc] Streamable HTTP MCP server listening at http://{}/mcp",
        local_addr
    );

    let shutdown = cancellation_token.clone();
    tokio::spawn(async move {
        let _ = tokio::signal::ctrl_c().await;
        shutdown.cancel();
    });

    axum::serve(listener, router)
        .with_graceful_shutdown(async move {
            cancellation_token.cancelled_owned().await;
        })
        .await
        .map_err(|e| crate::error::SxmcError::Other(format!("HTTP server failed: {e}")))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;

    #[tokio::test]
    async fn test_streamable_http_server_serves_mcp_endpoint() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let cancel = CancellationToken::new();
        let router = build_http_router(
            Arc::new(vec![PathBuf::from("tests/fixtures")]),
            cancel.child_token(),
        );

        let handle = tokio::spawn({
            let cancel = cancel.clone();
            async move {
                let _ = axum::serve(listener, router)
                    .with_graceful_shutdown(async move {
                        cancel.cancelled_owned().await;
                    })
                    .await;
            }
        });

        let response = reqwest::Client::new()
            .post(format!("http://{addr}/mcp"))
            .header("Content-Type", "application/json")
            .header("Accept", "application/json, text/event-stream")
            .body(r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-03-26","capabilities":{},"clientInfo":{"name":"test","version":"1.0"}}}"#)
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");
        assert!(content_type.contains("text/event-stream"));

        cancel.cancel();
        handle.await.unwrap();
    }
}
