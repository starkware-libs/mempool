mod tests {
    use crate::network_component::NetworkComponent;
    use tokio::{sync::mpsc::channel, task};

    #[tokio::test]
    async fn test_simple_send_and_receive() {
        type AtoB = u32;
        type BtoA = i32;

        struct A {
            pub network_component: NetworkComponent<AtoB, BtoA>,
        }
        struct B {
            pub network_component: NetworkComponent<BtoA, AtoB>,
        }

        let (tx_a2b, rx_a2b) = channel::<AtoB>(1);
        let (tx_b2a, rx_b2a) = channel::<BtoA>(1);

        let network_component_a = NetworkComponent::new(tx_a2b, rx_b2a);
        let network_component_b = NetworkComponent::new(tx_b2a, rx_a2b);

        let a = A {
            network_component: network_component_a,
        };
        let mut b = B {
            network_component: network_component_b,
        };

        task::spawn(async move {
            let a2b: AtoB = 1;
            a.network_component.tx.send(a2b).await.unwrap();
        })
        .await
        .unwrap();

        let ret = task::spawn(async move { b.network_component.rx.recv().await.unwrap() })
            .await
            .unwrap();

        let expected_ret: AtoB = 1;
        assert_eq!(ret, expected_ret);
    }
}
