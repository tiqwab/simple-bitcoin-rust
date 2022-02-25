use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

const PROTOCOL_NAME: &str = "simple_bitcoin_protocol";
const MY_VERSION: &str = "0.1.0";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Message {
    protocol: String,
    version: String,
    pub port: u16,
    #[serde(flatten)]
    pub payload: Payload,
}

impl Message {
    pub fn new(port: u16, payload: Payload) -> Message {
        Message {
            protocol: PROTOCOL_NAME.to_string(),
            version: MY_VERSION.to_string(),
            port,
            payload,
        }
    }

    pub fn new_with_proto_version(
        protocol: String,
        version: String,
        port: u16,
        payload: Payload,
    ) -> Message {
        Message {
            protocol,
            version,
            port,
            payload,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "msg_type")]
pub enum Payload {
    // TODO: remove field1
    #[serde(rename = "0")]
    Add { field1: u8 },
    #[serde(rename = "1")]
    Remove,
    #[serde(rename = "2")]
    CoreList { nodes: Vec<SocketAddr> },
    #[serde(rename = "3")]
    RequestCoreList,
    #[serde(rename = "4")]
    Ping,
    #[serde(rename = "5")]
    AddAsEdge,
    #[serde(rename = "6")]
    RemoveEdge,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_message_add() {
        let raw = r#"{
          "protocol": "simple_bitcoin_protocol",
          "version": "0.1.0",
          "port": 12345,
          "msg_type": "0",
          "field1": 1
        }"#;
        let expected = Message::new(12345, Payload::Add { field1: 1 });
        let actual: Message = serde_json::from_str(raw).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_round_trip_message_add() {
        let message = Message::new(12345, Payload::Add { field1: 1 });

        let actual = serde_json::to_string(&message).unwrap();
        let actual: Message = serde_json::from_str(&actual[..]).unwrap();
        assert_eq!(actual, message);
    }
}
