pub trait MemPool {
    fn add_transaction(&mut self, trx: u32) -> impl std::future::Future<Output = bool> + Send;
}

#[derive(Default)]
pub struct DummyActualMemPool {
    transactions: Vec<u32>,
}
impl DummyActualMemPool {
    pub fn new() -> Self {
        DummyActualMemPool {
            transactions: vec![],
        }
    }
}

impl MemPool for DummyActualMemPool {
    async fn add_transaction(&mut self, trx: u32) -> bool {
        self.transactions.push(trx);
        true
    }
}
