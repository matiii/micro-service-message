use anyhow::Context;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum OpCode {
    KeepAlive(ClientName),
    TestConnection,
    Connect(ConnectDetails),
    Disconnect(ClientName),
    Send(SendDetails),
    Confirmed(Uuid),
    Receive(ReceiveDetails),
    Commit(Uuid),
    SetState(QueueName, QueueState),
    GetState(QueueName),
}

impl OpCode {

    pub fn serialize(&self) -> anyhow::Result<Vec<u8>> {
        postcard::to_allocvec(&self)
            .with_context(|| format!("Failed to serialize OpCode: {:?}", self))
    }

    pub fn deserialize(bytes: &[u8]) -> anyhow::Result<Self> {
        postcard::from_bytes(&bytes)
        .with_context(|| format!("Failed to deserialize OpCode: {:?}", bytes))
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct SendDetails {
    queue: String,
    id: Uuid,
    session_id: Option<String>,
    header: Vec<(String, String)>,
    payload: String,
    payload_columns: Vec<String>,
    weight: u8,
    version: u8,
}

impl SendDetails {
    pub fn new(queue: String, payload: String, weight: u8, version: u8, header: Vec<(String, String)>, payload_columns: Vec<String>, session_id: Option<String>) -> Self {
        SendDetails{
            queue,
            id: Uuid::new_v4(),
            session_id,
            header,
            payload,
            payload_columns,
            weight,
            version,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct ConnectDetails {
    unique_name: String,
    subscriptions: Vec<String>,
    capacity: u32,
}

impl ConnectDetails {
    pub fn new(unique_name: String, subscriptions: Vec<String>, capacity: u32) -> Self {
        ConnectDetails {
            unique_name,
            subscriptions,
            capacity,
        }
    }

    pub fn unique_name(&self) -> &str {
        &self.unique_name
    }

    pub fn subscriptions(&self) -> &Vec<String> {
        &self.subscriptions
    }

    pub fn capacity(&self) -> u32 {
        self.capacity
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct ReceiveDetails {
    queue: String,
    id: Uuid,
    payload: String,
    version: u8,
}

pub type ClientName = String;
pub type QueueState = String;
pub type QueueName = String;