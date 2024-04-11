use crate::mempool::MempoolInput;
use crate::{
    errors::MempoolError,
    mempool::{AccountState, Mempool},
    priority_queue::PQTransaction,
};
use assert_matches::assert_matches;
use rstest::rstest;
use starknet_api::{
    core::{ContractAddress, PatriciaKey},
    hash::StarkFelt,
    internal_transaction::{InternalInvokeTransaction, InternalTransaction},
    transaction::{Tip, TransactionHash},
};
use starknet_api::{
    data_availability::DataAvailabilityMode,
    transaction::{InvokeTransaction, InvokeTransactionV3, ResourceBounds, ResourceBoundsMapping},
};

// TODO(Ayelet): Move to StarkNet API.
pub fn create_internal_invoke_tx_for_testing(
    tip: Tip,
    tx_hash: TransactionHash,
    sender_address: ContractAddress,
) -> InternalTransaction {
    let tx = InvokeTransactionV3 {
        resource_bounds: ResourceBoundsMapping::try_from(vec![
            (
                starknet_api::transaction::Resource::L1Gas,
                ResourceBounds::default(),
            ),
            (
                starknet_api::transaction::Resource::L2Gas,
                ResourceBounds::default(),
            ),
        ])
        .unwrap(),
        signature: Default::default(),
        nonce: Default::default(),
        sender_address,
        calldata: Default::default(),
        nonce_data_availability_mode: DataAvailabilityMode::L1,
        fee_data_availability_mode: DataAvailabilityMode::L1,
        paymaster_data: Default::default(),
        account_deployment_data: Default::default(),
        tip,
    };

    InternalTransaction::Invoke(InternalInvokeTransaction {
        tx: InvokeTransaction::V3(tx),
        tx_hash,
        only_query: false,
    })
}

#[rstest]
#[case(3)] // Requesting exactly the number of transactions in the queue
#[case(5)] // Requesting more transactions than are in the queue
#[case(2)] // Requesting fewer transactions than are in the queue
fn test_get_txs(#[case] requested_txs: usize) {
    let account_state1 = AccountState {
        contract_address: ContractAddress(PatriciaKey::from(0u128)),
        ..Default::default()
    };
    let tx1 = create_internal_invoke_tx_for_testing(
        Tip(50),
        TransactionHash(StarkFelt::ONE),
        account_state1.contract_address,
    );
    let account_state2 = AccountState {
        contract_address: ContractAddress(PatriciaKey::from(1u128)),
        ..Default::default()
    };
    let tx2 = create_internal_invoke_tx_for_testing(
        Tip(100),
        TransactionHash(StarkFelt::TWO),
        account_state2.contract_address,
    );
    let account_state3 = AccountState {
        contract_address: ContractAddress(PatriciaKey::from(2u128)),
        ..Default::default()
    };
    let tx3 = create_internal_invoke_tx_for_testing(
        Tip(10),
        TransactionHash(StarkFelt::THREE),
        account_state3.contract_address,
    );

    let mut mempool = Mempool::new(vec![
        MempoolInput {
            tx: tx1.clone(),
            account_state: account_state1,
        },
        MempoolInput {
            tx: tx2.clone(),
            account_state: account_state2,
        },
        MempoolInput {
            tx: tx3.clone(),
            account_state: account_state3,
        },
    ]);

    let sorted_txs = vec![tx2, tx1, tx3];
    let txs = mempool.get_txs(requested_txs).unwrap();

    // This ensures we do not exceed the priority queue's limit of 3 transactions.
    let max_requested_txs = requested_txs.min(3);

    assert_eq!(txs.len(), max_requested_txs);
    assert_eq!(txs, sorted_txs[..max_requested_txs].to_vec());
}

#[test]
fn test_add_tx() {
    let account_state1 = AccountState {
        contract_address: ContractAddress(PatriciaKey::from(0u128)),
        ..Default::default()
    };
    let tx1 = create_internal_invoke_tx_for_testing(
        Tip(50),
        TransactionHash(StarkFelt::ONE),
        account_state1.contract_address,
    );
    let account_state2 = AccountState {
        contract_address: ContractAddress(PatriciaKey::from(1u128)),
        ..Default::default()
    };
    let tx2 = create_internal_invoke_tx_for_testing(
        Tip(100),
        TransactionHash(StarkFelt::TWO),
        account_state2.contract_address,
    );

    let mut mempool = Mempool::default();
    assert!(mempool.add_tx(tx1.clone(), &account_state1).is_ok());
    assert!(mempool.add_tx(tx2.clone(), &account_state2).is_ok());

    assert_eq!(mempool.state.len(), 2);
    mempool.state.contains_key(&account_state1.contract_address);
    mempool.state.contains_key(&account_state2.contract_address);

    assert_eq!(
        mempool.priority_queue.pop_last().unwrap(),
        PQTransaction(tx2)
    );
    assert_eq!(
        mempool.priority_queue.pop_last().unwrap(),
        PQTransaction(tx1)
    );
}

#[test]
fn test_add_same_tx() {
    let account_state = AccountState {
        contract_address: ContractAddress(PatriciaKey::from(0u128)),
        ..Default::default()
    };
    let tx1 = create_internal_invoke_tx_for_testing(
        Tip(50),
        TransactionHash(StarkFelt::ONE),
        ContractAddress(PatriciaKey::from(0u128)),
    );
    let tx2 = create_internal_invoke_tx_for_testing(
        Tip(50),
        TransactionHash(StarkFelt::ONE),
        ContractAddress(PatriciaKey::from(0u128)),
    );

    let mut mempool = Mempool::default();

    assert!(mempool.add_tx(tx1.clone(), &account_state).is_ok());
    assert_matches!(
        mempool.add_tx(tx2.clone(), &account_state),
        Err(MempoolError::DuplicateTransaction)
    );
}
