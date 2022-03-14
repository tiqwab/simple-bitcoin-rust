use crate::util;
use chrono::{DateTime, Utc};
use rsa::pkcs1::FromRsaPublicKey;
use rsa::RsaPublicKey;
use serde::{Deserialize, Serialize};

pub type Address = String;
pub type TransactionSignature = String;

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

    pub fn get_value(&self) -> u64 {
        self.value
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

    pub fn get_input_pubkey(&self) -> RsaPublicKey {
        let recipient = self.inputs.first().unwrap().get_recipient();
        RsaPublicKey::from_pkcs1_der(&util::hex_to_bytes(recipient)).unwrap()
    }

    pub fn get_input(&self, idx: usize) -> Option<TransactionInput> {
        self.inputs.iter().nth(idx).cloned()
    }

    pub fn get_inputs(&self) -> Vec<TransactionInput> {
        self.inputs.clone()
    }

    pub fn get_output(&self, idx: usize) -> Option<TransactionOutput> {
        self.outputs.iter().nth(idx).cloned()
    }

    pub fn get_outputs(&self) -> Vec<TransactionOutput> {
        self.outputs.clone()
    }
}

/// ブロック内の transaction リストを表現する。
/// 最初の transaction が coinbase, 以降は normal であることを保証する。
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct Transactions {
    coinbase: CoinbaseTransaction,
    transactions: Vec<NormalTransaction>,
}

impl Transactions {
    pub fn new(
        coinbase: CoinbaseTransaction,
        transactions: Vec<NormalTransaction>,
    ) -> Transactions {
        Transactions {
            coinbase,
            transactions,
        }
    }

    pub fn get_transaction_at(&self, idx: usize) -> Option<Transaction> {
        if idx == 0 {
            Some(Transaction::Coinbase(self.coinbase.clone()))
        } else {
            self.transactions
                .iter()
                .cloned()
                .nth(idx - 1)
                .map(Transaction::Normal)
        }
    }

    pub fn get_transactions(&self) -> Vec<Transaction> {
        vec![Transaction::Coinbase(self.coinbase.clone())]
            .into_iter()
            .chain(self.transactions.iter().cloned().map(Transaction::Normal))
            .collect()
    }

    pub fn get_coinbase_transaction(&self) -> CoinbaseTransaction {
        self.coinbase.clone()
    }

    pub fn get_normal_transactions(&self) -> Vec<NormalTransaction> {
        self.transactions.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    use std::str::FromStr;

    #[test]
    fn test_deserialize_transactions() {
        let now: DateTime<Utc> = DateTime::from_str("2022-03-09T12:00:00Z").unwrap();

        let json = r#"
            {
              "coinbase": {
                "recipient": "alice",
                "value": 10,
                "timestamp": "2022-03-09T12:00:00Z"
              },
              "transactions": [
                {
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
            }
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

        let json = serde_json::to_string(&txs).expect("failed to serialize transactions");
        let actual: Transactions =
            serde_json::from_str(&json).expect("failed to parse transactions");

        assert_eq!(actual, txs);
    }

    fn generate_sample() -> (CoinbaseTransaction, NormalTransaction, Transactions) {
        let now: DateTime<Utc> = DateTime::from_str("2022-03-09T12:00:00Z").unwrap();
        let sec = Duration::seconds(1);

        let tx1 = CoinbaseTransaction::new("alice".to_string(), 10, now + sec * 0);
        let tx2 = NormalTransaction::new(
            vec![TransactionInput::new(
                Transaction::Coinbase(CoinbaseTransaction::new(
                    "alice".to_string(),
                    10,
                    now + sec * 1,
                )),
                0,
            )],
            vec![TransactionOutput::new("bob".to_string(), 10)],
            Utc::now(),
        );
        let txs = Transactions::new(tx1.clone(), vec![tx2.clone()]);
        (tx1, tx2, txs)
    }

    #[test]
    fn test_get_transaction_at() {
        let (tx1, tx2, txs) = generate_sample();
        assert_eq!(
            txs.get_transaction_at(0),
            Some(Transaction::Coinbase(tx1.clone()))
        );
        assert_eq!(
            txs.get_transaction_at(1),
            Some(Transaction::Normal(tx2.clone()))
        );
        assert_eq!(txs.get_transaction_at(2), None);
    }

    #[test]
    fn test_get_transactions() {
        let (tx1, tx2, txs) = generate_sample();
        assert_eq!(
            txs.get_transactions(),
            vec![Transaction::Coinbase(tx1), Transaction::Normal(tx2)]
        );
    }

    #[test]
    fn test_get_normal_transactions() {
        let (tx1, tx2, txs) = generate_sample();
        assert_eq!(txs.get_normal_transactions(), vec![tx2]);
    }
}
