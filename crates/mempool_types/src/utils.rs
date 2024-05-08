use starknet_api::transaction::TransactionHash;
use starknet_api::{core::ContractAddress, transaction::Tip};
use crate::mempool_types::ThinTransaction;

pub fn create_thin_tx_for_testing(
    tip: Tip,
    tx_hash: TransactionHash,
    contract_address: ContractAddress,
) -> ThinTransaction {
    ThinTransaction {
        contract_address,
        tx_hash,
        tip,
        nonce: Default::default(),
    }
}
