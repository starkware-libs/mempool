use crate::gateway::add_transaction;
use axum::Json;
use axum::{body::HttpBody, response::IntoResponse};
use rstest::fixture;
use starknet_api::external_transaction::DataAvailabilityMode::L1;
use starknet_api::external_transaction::{
    ExternalDeclareTransaction, ExternalDeclareTransactionV3, ExternalDeployAccountTransaction,
    ExternalDeployAccountTransactionV3, ExternalInvokeTransaction, ExternalInvokeTransactionV3,
    ExternalTransaction,
};

// TODO(Ayelet): Change to non-default values.
#[fixture]
fn create_external_declare_transaction_v3() -> ExternalTransaction {
    let declare_transaction = ExternalDeclareTransactionV3 {
        contract_class: Default::default(),
        resource_bounds: Default::default(),
        tip: Default::default(),
        signature: Default::default(),
        nonce: Default::default(),
        compiled_class_hash: Default::default(),
        sender_address: Default::default(),
        nonce_data_availability_mode: L1,
        fee_data_availability_mode: L1,
        paymaster_data: Default::default(),
        account_deployment_data: Default::default(),
        version: Default::default(),
        r#type: Default::default(),
    };

    ExternalTransaction::Declare(ExternalDeclareTransaction::V3(declare_transaction))
}

// TODO(Ayelet): Change to non-default values.
#[fixture]
fn create_external_deploy_account_transaction_v3() -> ExternalTransaction {
    let deploy_account_transaction = ExternalDeployAccountTransactionV3 {
        resource_bounds: Default::default(),
        tip: Default::default(),
        contract_address_salt: Default::default(),
        class_hash: Default::default(),
        constructor_calldata: Default::default(),
        nonce: Default::default(),
        signature: Default::default(),
        nonce_data_availability_mode: L1,
        fee_data_availability_mode: L1,
        paymaster_data: Default::default(),
        version: Default::default(),
        r#type: Default::default(),
    };

    ExternalTransaction::DeployAccount(ExternalDeployAccountTransaction::V3(
        deploy_account_transaction,
    ))
}

// TODO(Ayelet): Change to non-default values.
#[fixture]
fn create_external_invoke_transaction_v3() -> ExternalTransaction {
    let invoke_transaction = ExternalInvokeTransactionV3 {
        resource_bounds: Default::default(),
        tip: Default::default(),
        calldata: Default::default(),
        sender_address: Default::default(),
        nonce: Default::default(),
        signature: Default::default(),
        nonce_data_availability_mode: L1,
        fee_data_availability_mode: L1,
        paymaster_data: Default::default(),
        account_deployment_data: Default::default(),
        version: Default::default(),
        r#type: Default::default(),
    };

    ExternalTransaction::Invoke(ExternalInvokeTransaction::V3(invoke_transaction))
}

#[rstest::rstest]
#[case("DECLARE", create_external_declare_transaction_v3())]
#[case("DEPLOY_ACCOUNT", create_external_deploy_account_transaction_v3())]
#[case("INVOKE", create_external_invoke_transaction_v3())]
#[tokio::test]
async fn test_add_transaction(
    #[case] expected_response: &str,
    #[case] transaction_instance: ExternalTransaction,
) {
    let transaction_json: Json<ExternalTransaction> = Json(transaction_instance);
    let response = add_transaction(transaction_json).await.into_response();
    let response_bytes = response.into_body().collect().await.unwrap().to_bytes();

    assert_eq!(
        &String::from_utf8(response_bytes.to_vec()).unwrap(),
        expected_response
    );
}
