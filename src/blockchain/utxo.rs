use crate::blockchain::transaction::{
    Address, NormalTransaction, Transaction, TransactionInput, TransactionOutput,
};
use anyhow::{bail, Result};
use chrono::Utc;
use log::debug;

pub struct UTXOManager {
    my_address: Address,
    transactions: Vec<(Transaction, usize)>,
    balance: u64,
}

impl UTXOManager {
    pub fn new(my_address: Address) -> UTXOManager {
        UTXOManager {
            my_address,
            transactions: vec![],
            balance: 0,
        }
    }

    pub fn get_balance(&self) -> u64 {
        self.balance
    }

    /// 与えられた transaction 群から UTXO を再計算する。
    pub fn refresh_utxos(&mut self, txs: &Vec<Transaction>) {
        let txs = self.extract_utxos(txs);
        self.transactions.clear();
        for tx in txs.into_iter() {
            self.put_utxo(tx);
        }
    }

    /// 与えられた Transaction 群の中から UTXO としてまだ利用可能なもののみを抽出する
    fn extract_utxos(&self, txs: &Vec<Transaction>) -> Vec<Transaction> {
        let mut outputs = vec![];
        let mut inputs = vec![];

        for tx in txs.iter() {
            for tx_out in tx.get_outputs() {
                let recipient = tx_out.get_recipient();
                if recipient == self.my_address {
                    outputs.push(tx.clone());
                }
            }
            for tx_in in tx.get_inputs() {
                let recipient = tx_in.get_recipient();
                if recipient == self.my_address {
                    inputs.push(tx.clone());
                }
            }
        }

        outputs
            .into_iter()
            .filter(|output| {
                inputs
                    .iter()
                    .find(|input| {
                        input
                            .get_inputs()
                            .iter()
                            .find(|i| output == i.get_transaction())
                            .is_some()
                    })
                    .is_none()
            })
            .collect::<Vec<_>>()
    }

    /// 与えられた transaction を UTXO として保存する。
    fn put_utxo(&mut self, tx: Transaction) {
        for (idx, output) in tx.get_outputs().iter().enumerate() {
            if output.get_recipient() == self.my_address {
                self.transactions.push((tx.clone(), idx));
            }
        }
        self.compute_my_balance();
    }

    fn compute_my_balance(&mut self) {
        let mut balance = 0;

        for (tx, idx) in self.transactions.iter() {
            balance += tx.get_output(*idx).unwrap().get_value();
        }

        self.balance = balance;
    }

    pub fn create_transaction_for(
        &mut self,
        recipient: Address,
        value: u64,
        fee: u64,
    ) -> Result<NormalTransaction> {
        if self.balance < value + fee {
            bail!("doesn't have enough coins");
        }

        let mut sum = 0;
        let input_txs = self.transactions.iter().fold(vec![], |mut acc, (tx, idx)| {
            if sum < value + fee {
                sum += tx.get_output(*idx).unwrap().get_value();
                acc.push(TransactionInput::new(tx.clone(), *idx));
                acc
            } else {
                acc
            }
        });

        let mut output_txs = vec![TransactionOutput::new(recipient, value)];
        if sum > value + fee {
            output_txs.push(TransactionOutput::new(
                self.my_address.clone(),
                sum - (value + fee),
            ));
        }

        let input_len = input_txs.len();
        let res = NormalTransaction::new(input_txs, output_txs, Utc::now());

        // drain used transactions
        self.transactions = self.transactions.drain(input_len..).collect();
        self.put_utxo(Transaction::Normal(res.clone()));

        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blockchain::transaction::{
        CoinbaseTransaction, NormalTransaction, TransactionInput, TransactionOutput,
    };
    use crate::key_manager::KeyManager;
    use chrono::{Duration, Utc};
    use rand::rngs::OsRng;

    #[test]
    fn test_refresh_utxos() {
        let rng = OsRng;

        let my_km = KeyManager::new(rng).unwrap();
        let mut my_um = UTXOManager::new(my_km.get_address());

        let km1 = KeyManager::new(rng).unwrap();

        let now = Utc::now();
        let sec = Duration::seconds(1);

        let tx1 = Transaction::Coinbase(CoinbaseTransaction::new(
            my_km.get_address(),
            2,
            now + sec * 0,
        ));
        let tx2 = Transaction::Coinbase(CoinbaseTransaction::new(
            my_km.get_address(),
            3,
            now + sec * 1,
        ));
        let tx3 = Transaction::Coinbase(CoinbaseTransaction::new(
            my_km.get_address(),
            4,
            now + sec * 2,
        ));

        let tx4 = Transaction::Normal(NormalTransaction::new(
            vec![TransactionInput::new(tx1.clone(), 0)],
            vec![
                TransactionOutput::new(km1.get_address(), 1),
                TransactionOutput::new(my_km.get_address(), 1),
            ],
            now,
        ));

        let txs = vec![tx1, tx2, tx3, tx4];
        my_um.refresh_utxos(&txs);

        assert_eq!(my_um.get_balance(), 8);
    }

    #[test]
    fn test_create_transaction_for() {
        let rng = OsRng;

        let my_km = KeyManager::new(rng).unwrap();
        let mut my_um = UTXOManager::new(my_km.get_address());

        let km1 = KeyManager::new(rng).unwrap();

        let now = Utc::now();
        let sec = Duration::seconds(1);

        let tx1 = Transaction::Coinbase(CoinbaseTransaction::new(
            my_km.get_address(),
            2,
            now + sec * 0,
        ));
        let tx2 = Transaction::Coinbase(CoinbaseTransaction::new(
            my_km.get_address(),
            3,
            now + sec * 1,
        ));
        let tx3 = Transaction::Coinbase(CoinbaseTransaction::new(
            my_km.get_address(),
            4,
            now + sec * 2,
        ));

        let txs = vec![tx1, tx2, tx3];
        my_um.refresh_utxos(&txs);

        let tx = my_um
            .create_transaction_for(km1.get_address(), 1, 1)
            .unwrap();

        assert_eq!(tx.get_input_value(), 2);
        assert_eq!(tx.get_output_value(), 1);
        assert_eq!(my_um.get_balance(), 7);

        let tx = my_um
            .create_transaction_for(km1.get_address(), 3, 1)
            .unwrap();

        assert_eq!(tx.get_input_value(), 7);
        assert_eq!(tx.get_output_value(), 6);
        assert_eq!(my_um.get_balance(), 3);
    }
}
