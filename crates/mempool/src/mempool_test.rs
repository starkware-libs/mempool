use rstest::rstest;
use starknet_api::{
    hash::StarkFelt,
    internal_transaction::{InternalInvokeTransaction, InternalTransaction},
    transaction::{Tip, TransactionHash},
};

use crate::{mempool::Mempool, priority_queue::PriorityQueue};

use starknet_api::{
    data_availability::DataAvailabilityMode,
    transaction::{InvokeTransaction, InvokeTransactionV3, ResourceBounds, ResourceBoundsMapping},
};

// TODO(Ayelet): Move to StarkNet API.
pub fn create_internal_invoke_tx_for_testing(
    tip: Tip,
    tx_hash: TransactionHash,
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
        sender_address: Default::default(),
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
    let tx1 = create_internal_invoke_tx_for_testing(Tip(50), TransactionHash(StarkFelt::ONE));
    let tx2 = create_internal_invoke_tx_for_testing(Tip(100), TransactionHash(StarkFelt::TWO));
    let tx3 = create_internal_invoke_tx_for_testing(Tip(10), TransactionHash(StarkFelt::THREE));

    // TODO(Ayelet): Change to add_txs when implemented.
    let mut priority_queue = PriorityQueue::default();
    priority_queue.push(tx1.clone());
    priority_queue.push(tx2.clone());
    priority_queue.push(tx3.clone());
    let mut mempool = Mempool { priority_queue };

    let txs = mempool.get_txs(requested_txs).unwrap();

    // This ensures we do not exceed the priority queue's limit of 3 transactions.
    let max_requested_txs = requested_txs.min(3);

    let sorted_txs = vec![tx2, tx1, tx3];
    assert_eq!(txs.len(), max_requested_txs);
    assert_eq!(txs, sorted_txs[..max_requested_txs])
}
