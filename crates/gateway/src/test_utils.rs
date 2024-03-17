use starknet_api::core::ClassHash;
use starknet_api::core::ContractAddress;
use starknet_api::core::Nonce;
use starknet_api::hash::StarkFelt;
use starknet_api::transaction::{DeclareTransactionV0V1, Fee, TransactionSignature};

pub fn create_a_declare_tx() -> DeclareTransactionV0V1 {
    DeclareTransactionV0V1 {
        max_fee: Fee(0),
        signature: TransactionSignature::default(),
        nonce: Nonce(StarkFelt::from(1_u8)),
        class_hash: ClassHash(StarkFelt::from(1_u8)),
        sender_address: ContractAddress::try_from(StarkFelt::from(1_u8)).unwrap(),
    }
}
