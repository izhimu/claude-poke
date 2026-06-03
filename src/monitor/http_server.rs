use crate::state::StatusFile;
use anyhow::Result;
use log::{debug, error, info, warn};
use std::net::SocketAddr;
use std::sync::mpsc;

/// A minimal HTTP server that receives status updates via POST /status.
///
/// The server runs in a background thread. Dropping this handle does NOT
/// stop the server — it lives until the process exits.
pub struct HttpServer {
    port: u16,
}

impl HttpServer {
    /// Start the HTTP server on the given port.
    /// Received status updates are sent through `tx`.
    pub fn start(port: u16, tx: mpsc::Sender<StatusFile>) -> Result<Self> {
        let addr: SocketAddr = ([127, 0, 0, 1], port).into();
        let server = tiny_http::Server::http(addr)
            .map_err(|e| anyhow::anyhow!("Failed to start HTTP server on {}: {}", addr, e))?;

        info!("HTTP server listening on http://{}", addr);

        std::thread::Builder::new()
            .name("http-status-server".into())
            .spawn(move || {
                Self::run_loop(server, tx);
            })?;

        Ok(Self { port })
    }

    #[allow(dead_code)]
    pub fn port(&self) -> u16 {
        self.port
    }

    fn run_loop(server: tiny_http::Server, tx: mpsc::Sender<StatusFile>) {
        for mut request in server.incoming_requests() {
            let method = request.method().clone();
            let url = request.url().to_string();

            if url != "/status" {
                let response = tiny_http::Response::from_string("Not Found")
                    .with_status_code(404);
                let _ = request.respond(response);
                continue;
            }

            if method != tiny_http::Method::Post {
                let response = tiny_http::Response::from_string("Method Not Allowed")
                    .with_status_code(405);
                let _ = request.respond(response);
                continue;
            }

            // Read request body via as_reader (borrows, doesn't consume)
            let mut body = String::new();
            if let Err(e) = request.as_reader().read_to_string(&mut body) {
                warn!("Failed to read request body: {}", e);
                let response = tiny_http::Response::from_string("Bad Request")
                    .with_status_code(400);
                let _ = request.respond(response);
                continue;
            }

            debug!("Received status update: {}", body);

            match StatusFile::from_json(&body) {
                Ok(status) => {
                    let _ = tx.send(status);
                    let response = tiny_http::Response::empty(204);
                    let _ = request.respond(response);
                }
                Err(e) => {
                    error!("Invalid status JSON: {}", e);
                    let response =
                        tiny_http::Response::from_string(format!("Bad Request: {}", e))
                            .with_status_code(400);
                    let _ = request.respond(response);
                }
            }
        }
    }
}
