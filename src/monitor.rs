use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Instant;

use http_body_util::Full;
use hyper::{Request, Response};
use hyper::body::{Bytes, Incoming};
use hyper::server::conn::{http1};
use hyper::service::service_fn;
use hyper_util::rt::TokioIo;
use log::error;
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
        let server = self.create_server().await.unwrap();
        loop {
            select! {
                _ = self.context.cancelled() => { return Err(Terminated) },
                connection = server.accept() => { self.process_connection(connection).await }
            }
        }
    }

    async fn create_server(&self) -> Result<TcpListener, MonitorError> {
        match TcpListener::bind(SocketAddr::from(([0, 0, 0, 0], 3000))).await {
            Ok(server) => Ok(server),
            Err(error) => Err(PortBindingFailed(error)),
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
        if let Err(error) = http1::Builder::new().keep_alive(false).serve_connection(stream, service).await {
            error!("{}", error);
        }
    }

    pub async fn get_health(&self, _request: Request<Incoming>) -> Result<Response<Full<Bytes>>, Infallible> {
        let payload = json!({
            "health": {
                "uptime": self.start_time.elapsed().as_secs(),
                "bytes_received": self.listener.get_bytes(),
                "bytes_written": self.recorder.get_bytes()
            }
        });

        Ok(self.build_response(payload))
    }

    fn build_response(&self, payload: Value) -> Response<Full<Bytes>> {
        Response::builder()
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
