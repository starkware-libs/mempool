use crate::mempool::MempoolInput;
use crate::{
    errors::MempoolError,
    mempool::{AccountState, Mempool},
    priority_queue::PQTransaction,
};
use assert_matches::assert_matches;
use rstest::rstest;
use starknet_api::core::{ContractAddress, PatriciaKey};
use starknet_api::{
    data_availability::DataAvailabilityMode,
    transaction::{InvokeTransaction, InvokeTransactionV3, ResourceBounds, ResourceBoundsMapping},
};
use starknet_api::{
    hash::StarkFelt,
    internal_transaction::{InternalInvokeTransaction, InternalTransaction},
    transaction::{Tip, TransactionHash},
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
    let tx_tip_50_contract_address_0 = create_internal_invoke_tx_for_testing(
        Tip(50),
        TransactionHash(StarkFelt::ONE),
        account_state1.contract_address,
    );
    let account_state2 = AccountState {
        contract_address: ContractAddress(PatriciaKey::from(1u128)),
        ..Default::default()
    };
    let tx_tip_100_contract_address_1 = create_internal_invoke_tx_for_testing(
        Tip(100),
        TransactionHash(StarkFelt::TWO),
        account_state2.contract_address,
    );
    let account_state3 = AccountState {
        contract_address: ContractAddress(PatriciaKey::from(2u128)),
        ..Default::default()
    };
    let tx_tip_10_contract_address_2 = create_internal_invoke_tx_for_testing(
        Tip(10),
        TransactionHash(StarkFelt::THREE),
        account_state3.contract_address,
    );

    let mut mempool = Mempool::new(vec![
        MempoolInput {
            tx: tx_tip_50_contract_address_0.clone(),
            account_state: account_state1,
        },
        MempoolInput {
            tx: tx_tip_100_contract_address_1.clone(),
            account_state: account_state2,
        },
        MempoolInput {
            tx: tx_tip_10_contract_address_2.clone(),
            account_state: account_state3,
        },
    ]);

    let expected_addresses = vec![
        ContractAddress(PatriciaKey::from(0u128)),
        ContractAddress(PatriciaKey::from(1u128)),
        ContractAddress(PatriciaKey::from(2u128)),
    ];
    // checks that the transactions were added to the mempool.
    for address in &expected_addresses {
        assert!(mempool.state.contains_key(address));
    }

    let sorted_txs = vec![
        tx_tip_100_contract_address_1,
        tx_tip_50_contract_address_0,
        tx_tip_10_contract_address_2,
    ];

    let txs = mempool.get_txs(requested_txs).unwrap();

    // This ensures we do not exceed the priority queue's limit of 3 transactions.
    let max_requested_txs = requested_txs.min(3);

    // checks that the returned transactions are the ones with the highest priority.
    assert_eq!(txs.len(), max_requested_txs);
    assert_eq!(txs, sorted_txs[..max_requested_txs].to_vec());
    assert_eq!(txs, sorted_txs[..max_requested_txs].to_vec());

    // checks that the transactions that were not returned are still in the mempool.
    let actual_addresses: Vec<&ContractAddress> = mempool.state.keys().collect();
    let expected_remaining_addresses: Vec<&ContractAddress> =
        expected_addresses[max_requested_txs..].iter().collect();
    assert_eq!(actual_addresses, expected_remaining_addresses,);
}

#[test]
fn test_add_tx() {
    let account_state1 = AccountState {
        contract_address: ContractAddress(PatriciaKey::from(0u128)),
        ..Default::default()
    };
    let tx_tip_50_contract_address_0 = create_internal_invoke_tx_for_testing(
        Tip(50),
        TransactionHash(StarkFelt::ONE),
        account_state1.contract_address,
    );
    let account_state2 = AccountState {
        contract_address: ContractAddress(PatriciaKey::from(1u128)),
        ..Default::default()
    };
    let tx_tip_100_contract_address_1 = create_internal_invoke_tx_for_testing(
        Tip(100),
        TransactionHash(StarkFelt::TWO),
        account_state2.contract_address,
    );

    let mut mempool = Mempool::default();
    assert!(mempool
        .add_tx(tx_tip_50_contract_address_0.clone(), &account_state1)
        .is_ok());
    assert!(mempool
        .add_tx(tx_tip_100_contract_address_1.clone(), &account_state2)
        .is_ok());

    assert_eq!(mempool.state.len(), 2);
    mempool.state.contains_key(&account_state1.contract_address);
    mempool.state.contains_key(&account_state2.contract_address);

    assert_eq!(
        mempool.priority_queue.pop_last().unwrap(),
        PQTransaction(tx_tip_100_contract_address_1)
    );
    assert_eq!(
        mempool.priority_queue.pop_last().unwrap(),
        PQTransaction(tx_tip_50_contract_address_0)
    );
}

#[test]
fn test_add_same_tx() {
    let account_state = AccountState {
        contract_address: ContractAddress(PatriciaKey::from(0u128)),
        ..Default::default()
    };
    let tx = create_internal_invoke_tx_for_testing(
        Tip(50),
        TransactionHash(StarkFelt::ONE),
        ContractAddress(PatriciaKey::from(0u128)),
    );
    let same_tx = create_internal_invoke_tx_for_testing(
        Tip(50),
        TransactionHash(StarkFelt::ONE),
        ContractAddress(PatriciaKey::from(0u128)),
    );

    let mut mempool = Mempool::default();

    assert!(mempool.add_tx(tx, &account_state).is_ok());
    assert_matches!(
        mempool.add_tx(same_tx, &account_state),
        Err(MempoolError::DuplicateTransaction)
    );
}
