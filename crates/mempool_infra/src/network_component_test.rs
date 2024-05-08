use tokio::sync::mpsc::channel;
use tokio::task;

use crate::network_component::{CommunicationInterface, NetworkComponent};
use crate::tonic_network_component::TonicNetworkComponent;

type AtoB = u32;
type BtoA = i32;

struct TestComponentA {
    pub network: Box<dyn CommunicationInterface<SendType = AtoB, ReceiveType = BtoA> + Send + Sync>,
}
struct TestComponentB {
    pub network: Box<dyn CommunicationInterface<SendType = BtoA, ReceiveType = AtoB> + Send + Sync>,
}

#[tokio::test]
async fn test_send_and_receive() {
    let (tx_a2b, rx_a2b) = channel::<AtoB>(1);
    let (tx_b2a, rx_b2a) = channel::<BtoA>(1);

    let network_a = NetworkComponent::new(tx_a2b, rx_b2a);
    let network_b = NetworkComponent::new(tx_b2a, rx_a2b);

    let mut a = TestComponentA { network: Box::new(network_a) };
    let mut b = TestComponentB { network: Box::new(network_b) };

    task::spawn(async move {
        let a2b: AtoB = 1;
        a.network.send(a2b).await.unwrap();
    })
    .await
    .unwrap();

    let ret = task::spawn(async move { b.network.recv().await }).await.unwrap();

    let expected_ret: Option<AtoB> = Some(1);
    assert_eq!(ret, expected_ret);
}

#[tokio::test]
async fn test_tonic() {
    let network_a = TonicNetworkComponent::<u32, i32>::new(10000, 10001);
    let network_b = TonicNetworkComponent::<i32, u32>::new(10001, 10000);

    let mut a = TestComponentA { network: Box::new(network_a) };
    let mut b = TestComponentB { network: Box::new(network_b) };

    a.network.send(1).await.unwrap();

    let ret = b.network.recv().await.unwrap();
    assert_eq!(ret, 1);

    let ret = a.network.send(1).await;
    assert_eq!(ret, Ok(()));

    let ret = b.network.recv().await.unwrap();
    assert_eq!(ret, 1);
}
