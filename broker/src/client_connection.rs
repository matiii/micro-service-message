use std::time::Duration;
use bytes::Bytes;
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio::time::timeout;
use tokio_rustls::TlsAcceptor;
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};
use tracing::error;
use common::op_codes::OpCode;
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

        println!("New connection from {}", addr);

        tokio::spawn(async move {
            match acceptor.accept(stream).await {
                Ok(stream) => {
                    let test_connection_code: Bytes = OpCode::TestConnection.serialize().unwrap().into();
                    let (stream_read, stream_write) = tokio::io::split(stream);
                    let mut framed_read = FramedRead::new(stream_read, LengthDelimitedCodec::new());
                    let framed_write = FramedWrite::new(stream_write, LengthDelimitedCodec::new());
                    let mut router = Router::new(framed_write);

                    loop {
                        match timeout(Duration::from_secs(120), framed_read.next()).await {
                            Ok(Some(Ok(x))) => {
                                match OpCode::deserialize(&x) {
                                    Ok(op_code) => {
                                        router.route(op_code).await;
                                    },
                                    Err(e) => {
                                        error!("Failed to deserialize opcode: {}", e);

                                        router.error(format!("Failed to deserialize opcode: {}", e)).await;
                                    }
                                }
                            }
                            Ok(Some(Err(e))) => {
                                println!("Codec/IO error reading frame: {e}");
                                break;
                            }
                            Ok(None) => {
                                println!("Client closed connection.");
                                break;
                            }
                            Err(_elapsed) => {
                                println!("Timeout from message read.");
                                // if framed_write.send(test_connection_code.clone()).await.is_err() {
                                //     println!("Client connection closed. Quit connection.");
                                //     break;
                                // }
                            }
                        }
                    }

                },
                Err(e) => {
                    println!("Failed to accept connection: {}", e);
                }
            }
        });

        Ok(())
    }
}