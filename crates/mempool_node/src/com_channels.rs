use starknet_mempool_types::communication::MempoolRequestAndResponseSender;
use tokio::sync::mpsc::{channel, Receiver, Sender};

pub struct SingleChannel<T: Send + Sync> {
    pub tx: Sender<T>,
    pub rx: Receiver<T>,
}

pub struct CommChannels {
    pub mempool_channel: SingleChannel<MempoolRequestAndResponseSender>,
}

pub fn create_channels() -> CommChannels {
    // TODO: (lev | 2021-08-31) Make this configurable.
    const MEMPOOL_INVOCATIONS_QUEUE_SIZE: usize = 32;
    let (tx_mempool, rx_mempool) =
        channel::<MempoolRequestAndResponseSender>(MEMPOOL_INVOCATIONS_QUEUE_SIZE);
    CommChannels { mempool_channel: SingleChannel { tx: tx_mempool, rx: rx_mempool } }
}
