use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub type Address = String;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct TransactionInput {
    transaction: Transaction,
    index: usize,
}

impl TransactionInput {
    pub fn new(transaction: Transaction, index: usize) -> TransactionInput {
        TransactionInput { transaction, index }
    }

    pub fn get_transaction(&self) -> &Transaction {
        &self.transaction
    }

    pub fn get_recipient(&self) -> Address {
        self.transaction
            .get_outputs()
            .iter()
            .nth(self.index)
            .unwrap()
            .recipient
            .clone()
    }

    pub fn get_value(&self) -> u64 {
        self.transaction
            .get_outputs()
            .iter()
            .nth(self.index)
            .unwrap()
            .value
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct TransactionOutput {
    recipient: Address,
    value: u64,
}

impl TransactionOutput {
    pub fn new(recipient: Address, value: u64) -> TransactionOutput {
        TransactionOutput { recipient, value }
    }

    pub fn get_recipient(&self) -> Address {
        self.recipient.clone()
    }

    pub fn get_value(&self) -> u64 {
        self.value
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(tag = "tx_type")]
pub enum Transaction {
    #[serde(rename = "0")]
    Coinbase(CoinbaseTransaction),
    #[serde(rename = "1")]
    Normal(NormalTransaction),
}

impl Transaction {
    pub fn get_input(&self, idx: usize) -> Option<TransactionInput> {
        match self {
            Transaction::Coinbase(_) => None,
            Transaction::Normal(tx) => tx.inputs.iter().nth(idx).cloned(),
        }
    }

    pub fn get_inputs(&self) -> Vec<TransactionInput> {
        match self {
            Transaction::Coinbase(_) => vec![],
            Transaction::Normal(tx) => tx.inputs.clone(),
        }
    }

    pub fn get_output(&self, idx: usize) -> Option<TransactionOutput> {
        match self {
            Transaction::Coinbase(tx) if idx == 0 => {
                Some(TransactionOutput::new(tx.recipient.clone(), tx.value))
            }
            Transaction::Normal(tx) => tx.outputs.iter().nth(idx).cloned(),
            _ => None,
        }
    }

    pub fn get_outputs(&self) -> Vec<TransactionOutput> {
        match self {
            Transaction::Coinbase(tx) => {
                vec![TransactionOutput::new(tx.recipient.clone(), tx.value)]
            }
            Transaction::Normal(tx) => tx.outputs.clone(),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CoinbaseTransaction {
    recipient: Address,
    value: u64,
    timestamp: DateTime<Utc>,
}

impl CoinbaseTransaction {
    pub fn new(recipient: Address, value: u64, timestamp: DateTime<Utc>) -> CoinbaseTransaction {
        CoinbaseTransaction {
            recipient,
            value,
            timestamp,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct NormalTransaction {
    inputs: Vec<TransactionInput>,
    outputs: Vec<TransactionOutput>,
    timestamp: DateTime<Utc>,
}

impl NormalTransaction {
    pub fn new(
        inputs: Vec<TransactionInput>,
        outputs: Vec<TransactionOutput>,
        timestamp: DateTime<Utc>,
    ) -> NormalTransaction {
        NormalTransaction {
            inputs,
            outputs,
            timestamp,
        }
    }

    pub fn get_input_value(&self) -> u64 {
        self.inputs
            .iter()
            .fold(0, |acc, input| acc + input.get_value())
    }

    pub fn get_output_value(&self) -> u64 {
        self.outputs
            .iter()
            .fold(0, |acc, output| acc + output.get_value())
    }
}

/// ブロック内の transaction リストを表現する。
/// 最初の transaction が coinbase, 以降は normal であることを保証する。
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Transactions(Vec<Transaction>);

impl Transactions {
    pub fn new(head: CoinbaseTransaction, tail: Vec<NormalTransaction>) -> Transactions {
        let mut txs = vec![Transaction::Coinbase(head)];
        txs.extend(tail.into_iter().map(Transaction::Normal));
        Transactions(txs)
    }

    pub fn get_transaction_at(&self, idx: usize) -> Option<Transaction> {
        self.0.iter().cloned().nth(idx)
    }

    pub fn get_transactions(&self) -> Vec<Transaction> {
        self.0.clone()
    }

    pub fn get_normal_transactions(&self) -> Vec<NormalTransaction> {
        self.0
            .iter()
            .cloned()
            .filter_map(|x| {
                if let Transaction::Normal(tx) = x {
                    Some(tx)
                } else {
                    None
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_deserialize_transactions() {
        let now: DateTime<Utc> = DateTime::from_str("2022-03-09T12:00:00Z").unwrap();

        let json = r#"
            [
              {
                "tx_type": "0",
                "recipient": "alice",
                "value": 10,
                "timestamp": "2022-03-09T12:00:00Z"
              },
              {
                "tx_type": "1",
                "inputs": [
                  {
                    "transaction": {
                      "tx_type": "0",
                      "recipient": "alice",
                      "value": 10,
                      "timestamp": "2022-03-09T12:00:00Z"
                    },
                    "index": 0
                  }
                ],
                "outputs": [
                  {
                    "recipient": "bob",
                    "value": 10
                  }
                ],
                "timestamp": "2022-03-09T12:00:00Z"
              }
            ]
        "#;

        let actual: Transactions =
            serde_json::from_str(json).expect("failed to parse transactions");

        let expected = Transactions::new(
            CoinbaseTransaction::new("alice".to_string(), 10, now),
            vec![NormalTransaction::new(
                vec![TransactionInput::new(
                    Transaction::Coinbase(CoinbaseTransaction::new("alice".to_string(), 10, now)),
                    0,
                )],
                vec![TransactionOutput::new("bob".to_string(), 10)],
                now,
            )],
        );

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_round_trip_transactions() {
        let now: DateTime<Utc> = DateTime::from_str("2022-03-09T12:00:00Z").unwrap();

        let txs = Transactions::new(
            CoinbaseTransaction::new("alice".to_string(), 10, Utc::now()),
            vec![NormalTransaction::new(
                vec![TransactionInput::new(
                    Transaction::Coinbase(CoinbaseTransaction::new("alice".to_string(), 10, now)),
                    0,
                )],
                vec![TransactionOutput::new("bob".to_string(), 10)],
                Utc::now(),
            )],
        );

        let json = serde_json::to_string(&txs).expect("failed to serialize transactions");
        let actual: Transactions =
            serde_json::from_str(&json).expect("failed to parse transactions");

        assert_eq!(actual, txs);
    }
}
