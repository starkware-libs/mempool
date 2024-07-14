use blockifier::execution::contract_class::ContractClass;
use blockifier::state::state_api::StateReader;
use cairo_lang_starknet_classes::casm_contract_class::CasmContractClass;
use papyrus_rpc::CompiledContractClass;
use rstest::rstest;
use serde::Serialize;
use serde_json::json;
use starknet_api::block::{BlockNumber, GasPrice};
use starknet_api::core::{ClassHash, ContractAddress, Nonce, PatriciaKey};
use starknet_api::{class_hash, contract_address, felt, patricia_key};

use crate::config::RpcStateReaderConfig;
use crate::rpc_objects::{
    BlockHeader, BlockId, GetBlockWithTxHashesParams, GetClassHashAtParams,
    GetCompiledContractClassParams, GetNonceParams, GetStorageAtParams, ResourcePrice, RpcResponse,
    RpcSuccessResponse,
};
use crate::rpc_state_reader::RpcStateReader;
use crate::state_reader::MempoolStateReader;

async fn run_rpc_server() -> mockito::ServerGuard {
    mockito::Server::new_async().await
}

fn mock_rpc_interaction(
    server: &mut mockito::ServerGuard,
    json_rpc_version: &str,
    method: &str,
    params: impl Serialize,
    expected_response: &RpcResponse,
) -> mockito::Mock {
    let request_body = json!({
        "jsonrpc": json_rpc_version,
        "id": 0,
        "method": method,
        "params": json!(params),
    });
    server
        .mock("POST", "/")
        .match_header("Content-Type", "application/json")
        .match_body(mockito::Matcher::Json(request_body))
        .with_status(201)
        .with_body(serde_json::to_string(expected_response).unwrap())
        .create()
}

#[rstest]
#[case::get_block_info(
    "starknet_getBlockWithTxHashes",
    GetBlockWithTxHashesParams { block_id: BlockId::Latest },
    RpcResponse::Success(RpcSuccessResponse {
        result: serde_json::to_value(BlockHeader {
            block_number: BlockNumber(100),
            // GasPrice must be non-zero.
            l1_gas_price: ResourcePrice {
                price_in_wei: GasPrice(1),
                price_in_fri: GasPrice(1),
            },
            l1_data_gas_price: ResourcePrice {
                price_in_wei: GasPrice(1),
                price_in_fri: GasPrice(1),
            },
            ..Default::default()
        })
        .unwrap(),
        ..Default::default()
    }),
    |client: &RpcStateReader| {
        let block_info = client.get_block_info().unwrap();
        // TODO(yair): Add partial_eq for BlockInfo and assert_eq the whole BlockInfo.
        assert_eq!(block_info.block_number, BlockNumber(100));
    }
)]
#[case::get_storage_at(
    "starknet_getStorageAt",
    GetStorageAtParams { block_id: BlockId::Latest, contract_address: contract_address!("0x1"), key: starknet_api::state::StorageKey::from(0u32),},
    RpcResponse::Success(RpcSuccessResponse {
        result: serde_json::to_value(felt!("0x999")).unwrap(),
        ..Default::default()
    }),
    |client: &RpcStateReader| {
        let storage_value = client.get_storage_at(
            contract_address!("0x1"),
            starknet_api::state::StorageKey::from(0u32),
        ).unwrap();
        assert_eq!(storage_value, felt!("0x999"));
    }
)]
#[case::get_nonce_at(
    "starknet_getNonce",
    GetNonceParams { block_id: BlockId::Latest, contract_address: contract_address!("0x1")},
    RpcResponse::Success(RpcSuccessResponse {
        result: serde_json::to_value(felt!("0x999")).unwrap(),
        ..Default::default()
    }),
    |client: &RpcStateReader| {
        let nonce = client.get_nonce_at(
            contract_address!("0x1"),
        ).unwrap();
        assert_eq!(nonce, Nonce(felt!("0x999")));
    }
)]
#[case::get_compiled_contract_class(
    "starknet_getCompiledContractClass",
    GetCompiledContractClassParams { block_id: BlockId::Latest, class_hash: class_hash!("0x1")},
    RpcResponse::Success(RpcSuccessResponse {
        result: serde_json::to_value(CompiledContractClass::V1(CasmContractClass::default())).unwrap(),
        ..Default::default()
    }),
    |client: &RpcStateReader| {
        let compiled_class = client.get_compiled_contract_class(
            class_hash!("0x1"),
        ).unwrap();
        assert_eq!(compiled_class, ContractClass::V1(CasmContractClass::default().try_into().unwrap()));
    }
)]
#[case::get_class_hash_at(
    "starknet_getClassHashAt",
    GetClassHashAtParams { block_id: BlockId::Latest, contract_address: contract_address!("0x1")},
    RpcResponse::Success(RpcSuccessResponse {
        result: serde_json::to_value(class_hash!("0x999")).unwrap(),
        ..Default::default()
    }),
    |client: &RpcStateReader| {
        let class_hash = client.get_class_hash_at(
            contract_address!("0x1"),
        ).unwrap();
        assert_eq!(class_hash, class_hash!("0x999"));
    }
)]
#[tokio::test]
async fn test_rpc_state_reader_happy_flow(
    #[case] method: &str,
    #[case] params: impl Serialize,
    #[case] expected_response: RpcResponse,
    #[case] client_call: fn(&RpcStateReader) -> (),
) {
    let mut server = run_rpc_server().await;
    let config = RpcStateReaderConfig { url: server.url(), ..Default::default() };

    let mock = mock_rpc_interaction(
        &mut server,
        &config.json_rpc_version,
        method,
        params,
        &expected_response,
    );

    let client = RpcStateReader::from_latest(&config);
    tokio::task::spawn_blocking(move || client_call(&client)).await.unwrap();

    mock.assert_async().await;
}
