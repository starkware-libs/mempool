use assert_matches::assert_matches;
use itertools::zip_eq;
use pretty_assertions::assert_eq;
use rstest::{fixture, rstest};
use starknet_api::core::{ContractAddress, Nonce, PatriciaKey};
use starknet_api::hash::{StarkFelt, StarkHash};
use starknet_api::transaction::{Tip, TransactionHash};
use starknet_api::{contract_address, patricia_key};
use starknet_mempool_types::errors::MempoolError;
use starknet_mempool_types::mempool_types::{Account, MempoolInput, ThinTransaction};

use crate::mempool::Mempool;
use crate::priority_queue::PrioritizedTransaction;

/// Creates a valid input for mempool's `add_tx` with optional default value for
/// `sender_address`.
/// Usage:
/// 1. add_tx_input!(tip, tx_hash, address, nonce)
/// 2. add_tx_input!(tip, tx_hash, address)
/// 3. add_tx_input!(tip, tx_hash)
// TODO: Return MempoolInput once it's used in `add_tx`.
macro_rules! add_tx_input {
    // Pattern for all four arguments
    ($tip:expr, $tx_hash:expr, $sender_address:expr, $nonce:expr) => {{
        let account = Account { sender_address: $sender_address, ..Default::default() };
        let tx = ThinTransaction {
            tip: $tip,
            tx_hash: $tx_hash,
            sender_address: $sender_address,
            nonce: $nonce,
        };
        (tx, account)
    }};
    // Pattern for three arguments: tip, tx_hash, address
    ($tip:expr, $tx_hash:expr, $address:expr) => {
        add_tx_input!($tip, $tx_hash, $address, Nonce::default())
    };
    // Pattern for two arguments: tip, tx_hash
    ($tip:expr, $tx_hash:expr) => {
        add_tx_input!($tip, $tx_hash, ContractAddress::default(), Nonce::default())
    };
}

#[fixture]
fn mempool() -> Mempool {
    Mempool::empty()
}

#[test]
fn test_new_with_duplicate_tx() {
    let (tx, account) = add_tx_input!(Tip(0), TransactionHash(StarkFelt::ONE));
    let same_tx = tx.clone();

    let inputs = vec![MempoolInput { tx, account }, MempoolInput { tx: same_tx, account }];

    assert!(matches!(
        Mempool::new(inputs),
        Err(MempoolError::DuplicateTransaction { tx_hash: TransactionHash(StarkFelt::ONE) })
    ));
}

#[test]
fn test_new_success() {
    let (tx0, account0) =
        add_tx_input!(Tip(50), TransactionHash(StarkFelt::ZERO), contract_address!("0x0"));
    let (tx1, account1) =
        add_tx_input!(Tip(60), TransactionHash(StarkFelt::ONE), contract_address!("0x1"));
    let (tx3, _) = add_tx_input!(
        Tip(80),
        TransactionHash(StarkFelt::THREE),
        contract_address!("0x0"),
        Nonce(StarkFelt::ONE)
    );

    let inputs = vec![
        MempoolInput { tx: tx0.clone(), account: account0 },
        MempoolInput { tx: tx1.clone(), account: account1 },
        MempoolInput { tx: tx3.clone(), account: account0 },
    ];

    let mempool = Mempool::new(inputs).unwrap();

    assert!(mempool.address_to_store.get(&account0.sender_address).unwrap().contains(&tx0));
    assert!(mempool.address_to_store.get(&account1.sender_address).unwrap().contains(&tx1));
    assert!(mempool.address_to_store.get(&account0.sender_address).unwrap().contains(&tx3));

    assert!(mempool.tx_queue.contains(&PrioritizedTransaction(tx0)));
    assert!(mempool.tx_queue.contains(&PrioritizedTransaction(tx1)));
    assert!(!mempool.tx_queue.contains(&PrioritizedTransaction(tx3)));
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
    let (tx2_address_0, _) = add_tx_input!(
        Tip(50),
        TransactionHash(StarkFelt::ZERO),
        contract_address!("0x0"),
        Nonce(StarkFelt::ONE)
    );

    let inputs = [
        MempoolInput { tx: tx_tip_50_address_0.clone(), account: account1 },
        MempoolInput { tx: tx_tip_100_address_1.clone(), account: account2 },
        MempoolInput { tx: tx_tip_10_address_2.clone(), account: account3 },
        MempoolInput { tx: tx2_address_0.clone(), account: account1 },
    ];

    let mut mempool = Mempool::new(inputs).unwrap();

    assert!(mempool.tx_queue.contains(&PrioritizedTransaction(tx_tip_50_address_0.clone())));
    assert!(mempool.tx_queue.contains(&PrioritizedTransaction(tx_tip_100_address_1.clone())));
    assert!(mempool.tx_queue.contains(&PrioritizedTransaction(tx_tip_10_address_2.clone())));
    assert!(!mempool.tx_queue.contains(&PrioritizedTransaction(tx2_address_0.clone())));

    let sorted_txs = vec![tx_tip_100_address_1, tx_tip_50_address_0, tx_tip_10_address_2];

    let txs = mempool.get_txs(requested_txs).unwrap();

    // check that the account1's queue and the mempool's txs_queue are updated.
    assert!(
        mempool.address_to_store.get(&account1.sender_address).unwrap().contains(&tx2_address_0)
    );
    assert!(mempool.tx_queue.contains(&PrioritizedTransaction(tx2_address_0)));

    // This ensures we do not exceed the priority queue's limit of 3 transactions.
    let max_requested_txs = requested_txs.min(3);

    // checks that the returned transactions are the ones with the highest priority.
    assert_eq!(txs.len(), max_requested_txs);
    assert_eq!(txs, sorted_txs[..max_requested_txs].to_vec());

    // checks that the transactions that were not returned are still in the mempool.
    let expected_remaining_txs: Vec<&ThinTransaction> = txs[max_requested_txs..].iter().collect();
    for tx in expected_remaining_txs {
        assert!(mempool.tx_queue.contains(&PrioritizedTransaction(tx.clone())));
    }
}

