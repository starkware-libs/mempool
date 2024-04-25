use std::panic;

use assert_matches::assert_matches;
use pretty_assertions::assert_eq;
use rstest::{fixture, rstest};
use starknet_api::core::{ContractAddress, PatriciaKey};
use starknet_api::hash::{StarkFelt, StarkHash};
use starknet_api::transaction::{Tip, TransactionHash};
use starknet_api::{contract_address, patricia_key};
use starknet_mempool_types::errors::MempoolError;
use starknet_mempool_types::mempool_types::{
    BatcherToMempoolChannels, BatcherToMempoolMessage, GatewayToMempoolMessage, MempoolInput,
    MempoolNetworkComponent, MempoolToBatcherMessage, MempoolToGatewayMessage, ThinTransaction,
};
use starknet_mempool_types::utils::create_thin_tx_for_testing;
use tokio::sync::mpsc::channel;

use crate::mempool::{Account, Mempool};

/// Creates a valid input for mempool's `add_tx` with optional default value for
/// `sender_address`.
/// Usage:
/// 1. add_tx_input!(tip, tx_hash, address)
/// 2. add_tx_input!(tip, tx_hash)
// TODO: Return MempoolInput once it's used in `add_tx`.
macro_rules! add_tx_input {
    ($tip:expr, $tx_hash:expr, $address:expr) => {{
        let account = Account { address: $address, ..Default::default() };
        let tx = create_thin_tx_for_testing($tip, $tx_hash, $address);
        (tx, account)
    }};
    ($tip:expr, $tx_hash:expr) => {
        add_tx_input!($tip, $tx_hash, ContractAddress::default())
    };
}

#[fixture]
fn mempool() -> Mempool {
    create_mempool_for_testing([])
}

#[test]
fn test_mempool_new_with_duplicate_txs() {
    let account = Account { address: contract_address!("0x0"), ..Default::default() };
    let tx = create_thin_tx_for_testing(Tip(50), TransactionHash(StarkFelt::ONE), account.address);
    let same_tx = tx.clone();

    let inputs = vec![
        MempoolInput { tx: tx.clone(), account },
        MempoolInput { tx: same_tx.clone(), account },
    ];

    let result = panic::catch_unwind(|| create_mempool_for_testing(inputs));
    assert!(result.is_err(), "Expected create_mempool_for_testing to panic, but it didn't.");
}

#[test]
fn test_mempool_new_with_fee_escalation() {
    let account = Account { address: contract_address!("0x0"), ..Default::default() };
    let tx_tip_50 =
        create_thin_tx_for_testing(Tip(50), TransactionHash(StarkFelt::ONE), account.address);
    let tx_tip_100 =
        create_thin_tx_for_testing(Tip(100), TransactionHash(StarkFelt::TWO), account.address);
    let tx_tip_70 =
        create_thin_tx_for_testing(Tip(70), TransactionHash(StarkFelt::THREE), account.address);

    let inputs = vec![
        MempoolInput { tx: tx_tip_50.clone(), account },
        MempoolInput { tx: tx_tip_100.clone(), account },
        MempoolInput { tx: tx_tip_70.clone(), account },
    ];

    let mut mempool = create_mempool_for_testing(inputs);

    assert_eq!(mempool.state.get(&account.address).unwrap().nonce, account.state.nonce);
    assert!(mempool.addresses_to_queues.contains_key(&account.address));
    assert_eq!(mempool.addresses_to_queues.get(&account.address).unwrap().0.len(), 1);
    assert_eq!(
        mempool.addresses_to_queues.get(&account.address).unwrap().0.first().unwrap(),
        &tx_tip_100
    );
    assert_eq!(mempool.txs_queue.len(), 1);
    assert_eq!(mempool.txs_queue.pop_last_chunk(1).first().unwrap(), &tx_tip_100);
}

#[rstest]
#[case(3)] // Requesting exactly the number of transactions in the queue
#[case(5)] // Requesting more transactions than are in the queue
#[case(2)] // Requesting fewer transactions than are in the queue
fn test_get_txs(#[case] requested_txs: usize) {
    let (tx_tip_50_address_0, account1) = add_tx_input!(Tip(50), TransactionHash(StarkFelt::ONE));
    let (tx_tip_100_address_1, account2) =
        add_tx_input!(Tip(100), TransactionHash(StarkFelt::TWO), contract_address!("0x1"));
    let (tx_tip_10_address_2, account3) =
        add_tx_input!(Tip(10), TransactionHash(StarkFelt::THREE), contract_address!("0x2"));

    let mut mempool = create_mempool_for_testing([
        MempoolInput { tx: tx_tip_50_address_0.clone(), account: account1 },
        MempoolInput { tx: tx_tip_100_address_1.clone(), account: account2 },
        MempoolInput { tx: tx_tip_10_address_2.clone(), account: account3 },
    ]);

    let expected_addresses =
        vec![contract_address!("0x0"), contract_address!("0x1"), contract_address!("0x2")];
    // checks that the transactions were added to the mempool.
    for address in &expected_addresses {
        assert!(mempool.state.contains_key(address));
    }

    let sorted_txs = vec![tx_tip_100_address_1, tx_tip_50_address_0, tx_tip_10_address_2];

    let txs = mempool.get_txs(requested_txs).unwrap();

    // This ensures we do not exceed the priority queue's limit of 3 transactions.
    let max_requested_txs = requested_txs.min(3);

    // checks that the returned transactions are the ones with the highest priority.
    assert_eq!(txs.len(), max_requested_txs);
    assert_eq!(txs, sorted_txs[..max_requested_txs].to_vec());

    // checks that the transactions that were not returned are still in the mempool.
    let actual_addresses: Vec<ContractAddress> =
        mempool.txs_queue.iter().map(|tx| tx.sender_address).collect();
    let expected_remaining_addresses: Vec<&ContractAddress> =
        expected_addresses[max_requested_txs..].iter().collect();
    for address in expected_remaining_addresses {
        assert!(actual_addresses.contains(address));
    }
}

