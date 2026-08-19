use std::sync::OnceLock;
use bytes::Bytes;
use futures_util::SinkExt;
use tokio::io::WriteHalf;
use tokio::net::TcpStream;
use tokio_rustls::{server};
use tokio_util::codec::{FramedWrite, LengthDelimitedCodec};
use tracing::{error, trace, warn};
use common::action_codes::{ConnectDetails, ActionCode};

static TEST_CONNECTION_CODE: OnceLock<Bytes> = OnceLock::new();


pub struct Router {
    response:  FramedWrite<WriteHalf<server::TlsStream<TcpStream>>, LengthDelimitedCodec>,
    namespace: String,
    connect_details: Option<ConnectDetails>,
}

impl Router {
    
    pub fn new(response: FramedWrite<WriteHalf<server::TlsStream<TcpStream>>, LengthDelimitedCodec>, namespace: String) -> Self {
        Router {
            response,
            namespace,
            connect_details: None,
        }
    }
    
    pub async fn route(&mut self, op_code: ActionCode) {
        match op_code {
            ActionCode::KeepAlive(client_name) => { trace!("KeepAlive from client: {}", client_name);  }
            ActionCode::TestConnection => {
                let client_name = self.get_client_name();

                trace!("TestConnection from client: {}", client_name);
            }
            ActionCode::Connect(connect_details) => {
                if self.connect_details.is_none() {
                    self.connect_details = Some(connect_details);
                    trace!("Connect {:?}", self.connect_details);
                } else {
                    warn!("Cannot change connection details: {:?}", connect_details);
                }
            }
            ActionCode::Disconnect(_) => {}
            ActionCode::Send(send_details) => {
                println!("Send: {:?}", send_details);
            }
            ActionCode::Confirmed(_) => {}
            ActionCode::Receive(_) => {}
            ActionCode::Commit(_) => {}
            ActionCode::SetState(_, _) => {}
            ActionCode::GetState(_) => {}
            ActionCode::Error(_) => {}
        }
    }

    pub async fn error(&mut self, message: String) {
        let client_name = self.get_client_name();

        trace!("Error for client name: '{}' message: '{}'", client_name, message);

        match ActionCode::Error(message).serialize() {
            Ok(serialized_message) => {
                _ = self.response.send(serialized_message.into()).await;
            }
            Err(e) => {
                error!("Client name: '{}' Error: '{}'", client_name, e);
            }
        };
    }

    pub async fn test_connection(&mut self) -> std::io::Result<()> {
        let test_connection = TEST_CONNECTION_CODE.get_or_init(|| ActionCode::TestConnection.serialize().unwrap().into());

        self.response.send(test_connection.clone()).await
    }

    pub fn get_client_name(&self) -> &str {
        self.connect_details.as_ref().map_or("", |x| x.unique_name().as_ref())
    }
}