#[rstest]
fn test_add_tx(mut mempool: Mempool) {
    let (tx_tip_50_address_0, account1) = add_tx_input!(Tip(50), TransactionHash(StarkFelt::ONE));
    let (tx_tip_100_address_1, account2) =
        add_tx_input!(Tip(100), TransactionHash(StarkFelt::TWO), contract_address!("0x1"));
    let (tx_tip_80_address_2, account3) =
        add_tx_input!(Tip(80), TransactionHash(StarkFelt::THREE), contract_address!("0x2"));
    let (tx2_address_0, _) = add_tx_input!(
        Tip(50),
        TransactionHash(StarkFelt::ZERO),
        contract_address!("0x0"),
        Nonce(StarkFelt::ONE)
    );

    assert_matches!(mempool.add_tx(tx_tip_50_address_0.clone()), Ok(()));
    assert_matches!(mempool.add_tx(tx_tip_100_address_1.clone()), Ok(()));
    assert_matches!(mempool.add_tx(tx_tip_80_address_2.clone()), Ok(()));
    assert_matches!(mempool.add_tx(tx2_address_0.clone()), Ok(()));

    assert_eq!(mempool.tx_queue.len(), 3);

    let account_0_queue = mempool.address_to_store.get(&account1.sender_address).unwrap();
    assert_eq!(&tx_tip_50_address_0, account_0_queue.top().unwrap());
    assert!(account_0_queue.contains(&tx2_address_0));

    let account_1_queue = mempool.address_to_store.get(&account2.sender_address).unwrap();
    assert_eq!(&tx_tip_100_address_1, account_1_queue.top().unwrap());

    let account_2_queue = mempool.address_to_store.get(&account3.sender_address).unwrap();
    assert_eq!(&tx_tip_80_address_2, account_2_queue.top().unwrap());

    check_mempool_txs_eq(
        &mempool,
        &[tx_tip_50_address_0, tx_tip_80_address_2, tx_tip_100_address_1],
    )
}

#[rstest]
fn test_add_tx_with_duplicate_tx(mut mempool: Mempool) {
    let (tx, _account) = add_tx_input!(Tip(50), TransactionHash(StarkFelt::ONE));
    let same_tx = tx.clone();

    assert_matches!(mempool.add_tx(tx.clone()), Ok(()));
    assert_matches!(
        mempool.add_tx(same_tx),
        Err(MempoolError::DuplicateTransaction { tx_hash: TransactionHash(StarkFelt::ONE) })
    );
    // Assert that the original tx remains in the pool after the failed attempt.
    check_mempool_txs_eq(&mempool, &[tx])
}

// Asserts that the transactions in the mempool are in ascending order as per the expected
// transactions.
#[track_caller]
fn check_mempool_txs_eq(mempool: &Mempool, expected_txs: &[ThinTransaction]) {
    let mempool_txs = mempool.tx_queue.iter();

    assert!(
        zip_eq(expected_txs, mempool_txs)
            // Deref the inner mempool tx type.
            .all(|(expected_tx, mempool_tx)| *expected_tx == **mempool_tx)
    );
}

#[rstest]
fn test_add_tx_with_identical_tip_succeeds(mut mempool: Mempool) {
    let (tx1, _account1) = add_tx_input!(Tip(1), TransactionHash(StarkFelt::TWO));

    // Create a transaction with identical tip, it should be allowed through since the priority
    // queue tie-breaks identical tips by other tx-unique identifiers (for example tx hash).
    let (tx2, _account2) =
        add_tx_input!(Tip(1), TransactionHash(StarkFelt::ONE), contract_address!("0x1"));

    assert!(mempool.add_tx(tx1.clone()).is_ok());
    assert!(mempool.add_tx(tx2.clone()).is_ok());

    // TODO: currently hash comparison tie-breaks the two. Once more robust tie-breaks are added
    // replace this assertion with a dedicated test.
    check_mempool_txs_eq(&mempool, &[tx2, tx1]);
}

#[rstest]
fn test_tip_priority_over_tx_hash(mut mempool: Mempool) {
    let (tx_big_tip_small_hash, _account1) = add_tx_input!(Tip(2), TransactionHash(StarkFelt::ONE));

    // Create a transaction with identical tip, it should be allowed through since the priority
    // queue tie-breaks identical tips by other tx-unique identifiers (for example tx hash).
    let (tx_small_tip_big_hash, _account2) =
        add_tx_input!(Tip(1), TransactionHash(StarkFelt::TWO), contract_address!("0x1"));

    assert!(mempool.add_tx(tx_big_tip_small_hash.clone()).is_ok());
    assert!(mempool.add_tx(tx_small_tip_big_hash.clone()).is_ok());
    check_mempool_txs_eq(&mempool, &[tx_small_tip_big_hash, tx_big_tip_small_hash])
}
