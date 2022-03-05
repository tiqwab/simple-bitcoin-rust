use crate::blockchain::block::Block;
use crate::blockchain::transaction::Transaction;
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

/// ネットワークで実装されるアプリケーション用のペイロード
/// ServerCore や ClientCore で処理される
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "msg_type")]
pub enum ApplicationPayload {
    #[serde(rename = "0")]
    NewTransaction { transaction: Transaction },
    #[serde(rename = "1")]
    NewBlock { block: Block },
    #[serde(rename = "2")]
    RequestFullChain,
    #[serde(rename = "3")]
    FullChain { chain: Vec<Block> },
    #[serde(rename = "4")]
    Enhanced { data: Vec<u8> },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "msg_type")]
pub enum Payload {
    #[serde(rename = "0")]
    Add,
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
    #[serde(rename = "7")]
    Application { payload: ApplicationPayload },
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_deserialize_message_core_list() {
        let raw = r#"{
          "protocol": "simple_bitcoin_protocol",
          "version": "0.1.0",
          "port": 12345,
          "msg_type": "2",
          "nodes": ["127.0.0.1:12345"]
        }"#;
        let expected = Message::new(
            12345,
            Payload::CoreList {
                nodes: vec![SocketAddr::from_str("127.0.0.1:12345").unwrap()],
            },
        );
        let actual: Message = serde_json::from_str(raw).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_round_trip_message_core_list() {
        let message = Message::new(
            12345,
            Payload::CoreList {
                nodes: vec![SocketAddr::from_str("127.0.0.1:12345").unwrap()],
            },
        );

        let actual = serde_json::to_string(&message).unwrap();
        let actual: Message = serde_json::from_str(&actual[..]).unwrap();
        assert_eq!(actual, message);
    }

    #[test]
    fn test_deserialize_message_application_enhanced() {
        let raw = r#"{
          "protocol": "simple_bitcoin_protocol",
          "version": "0.1.0",
          "port": 12345,
          "msg_type": "7",
          "payload": {
            "msg_type": "4",
            "data": [104, 101, 108, 108, 111]
          }
        }"#;
        let expected = Message::new(
            12345,
            Payload::Application {
                payload: ApplicationPayload::Enhanced {
                    data: "hello".as_bytes().to_owned(),
                },
            },
        );
        let actual: Message = serde_json::from_str(raw).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_round_trip_message_core_application_enhanced() {
        let message = Message::new(
            12345,
            Payload::Application {
                payload: ApplicationPayload::Enhanced {
                    data: "hello".as_bytes().to_owned(),
                },
            },
        );

        let actual = serde_json::to_string(&message).unwrap();
        let actual: Message = serde_json::from_str(&actual[..]).unwrap();
        assert_eq!(actual, message);
    }
}
