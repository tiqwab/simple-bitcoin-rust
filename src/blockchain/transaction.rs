use serde::{Deserialize, Serialize};

type TransactionHash = String;
type Address = String;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct TransactionInput {
    transaction: TransactionHash,
    index: usize,
}

impl TransactionInput {
    pub fn new(transaction: TransactionHash, index: usize) -> TransactionInput {
        TransactionInput { transaction, index }
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
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(tag = "tx_type")]
pub enum Transaction {
    #[serde(rename = "0")]
    Coinbase(CoinbaseTransaction),
    #[serde(rename = "1")]
    Normal(NormalTransaction),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct CoinbaseTransaction {
    recipient: Address,
    value: u64,
}

impl CoinbaseTransaction {
    pub fn new(recipient: Address, value: u64) -> CoinbaseTransaction {
        CoinbaseTransaction { recipient, value }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct NormalTransaction {
    inputs: Vec<TransactionInput>,
    outputs: Vec<TransactionOutput>,
}

impl NormalTransaction {
    pub fn new(
        inputs: Vec<TransactionInput>,
        outputs: Vec<TransactionOutput>,
    ) -> NormalTransaction {
        NormalTransaction { inputs, outputs }
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

    #[test]
    fn test_deserialize_transactions() {
        let json = r#"
            [
              {
                "tx_type": "0",
                "recipient": "alice",
                "value": 10
              },
              {
                "tx_type": "1",
                "inputs": [
                  {
                    "transaction": "alice",
                    "index": 0
                  }
                ],
                "outputs": [
                  {
                    "recipient": "bob",
                    "value": 10
                  }
                ]
              }
            ]
        "#;

        let actual: Transactions =
            serde_json::from_str(json).expect("failed to parse transactions");

        let expected = Transactions::new(
            CoinbaseTransaction::new("alice".to_string(), 10),
            vec![NormalTransaction::new(
                vec![TransactionInput::new("alice".to_string(), 0)],
                vec![TransactionOutput::new("bob".to_string(), 10)],
            )],
        );

        assert_eq!(actual, expected);
    }

    #[test]
    fn test_round_trip_transactions() {
        let txs = Transactions::new(
            CoinbaseTransaction::new("alice".to_string(), 10),
            vec![NormalTransaction::new(
                vec![TransactionInput::new("alice".to_string(), 0)],
                vec![TransactionOutput::new("bob".to_string(), 10)],
            )],
        );

        let json = serde_json::to_string(&txs).expect("failed to serialize transactions");
        let actual: Transactions =
            serde_json::from_str(&json).expect("failed to parse transactions");

        assert_eq!(actual, txs);
    }
}
