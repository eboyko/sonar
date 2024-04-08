use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;

use http_body_util::Full;
use hyper::body::{Bytes, Incoming};
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper::{Request, Response};
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
use crate::recorder::Recorder;

mod error;

#[derive(Clone)]
pub(crate) struct Monitor {
    listener: Arc<Listener>,
    recorder: Arc<Recorder>,
    context: CancellationToken,
    start_time: Instant,
}

impl Monitor {
    pub(crate) fn new(
        listener: Arc<Listener>,
        recorder: Arc<Recorder>,
        context: CancellationToken,
    ) -> Self {
        Monitor {
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
                    warn!("Termination signal received");
                    return Err(Terminated)
                },
            }
        }
    }

    async fn run_server(&self) -> Result<TcpListener, MonitorError> {
        let address = SocketAddr::from(([0, 0, 0, 0], 3000));

        match TcpListener::bind(address).await {
            Ok(server) => Ok(server),
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
                "uptime": self.start_time.elapsed().as_secs(),
                "bytes_received": self.listener.get_bytes(),
                "bytes_written": self.recorder.get_bytes()
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
}

pub(crate) async fn build(
    listener: Arc<Listener>,
    recorder: Arc<Recorder>,
    context: CancellationToken,
) -> Result<Arc<Monitor>, MonitorError> {
    Ok(Arc::new(Monitor::new(listener, recorder, context)))
}