#[rstest]
fn test_mempool_initialization_with_duplicate_tx(mut mempool: Mempool) {
    let (tx, account) = add_tx_input!(Tip(50), TransactionHash(StarkFelt::ONE));
    let same_tx = tx.clone();
    assert!(mempool.add_tx(tx, account).is_ok());
    assert!(mempool.add_tx(same_tx, account).is_err());
}

#[rstest]
fn test_add_tx(mut mempool: Mempool) {
    let (tx_tip_50_address_0, account1) = add_tx_input!(Tip(50), TransactionHash(StarkFelt::ONE));
    let (tx_tip_100_address_1, account2) =
        add_tx_input!(Tip(100), TransactionHash(StarkFelt::TWO), contract_address!("0x1"));
    let (tx_tip_80_address_2, account3) =
        add_tx_input!(Tip(80), TransactionHash(StarkFelt::THREE), contract_address!("0x2"));

    assert!(mempool.add_tx(tx_tip_50_address_0.clone(), account1).is_ok());
    assert!(mempool.add_tx(tx_tip_100_address_1.clone(), account2).is_ok());
    assert!(mempool.add_tx(tx_tip_80_address_2.clone(), account3).is_ok());

    assert_eq!(mempool.state.len(), 3);
    mempool.state.contains_key(&account1.address);
    mempool.state.contains_key(&account2.address);
    mempool.state.contains_key(&account3.address);

    let account_0_queue = mempool.addresses_to_queues.get(&account1.address).unwrap();
    let first_tx = account_0_queue.0.first().unwrap().clone();
    assert_eq!(tx_tip_50_address_0, first_tx);

    let account_1_queue = mempool.addresses_to_queues.get(&account2.address).unwrap();
    let first_tx = account_1_queue.0.first().unwrap().clone();
    assert_eq!(tx_tip_100_address_1, first_tx);

    let account_2_queue = mempool.addresses_to_queues.get(&account3.address).unwrap();
    let first_tx = account_2_queue.0.first().unwrap().clone();
    assert_eq!(tx_tip_80_address_2, first_tx);

    let account_0_queue = mempool.addresses_to_queues.get(&account1.address).unwrap();
    let first_tx = account_0_queue.0.first().unwrap().clone();
    assert_eq!(tx_tip_50_address_0, first_tx);

    let account_1_queue = mempool.addresses_to_queues.get(&account2.address).unwrap();
    let first_tx = account_1_queue.0.first().unwrap().clone();
    assert_eq!(tx_tip_100_address_1, first_tx);

    let account_2_queue = mempool.addresses_to_queues.get(&account3.address).unwrap();
    let first_tx = account_2_queue.0.first().unwrap().clone();
    assert_eq!(tx_tip_80_address_2, first_tx);

    check_mempool_txs_eq(
        &mempool,
        &[tx_tip_50_address_0, tx_tip_80_address_2, tx_tip_100_address_1],
    )
}

#[rstest]
fn test_add_same_tx(mut mempool: Mempool) {
    let (tx, account) = add_tx_input!(Tip(50), TransactionHash(StarkFelt::ONE));
    let same_tx = tx.clone();

    assert!(mempool.add_tx(tx.clone(), account).is_ok());
    assert_matches!(
        mempool.add_tx(same_tx, account),
        Err(MempoolError::DuplicateTransaction { tx_hash: TransactionHash(StarkFelt::ONE) })
    );
    // Assert that the original tx remains in the pool after the failed attempt.
    check_mempool_txs_eq(&mempool, &[tx])
}

// Asserts that the transactions in the mempool are in ascending order as per the expected
// transactions.
fn check_mempool_txs_eq(mempool: &Mempool, expected_txs: &[ThinTransaction]) {
    let mempool_txs = mempool.txs_queue.iter();
    // Deref the inner mempool tx type.
    expected_txs.iter().zip(mempool_txs).all(|(a, b)| *a == **b);
}

// TODO: remove network code once server abstraction is merged, then move into mempool with cfg.
fn create_mempool_for_testing(inputs: impl IntoIterator<Item = MempoolInput>) -> Mempool {
    let (_, rx_gateway_to_mempool) = channel::<GatewayToMempoolMessage>(1);
    let (tx_mempool_to_gateway, _) = channel::<MempoolToGatewayMessage>(1);
    let gateway_network =
        MempoolNetworkComponent::new(tx_mempool_to_gateway, rx_gateway_to_mempool);

    let (_, rx_mempool_to_batcher) = channel::<BatcherToMempoolMessage>(1);
    let (tx_batcher_to_mempool, _) = channel::<MempoolToBatcherMessage>(1);
    let batcher_network =
        BatcherToMempoolChannels { rx: rx_mempool_to_batcher, tx: tx_batcher_to_mempool };

    Mempool::new(inputs, gateway_network, batcher_network).unwrap()
}
