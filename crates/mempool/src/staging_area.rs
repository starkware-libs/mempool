use starknet_api::transaction::TransactionHash;
use starknet_mempool_types::errors::MempoolError;

#[derive(Clone, Debug, Default, derive_more::Deref, derive_more::DerefMut)]
pub struct StagingArea(Vec<TransactionHash>);
impl StagingArea {
    pub fn insert(&mut self, tx_hash: TransactionHash) -> Result<(), MempoolError> {
        if self.0.contains(&tx_hash) {
            return Err(MempoolError::DuplicateTransaction { tx_hash });
        }
        self.0.push(tx_hash);

        Ok(())
    }

    pub fn remove(&mut self, n: usize) {
        (0..n).for_each(|_| {
            self.0.remove(0);
        });
    }
}
