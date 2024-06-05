use starknet_api::core::{ContractAddress, Nonce};
use starknet_api::transaction::{Tip, TransactionHash};

use crate::mempool_types::ThinTransaction;

pub fn create_thin_tx_for_testing(
    tip: Tip,
    tx_hash: TransactionHash,
    sender_address: ContractAddress,
    nonce: Nonce,
) -> ThinTransaction {
    ThinTransaction { sender_address, tx_hash, tip, nonce }
}
