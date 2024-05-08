use tokio::task;

use crate::network_component::CommunicationInterface;
use crate::rpc_network_component::RpcNetworkComponent;

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
    let network_a = RpcNetworkComponent::<u32, i32>::new(10000, 10001);
    let network_b = RpcNetworkComponent::<i32, u32>::new(10001, 10000);

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
