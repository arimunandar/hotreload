use std::convert::Infallible;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::oneshot;
use warp::Filter;
use warp::reply::Reply;

/// Start the HTTP file server to serve compiled dylibs
#[allow(dead_code)]
pub async fn start_file_server(
    dylib_dir: PathBuf,
    port: u16,
    shutdown_rx: oneshot::Receiver<()>,
) -> anyhow::Result<()> {
    let dylib_dir = Arc::new(dylib_dir);

    let dir = dylib_dir.clone();
    let dylib_route = warp::path!("dylib" / String).then(move |name: String| {
        let dir = dir.clone();
        async move { serve_dylib(dir, name).await }
    });

    let health_route =
        warp::path("health").map(|| warp::reply::with_status("ok", warp::http::StatusCode::OK));

    let routes = dylib_route
        .or(health_route)
        .with(warp::cors().allow_any_origin());

    let addr: SocketAddr = ([0, 0, 0, 0], port).into();
    tracing::info!("Starting dylib HTTP server on http://{}", addr);

    let (_actual_addr, server) = warp::serve(routes)
        .bind_with_graceful_shutdown(addr, async {
            shutdown_rx.await.ok();
        });

    server.await;
    tracing::info!("File server stopped");
    Ok(())
}

/// Start the file server in a background task, returning the port and shutdown handle
pub async fn start_file_server_with_port(
    dylib_dir: PathBuf,
    port: u16,
) -> anyhow::Result<(u16, oneshot::Sender<()>)> {
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
    let dir = dylib_dir.clone();

    let dir_filter = Arc::new(dir);
    let dylib_route = warp::path!("dylib" / String).then(move |name: String| {
        let dir = dir_filter.clone(); // keep Arc alive across invocations
        async move { serve_dylib(dir, name).await }
    });

    let health_route =
        warp::path("health").map(|| warp::reply::with_status("ok", warp::http::StatusCode::OK));

    let routes = dylib_route
        .or(health_route)
        .with(warp::cors().allow_any_origin());

    // Try the requested port first, then fall back to nearby ports
    let mut actual_port = port;
    for attempt in 0..10 {
        let try_port = port + attempt;
        let addr: SocketAddr = ([0, 0, 0, 0], try_port).into();

        // Check if port is available
        match std::net::TcpListener::bind(addr) {
            Ok(listener) => {
                drop(listener);
                actual_port = try_port;
                break;
            }
            Err(_) => {
                if attempt == 0 {
                    tracing::info!("Port {} in use, trying next...", try_port);
                }
                continue;
            }
        }
    }

    let addr: SocketAddr = ([0, 0, 0, 0], actual_port).into();
    tracing::info!("Starting dylib HTTP server on http://0.0.0.0:{}", actual_port);

    tokio::spawn(async move {
        let (actual_addr, server) = warp::serve(routes)
            .bind_with_graceful_shutdown(addr, async {
                shutdown_rx.await.ok();
            });
        tracing::info!("File server listening on http://{}", actual_addr);
        server.await;
    });

    Ok((actual_port, shutdown_tx))
}

async fn serve_dylib(
    dir: Arc<PathBuf>,
    name: String,
) -> Result<warp::reply::Response, Infallible> {
    // Basic path traversal protection
    if name.contains('/') || name.contains("..") {
        return Ok(warp::reply::with_status("invalid path", warp::http::StatusCode::BAD_REQUEST)
            .into_response());
    }

    let file_path = dir.join(&name);

    match tokio::fs::read(&file_path).await {
        Ok(data) => {
            let mut response = warp::reply::Response::new(data.into());
            response.headers_mut().insert(
                "Content-Type",
                warp::http::header::HeaderValue::from_static("application/octet-stream"),
            );
            Ok(response)
        }
        Err(_) => Ok(
            warp::reply::with_status("not found", warp::http::StatusCode::NOT_FOUND)
                .into_response(),
        ),
    }
}
