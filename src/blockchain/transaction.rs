use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Transaction {
    sender: String,
    recipient: String,
    value: u64,
}

impl Transaction {
    pub fn new(sender: &str, recipient: &str, value: u64) -> Transaction {
        Transaction {
            sender: sender.to_string(),
            recipient: recipient.to_string(),
            value,
        }
    }
}
