use assert_matches::assert_matches;
use blockifier::state::errors::StateError;

use blockifier::state::state_api::StateReader;
use serde_json::json;
use starknet_api::block::BlockNumber;
use starknet_api::core::ContractAddress;
use starknet_api::core::PatriciaKey;
use starknet_api::hash::StarkHash;
use starknet_api::{contract_address, patricia_key};

use crate::rpc_objects::BlockId;
use crate::rpc_objects::GetNonceParams;
use crate::rpc_state_reader::RpcStateReader;
use url::Url;

const STARKNET_SPEC_VERSION: &str = "v0_6";
const JSON_RPC_VERSION: &str = "2.0";
const RPC_URL_MAINNET: &str = "https://papyrus-for-mempool-mainnet.sw-dev.io/rpc/";
// TODO(yael 1/5/2024): test also sepolia and sepolia-integration
const _RPC_URL_SEPOLIA: &str = "https://papyrus-for-mempool-sepolia.sw-dev.io/rpc/";
const _RPC_URL_INTEGRATION_SEPOLIA: &str =
    "https://papyrus-for-mempool-integration-sepolia.sw-dev.io/rpc/";

#[test]
fn test_rpc_get_nonce() {
    let state_reader = RpcStateReader {
        url: Url::parse(RPC_URL_MAINNET)
            .unwrap()
            .join(STARKNET_SPEC_VERSION)
            .unwrap(),
        json_rpc_version: JSON_RPC_VERSION.to_string(),
        block_number: BlockNumber(637395),
    };
    // Query with a valid address
    let contract_address =
        contract_address!("0x0240edc989dfc8b4d75d7f6fa6f8a48e7ff4af358c6dd72e4b3cb67687b204e0");

    let res = state_reader.get_nonce_at(contract_address);
    assert!(res.is_ok(), "Failed to get nonce: {:?}", res);

    // Query with an invalid address
    let contract_address = contract_address!("0x0");

    let res = state_reader.get_nonce_at(contract_address);

    let expected_error = format!(
        "Contract address not found, request: {}",
        json!({
            "jsonrpc": JSON_RPC_VERSION,
            "id": 0,
            "method": "starknet_getNonce",
            "params": json!(GetNonceParams {
                block_id: BlockId::Number(state_reader.block_number),
                contract_address,
            })}
        )
    );
    assert_matches!(
        res,
        Err(StateError::StateReadError(state_reader_error))
        if state_reader_error == expected_error
    );
}

#[test]
fn test_rpc_get_block_info() {
    use crate::rpc_objects::Tag;

    let rpc_reader = RpcReader {
        url: Url::parse("http://papyrus-for-mempool-mainnet.sw-dev.io/rpc/v0_7").unwrap(),
        json_rpc_version: "2.0".to_string(),
        block_id: BlockId::Tag(Tag::Latest),
    };

    let res = rpc_reader.get_block_info();
    assert!(res.is_ok(), "Failed to get block info: {:?}", res);
}
