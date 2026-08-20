use std::time::Duration;
use bytes::Bytes;
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio::time::timeout;
use tokio_rustls::TlsAcceptor;
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};
use tracing::{error, info, trace, warn};
use common::action_codes::ActionCode;
use common::certificates::get_certificate_subject;
use crate::router::Router;

pub struct ClientConnection<'a> {
    listener: &'a TcpListener,
    acceptor: TlsAcceptor,
}

impl<'a> ClientConnection<'a> {
    pub fn new(listener: &'a TcpListener, acceptor: TlsAcceptor) -> Self {
        ClientConnection {
            listener,
            acceptor,
        }
    }

    pub async fn handle(self) -> anyhow::Result<()> {
        let (stream, addr) = self.listener.accept().await?;
        let acceptor = self.acceptor;

        trace!("New connection from: '{}'", addr);

        tokio::spawn(async move {
            match acceptor.accept(stream).await {
                Ok(stream) => {
                    let namespace = match get_certificate_subject(&stream) {
                        Ok(s) => s.strip_prefix("CN=").unwrap_or(s.as_ref()).to_string(),
                        Err(e) => {
                            error!("Failed to get certificate subject from connection: '{}'", e);

                            return
                        }
                    };
                    let (stream_read, stream_write) = tokio::io::split(stream);
                    let mut framed_read = FramedRead::new(stream_read, LengthDelimitedCodec::new());
                    let framed_write = FramedWrite::new(stream_write, LengthDelimitedCodec::new());
                    let mut router = Router::new(framed_write, namespace);

                    loop {
                        let namespace = router.namespace();
                        info!("Waiting for commands from '{}' namespace", namespace);

                        match timeout(Duration::from_secs(120), framed_read.next()).await {
                            Ok(Some(Ok(x))) => {
                                match ActionCode::deserialize(&x) {
                                    Ok(op_code) => {
                                        if let ActionCode::Disconnect(client_name) = op_code {

                                            info!("Client disconnected: {}", client_name);
                                            break;
                                        }

                                        router.route(op_code).await;
                                    },
                                    Err(e) => {
                                        error!("Failed to deserialize opcode: {}", e);

                                        router.error(format!("Failed to deserialize opcode: {}", e)).await;
                                    }
                                }
                            }
                            Ok(Some(Err(e))) => {
                                let client_name = router.get_client_name();

                                warn!("For client: '{client_name}' connection error: {e}");
                                break;
                            }
                            Ok(None) => {
                                let client_name = router.get_client_name();

                                trace!("Client: '{client_name}' closed connection.");
                                break;
                            }
                            Err(elapsed) => {
                                let client_name = router.get_client_name().to_owned();
                                warn!("For client: '{client_name}' timeout: '{elapsed}' occurred.");

                                if router.test_connection().await.is_err() {
                                    info!("For client: '{client_name}' connection closed. Quit connection.");
                                    break;
                                }
                            }
                        }
                    }

                },
                Err(e) => {
                    error!("Failed from: {addr} to accept connection: {e}");
                }
            }
        });

        Ok(())
    }
}