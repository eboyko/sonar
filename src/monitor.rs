use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;
use std::thread::sleep;
use std::time::Instant;

use http_body_util::Full;
use hyper::{Request, Response};
use hyper::body::Bytes;
use hyper::server::conn::http1;
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use log::{debug, error, info, warn};
use serde_json::json;
use tokio::io::Result as Connection;
use tokio::net::{TcpListener, TcpStream};
use tokio::select;
use tokio_util::sync::CancellationToken;

use crate::error::Error;
use crate::recorder::Recorder;

pub(crate) struct Monitor {
    start_time: Instant,
    server: TcpListener,
    recorder: Arc<RwLock<Recorder>>,
    context: CancellationToken,
}

impl Monitor {
    pub async fn start(&mut self) {
        info!("Started");

        loop {
            select! {
                request = self.server.accept() => { self.handle_request(request).await }
                context = self.context.cancelled() => {
                    warn!("Shutting down");
                    break
                }
            }
        }
    }

    async fn handle_request(&self, request: Connection<(TcpStream, SocketAddr)>) {
        match request {
            Ok((stream, _)) => { self.process_request(stream).await }
            Err(error) => error!("Error accepting connection: {}", error)
        }
    }

    async fn process_request(&self, stream: TcpStream) {
        let connection = TokioIo::new(stream);
        let service = service_fn(move |request| self.health(request));

        if let Err(error) = http1::Builder::new().serve_connection(connection, service).await {
            error!("Error: {}", error);
        }
    }

    pub async fn health(&self, _: Request<hyper::body::Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
        let payload = json!({
            "health": {
                "uptime": self.start_time.elapsed().as_secs(),
                "bytes_written": self.recorder.read().unwrap().bytes_counter,
                "errors": []
            }
        });

        let body = Full::from(Bytes::from(payload.to_string()));

        let response =
            Response::builder()
                .header("Content-Type", "application/json")
                .body(body)
                .unwrap();

        Ok(response)
    }
}

pub(crate) async fn build(recorder: Arc<RwLock<Recorder>>, context: CancellationToken) -> Result<Monitor, Error> {
    let address = SocketAddr::from(([0, 0, 0, 0], 3000));
    let server = TcpListener::bind(address).await?;

    Ok(Monitor { start_time: Instant::now(), server, recorder, context })
}
