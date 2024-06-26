use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;

use http_body_util::Full;
use hyper::{Request, Response};
use hyper::body::{Bytes, Incoming};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use log::{error, info, warn};
use serde_json::{json, Value};
use tokio::io::Result as Connection;
use tokio::net::{TcpListener, TcpStream};
use tokio::select;
use tokio_util::sync::CancellationToken;

use crate::listener::Listener;
use crate::monitor::error::Error as MonitorError;
use crate::monitor::error::Error::{PortBindingFailed, Terminated};
use crate::storage::recorder::Recorder;

mod error;

pub(crate) struct Monitor {
    port: u16,
    listener: Arc<Listener>,
    recorder: Arc<Recorder>,
    context: CancellationToken,
    start_time: Instant,
}

impl Monitor {
    pub(crate) fn new(
        port: u16,
        listener: Arc<Listener>,
        recorder: Arc<Recorder>,
        context: CancellationToken,
    ) -> Self {
        Monitor {
            port,
            listener,
            recorder,
            context,
            start_time: Instant::now(),
        }
    }

    pub async fn start(&self) -> Result<(), MonitorError> {
        let server = self.run_server().await?;

        loop {
            select! {
                connection = server.accept() => { self.process_connection(connection).await },
                _ = self.context.cancelled() => {
                    warn!("Termination signal received. Shutting down.");
                    return Err(Terminated);
                },
            }
        }
    }

    async fn run_server(&self) -> Result<TcpListener, MonitorError> {
        let address = SocketAddr::from(([0, 0, 0, 0], self.port));

        match TcpListener::bind(address).await {
            Ok(server) => {
                info!("Serving health requests on http://{}/health", server.local_addr().unwrap());
                Ok(server)
            }
            Err(error) => {
                error!("Failed to start the server ({})", error);
                Err(PortBindingFailed(error))
            }
        }
    }

    async fn process_connection(&self, connection: Connection<(TcpStream, SocketAddr)>) {
        match connection {
            Ok((stream, _)) => self.process_stream(TokioIo::new(stream)).await,
            Err(error) => error!("{}", error),
        }
    }

    async fn process_stream(&self, stream: TokioIo<TcpStream>) {
        let service = service_fn(move |request| self.get_health(request));
        let handle = http1::Builder::new().keep_alive(false).serve_connection(stream, service);
        if let Err(error) = handle.await {
            error!("{}", error);
        }
    }

    pub async fn get_health(&self, request: Request<Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
        if request.uri().path() != "/health" {
            return Ok(self.build_response(404, json!({ "success": "false" })));
        }

        let payload = json!({
            "health": {
                "data_received": format!("{:.2} MB", self.megabytes(self.listener.bytes_received())),
                "data_written": format!("{:.2} MB", self.megabytes(self.recorder.bytes_written())),
                "disk_space_available": format!("{:.2} MB", self.megabytes(self.recorder.bytes_available())),
                "uptime": format!("{} seconds", self.start_time.elapsed().as_secs()),
            }
        });

        Ok(self.build_response(200, payload))
    }

    fn build_response(&self, status: u16, payload: Value) -> Response<Full<Bytes>> {
        Response::builder()
            .status(status)
            .header("Content-Type", "application/json")
            .body(Full::from(Bytes::from(payload.to_string())))
            .unwrap()
    }

    fn megabytes(&self, bytes: usize) -> f32 {
        bytes as f32 / 1_048_576.0
    }
}

pub(crate) fn build(
    port: u16,
    listener: Arc<Listener>,
    recorder: Arc<Recorder>,
    context: CancellationToken,
) -> Arc<Monitor> {
    Arc::new(Monitor::new(port, listener, recorder, context))
}
