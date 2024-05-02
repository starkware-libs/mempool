use assert_matches::assert_matches;
use blockifier::state::errors::StateError;

use blockifier::state::state_api::StateReader;
use serde_json::json;
use starknet_api::core::ContractAddress;
use starknet_api::core::PatriciaKey;
use starknet_api::hash::StarkHash;
use starknet_api::state::StorageKey;
use starknet_api::{contract_address, patricia_key};

use crate::rpc_objects::BlockId;
use crate::rpc_objects::BlockTag;
use crate::rpc_objects::GetClassHashAtParams;
use crate::rpc_objects::GetNonceParams;
use crate::rpc_objects::GetStorageAtParams;
use crate::rpc_state_reader::RpcStateReader;
use url::Url;

const STARKNET_SPEC_VERSION: &str = "v0_7";
const JSON_RPC_VERSION: &str = "2.0";
const RPC_URL_MAINNET: &str = "https://papyrus-for-mempool-mainnet.sw-dev.io/rpc/";
// TODO(yael 1/5/2024): test also sepolia and sepolia-integration
const _RPC_URL_SEPOLIA: &str = "https://papyrus-for-mempool-sepolia.sw-dev.io/rpc/";
const _RPC_URL_INTEGRATION_SEPOLIA: &str =
    "https://papyrus-for-mempool-integration-sepolia.sw-dev.io/rpc/";

// TODO(yael 1/5/2024): deploy a contract for testing, with known values for storage, nonce, etc.
const VALID_CONTRACT_ADDRESS: &str =
    "0x0240edc989dfc8b4d75d7f6fa6f8a48e7ff4af358c6dd72e4b3cb67687b204e0";

fn state_reader() -> RpcStateReader {
    RpcStateReader {
        url: Url::parse(RPC_URL_MAINNET)
            .unwrap()
            .join(STARKNET_SPEC_VERSION)
            .unwrap(),
        json_rpc_version: JSON_RPC_VERSION.to_string(),
    }
}

#[test]
fn test_rpc_get_nonce() {
    let state_reader = state_reader();
    // Query with a valid address
    let contract_address = contract_address!(VALID_CONTRACT_ADDRESS);

    let res = state_reader.get_nonce_at(contract_address);
    assert!(res.is_ok());

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
                block_id: BlockId::Tag(BlockTag::Latest),
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
fn test_rpc_get_storage_at() {
    let state_reader = state_reader();
    // Query with a valid address
    let contract_address = contract_address!(VALID_CONTRACT_ADDRESS);

    let res = state_reader.get_storage_at(contract_address, StorageKey(patricia_key!("0x0")));
    assert!(res.is_ok());

    // Query with an invalid address
    let contract_address = contract_address!("0x0");

    let res = state_reader.get_storage_at(contract_address, StorageKey(patricia_key!("0x0")));

    let expected_error = format!(
        "Contract address not found, request: {}",
        json!({
            "jsonrpc": JSON_RPC_VERSION,
            "id": 0,
            "method": "starknet_getStorageAt",
            "params": json!(GetStorageAtParams {
                block_id: BlockId::Tag(BlockTag::Latest),
                contract_address,
                key: StorageKey(patricia_key!("0x0")),
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
fn test_rpc_get_class_hash_at() {
    let state_reader = state_reader();
    // Query with a valid address
    let contract_address = contract_address!(VALID_CONTRACT_ADDRESS);

    let res = state_reader.get_class_hash_at(contract_address);
    assert!(res.is_ok());

    // Query with an invalid address
    let contract_address = contract_address!("0x0");

    let res = state_reader.get_class_hash_at(contract_address);

    let expected_error = format!(
        "Contract address not found, request: {}",
        json!({
            "jsonrpc": JSON_RPC_VERSION,
            "id": 0,
            "method": "starknet_getClassHashAt",
            "params": json!(GetClassHashAtParams {
                block_id: BlockId::Tag(BlockTag::Latest),
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
