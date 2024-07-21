use std::collections::{hash_map, BTreeMap, HashMap};
use std::fmt::Debug;

use starknet_api::core::{ContractAddress, Nonce};
use starknet_api::transaction::TransactionHash;
use starknet_mempool_types::errors::MempoolError;
use starknet_mempool_types::mempool_types::{MempoolResult, ThinTransaction};
use thiserror::Error;

use crate::mempool::TransactionReference;

type HashToTransaction = HashMap<TransactionHash, ThinTransaction>;

/// Contains all transactions currently held in the mempool.
/// Invariant: both data structures are consistent regarding the existence of transactions:
/// A transaction appears in one if and only if it appears in the other.
/// No duplicate transactions appear in the pool.
#[derive(Debug, Default, Eq, PartialEq)]
pub struct TransactionPool {
    // Holds the complete transaction objects; it should be the sole entity that does so.
    tx_pool: HashToTransaction,
    // Transactions organized by account address, sorted by ascending nonce values.
    txs_by_account: AccountTransactionIndex,
}

impl TransactionPool {
    pub fn insert(&mut self, tx: ThinTransaction) -> MempoolResult<()> {
        let tx_reference = TransactionReference::new(&tx);
        let tx_hash = tx_reference.tx_hash;

        // Insert to pool.
        if let hash_map::Entry::Vacant(entry) = self.tx_pool.entry(tx_hash) {
            entry.insert(tx);
        } else {
            return Err(MempoolError::DuplicateTransaction { tx_hash });
        }

        // Insert to account mapping.
        let unexpected_existing_tx = self.txs_by_account.insert(tx_reference);
        if unexpected_existing_tx.is_some() {
            panic!(
                "Transaction pool consistency error: transaction with hash {tx_hash} does not \
                 appear in main mapping, but it appears in the account mapping",
            )
        };

        Ok(())
    }

    pub fn remove(&mut self, tx_hash: TransactionHash) -> MempoolResult<ThinTransaction> {
        // Remove from pool.
        let tx =
            self.tx_pool.remove(&tx_hash).ok_or(MempoolError::TransactionNotFound { tx_hash })?;

        // Remove from account mapping.
        self.txs_by_account.remove(TransactionReference::new(&tx)).unwrap_or_else(|| {
            panic!(
                "Transaction pool consistency error: transaction with hash {tx_hash} appears in \
                 main mapping, but does not appear in the account mapping"
            )
        });

        Ok(tx)
    }

    pub fn get_by_tx_hash(&self, tx_hash: TransactionHash) -> MempoolResult<&ThinTransaction> {
        self.tx_pool.get(&tx_hash).ok_or(MempoolError::TransactionNotFound { tx_hash })
    }

    pub fn get_by_address_and_nonce(
        &self,
        address: ContractAddress,
        nonce: Nonce,
    ) -> Option<&TransactionReference> {
        self.txs_by_account.get(address, nonce)
    }

    #[cfg(test)]
    pub(crate) fn _tx_pool(&self) -> &HashToTransaction {
        &self.tx_pool
    }
}

#[derive(Debug, Default, Eq, PartialEq)]
struct AccountTransactionIndex(HashMap<ContractAddress, BTreeMap<Nonce, TransactionReference>>);

impl AccountTransactionIndex {
    /// If the transaction already exists in the mapping, the old value is returned.
    fn insert(&mut self, tx: TransactionReference) -> Option<TransactionReference> {
        self.0.entry(tx.sender_address).or_default().insert(tx.nonce, tx)
    }

    fn remove(&mut self, tx: TransactionReference) -> Option<TransactionReference> {
        let TransactionReference { sender_address, nonce, .. } = tx;
        let account_txs = self.0.get_mut(&sender_address)?;

        let removed_tx = account_txs.remove(&nonce);

        if removed_tx.is_some() && account_txs.is_empty() {
            self.0.remove(&sender_address);
        }

        removed_tx
    }

    fn get(&self, address: ContractAddress, nonce: Nonce) -> Option<&TransactionReference> {
        self.0.get(&address)?.get(&nonce)
    }
}

pub trait State {}

pub trait Executable {
    type State: State;
    type Output;
    type Error: Debug;

    fn execute(&self, state: &mut Self::State) -> Result<Self::Output, Self::Error>;
}
pub trait TxExecutor {
    type State: State;
    type Tx: Executable<State = Self::State>;

    #[allow(clippy::type_complexity)]
    fn execute_txs(
        &self,
        state: &mut Self::State,
        txs: &[Self::Tx],
    ) -> Vec<Result<<Self::Tx as Executable>::Output, <Self::Tx as Executable>::Error>>;
}

pub struct StateDiff {
    _nonces: HashMap<ContractAddress, Nonce>,
}

#[derive(Debug, Error)]

pub enum _MempoolError {}

pub type _MempoolResult<T> = Result<T, _MempoolError>;

pub trait MempoolClient {
    type State: State;
    type Tx: Executable<State = Self::State> + PartialEq + Eq;

    fn get_txs(&mut self, n_txs: usize) -> _MempoolResult<Vec<Self::Tx>>;
    fn add_tx(&mut self, tx: Self::Tx) -> _MempoolResult<()>;
    fn commit_block(&mut self, state_changes: StateDiff) -> _MempoolResult<()>;
}

struct Block<Tx: Executable> {
    tx_executions: Vec<Tx::Output>,
}

struct BlockContext;

#[derive(Debug, Error)]
pub enum BatcherError {
    #[error(transparent)]
    MempoolError(_MempoolError),
}

impl From<_MempoolError> for BatcherError {
    fn from(err: _MempoolError) -> Self {
        BatcherError::MempoolError(err)
    }
}

pub type BatcherResult<T> = Result<T, BatcherError>;

struct Batcher<
    S: State,
    Tx: Executable<State = S>,
    Mempool: MempoolClient<State = S, Tx = Tx>,
    TxExec: TxExecutor<State = S, Tx = Tx>,
> {
    mempool: Mempool,
    tx_executor: TxExec,
}

impl<S, Tx, Mempool, TxExec> Batcher<S, Tx, Mempool, TxExec>
where
    S: State,
    Tx: Executable<State = S>,
    Mempool: MempoolClient<State = S, Tx = Tx>,
    TxExec: TxExecutor<State = S, Tx = Tx>,
{
    // fn new(mempool_client: Self::Mempool, tx_executor: Self::TxExecutor) -> Self;

    fn build_tx_executor(&self, block_context: &BlockContext) -> BatcherResult<TxExec> {
        unimplemented!()
    }

    fn build_state(&self, block_context: &BlockContext) -> BatcherResult<S> {
        unimplemented!()
    }

    fn build_block(&mut self, block_context: &BlockContext) -> BatcherResult<Block<Tx>> {
        let tx_executor = self.build_tx_executor(block_context)?;
        let mut state = self.build_state(block_context)?;
        let txs = self.mempool.get_txs(10)?;
        let results = tx_executor.execute_txs(&mut state, &txs);
        let tx_executions = results.into_iter().filter_map(Result::ok).collect();

        Ok(Block { tx_executions })
    }
}
