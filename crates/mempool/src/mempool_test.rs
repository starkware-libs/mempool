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
        .expect("Resource bounds mapping has unexpected structure."),
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
#[case(3, 3)] // Requesting exactly the number of transactions in the queue
#[case(5, 3)] // Requesting more transactions than are in the queue
#[case(2, 2)] // Requesting fewer transactions than are in the queue
fn test_get_txs(#[case] requested_txs: usize, #[case] expected_txs: usize) {
    let tx1 = create_internal_invoke_tx_for_testing(Tip(50), TransactionHash(StarkFelt::ONE));
    let tx2 = create_internal_invoke_tx_for_testing(Tip(100), TransactionHash(StarkFelt::TWO));
    let tx3 = create_internal_invoke_tx_for_testing(Tip(10), TransactionHash(StarkFelt::THREE));

    let mut priority_queue = PriorityQueue::new();
    priority_queue.push(tx1.clone());
    priority_queue.push(tx2.clone());
    priority_queue.push(tx3.clone());
    let mut mempool = Mempool { priority_queue };

    let txs = mempool.get_tx(requested_txs).unwrap();
    assert_eq!(txs.len(), expected_txs);

    if requested_txs > 0 {
        assert_eq!(txs[0], tx2);
    }
    if requested_txs > 1 {
        assert_eq!(txs[1], tx1);
    }
    if requested_txs > 2 {
        assert_eq!(txs[2], tx3);
    }
}
