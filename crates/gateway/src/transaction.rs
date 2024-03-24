use serde::{Deserialize, Serialize};
use starknet_api::transaction::{Transaction, TransactionHash};

#[derive(Deserialize, Serialize, Debug, Clone)]

pub struct InternalTransaction {
    transaction: Transaction,
    hash: TransactionHash,
}

impl InternalTransaction {
    pub fn new(transaction: Transaction, hash: TransactionHash) -> Self {
        InternalTransaction { transaction, hash }
    }
    pub fn get_transaction_hash(&self) -> &TransactionHash {
        &self.hash
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ExternalTransaction {
    transaction: Transaction,
}

impl ExternalTransaction {
    pub fn new(transaction: Transaction) -> Self {
        ExternalTransaction { transaction }
    }

    pub fn get_transaction_type(&self) -> String {
        match &self.transaction {
            Transaction::Declare(_) => "Declare".to_string(),
            Transaction::Deploy(_) => "Deploy".to_string(),
            Transaction::DeployAccount(_) => "DeployAccount".to_string(),
            Transaction::Invoke(_) => "Invoke".to_string(),
            Transaction::L1Handler(_) => "L1Handler".to_string(),
        }
    }

    pub fn get_transaction(&self) -> &Transaction {
        &self.transaction
    }
}
