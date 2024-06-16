use starknet_api::transaction::Tip;
use starknet_gateway::invoke_tx_args;
use starknet_mempool_integration_tests::integration_test_setup::IntegrationTestSetup;

#[tokio::test]
// TODO: when multi-nonce is supported, add them here; split into multiple tests once things get
// too complicated.
async fn test_mempool_emptying_scenarios() {
    let n_accounts_in_test = 2;
    let (mut mock_running_system, mut tx_generator) =
        IntegrationTestSetup::new_with_tx_generator(n_accounts_in_test).await;

    // asking for more txs than the number stored should return all the stored txs.
    let contract0_tip1 = tx_generator.account(0).generate(invoke_tx_args! { tip: Tip(1)});
    let tx_hash_for_contract0_tip1 =
        mock_running_system.assert_add_tx_success(&contract0_tip1).await;
    mock_running_system.assert_get_txs_eq(2, &[tx_hash_for_contract0_tip1]).await;

    // Asking for any number of txs when the mempool is empty should return [].
    mock_running_system.assert_get_txs_eq(2, &[]).await;
    mock_running_system.assert_get_txs_eq(0, &[]).await;

    // Gen new tx from the first contract and add a second account with a a new tx, check that
    // retrieving 2 txs returns both.
    let contract0_tip3 = tx_generator.account(0).generate(invoke_tx_args! { tip: Tip(3) });
    let contract1_tip2 = tx_generator.account(1).generate(invoke_tx_args! { tip: Tip(2) });

    let tx_hash_for_contract1_tip2 =
        mock_running_system.assert_add_tx_success(&contract1_tip2).await;
    let tx_hash_for_contract0_tip3 =
        mock_running_system.assert_add_tx_success(&contract0_tip3).await;

    mock_running_system
        .assert_get_txs_eq(2, &[tx_hash_for_contract0_tip3, tx_hash_for_contract1_tip2])
        .await;
}
