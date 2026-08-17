use futures_util::SinkExt;
use tokio::io::WriteHalf;
use tokio::net::TcpStream;
use tokio_rustls::{server};
use tokio_util::codec::{FramedWrite, LengthDelimitedCodec};
use tracing::error;
use common::op_codes::OpCode;

pub struct Router {
    response:  FramedWrite<WriteHalf<server::TlsStream<TcpStream>>, LengthDelimitedCodec>,
}

impl Router {
    
    pub fn new(response: FramedWrite<WriteHalf<server::TlsStream<TcpStream>>, LengthDelimitedCodec>) -> Self {
        Router {
            response
        }
    }
    
    pub async fn route(&mut self, op_code: OpCode) {
        match op_code {
            OpCode::KeepAlive(_) => { println!("KeepAlive"); }
            OpCode::TestConnection => { println!("TestConnection"); }
            OpCode::Connect(connect_details) => {
                println!("Connect: {:?}", connect_details);
            }
            OpCode::Disconnect(_) => {}
            OpCode::Send(send_details) => {
                println!("Send: {:?}", send_details);
            }
            OpCode::Confirmed(_) => {}
            OpCode::Receive(_) => {}
            OpCode::Commit(_) => {}
            OpCode::SetState(_, _) => {}
            OpCode::GetState(_) => {}
            OpCode::Error(_) => {}
        }
    }

    pub async fn error(&mut self, message: String) {
        match OpCode::Error(message).serialize() {
            Ok(serialized_message) => {
                _ = self.response.send(serialized_message.into()).await;
            }
            Err(e) => {
                error!("{}", e);
            }
        };
    }
}