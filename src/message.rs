use serde::{Deserialize, Serialize};

const PROTOCOL_NAME: &str = "simple_bitcoin_protocol";
const MY_VERSION: &str = "0.1.0";

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Message {
    protocol: String,
    version: String,
    #[serde(flatten)]
    payload: Payload,
}

impl Message {
    pub fn new(protocol: String, version: String, payload: Payload) -> Message {
        Message {
            protocol,
            version,
            payload,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "msg_type")]
pub enum Payload {
    #[serde(rename = "0")]
    Add { field1: u8 },
    #[serde(rename = "1")]
    Remove,
    #[serde(rename = "2")]
    CoreList,
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
        let raw = r#"{"protocol": "simple_bitcoin_protocol", "version": "0.1.0", "msg_type": "0", "field1": 1}"#;
        let expected = Message::new(
            PROTOCOL_NAME.to_string(),
            MY_VERSION.to_string(),
            Payload::Add { field1: 1 },
        );
        let actual: Message = serde_json::from_str(raw).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_round_trip_message_add() {
        let message = Message::new(
            PROTOCOL_NAME.to_string(),
            MY_VERSION.to_string(),
            Payload::Add { field1: 1 },
        );

        let actual = serde_json::to_string(&message).unwrap();
        let actual: Message = serde_json::from_str(&actual[..]).unwrap();
        assert_eq!(actual, message);
    }
}
