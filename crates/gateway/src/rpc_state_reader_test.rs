use assert_matches::assert_matches;
use blockifier::state::errors::StateError;

use blockifier::state::state_api::StateReader;
use serde_json::json;
use starknet_api::core::ChainId;
use starknet_api::core::ContractAddress;
use starknet_api::core::PatriciaKey;
use starknet_api::hash::StarkHash;
use starknet_api::{contract_address, patricia_key};

use crate::rpc_objects::BlockId;
use crate::rpc_objects::BlockTag;
use crate::rpc_objects::GetNonceParams;
use crate::rpc_state_reader::RpcStateReader;

#[test]
fn test_rpc_get_nonce() {
    let state_reader = RpcStateReader::new(ChainId("SN_MAIN".to_string()), "v0_6").unwrap();
    // Query with a valid address
    let contract_address =
        contract_address!("0x0240edc989dfc8b4d75d7f6fa6f8a48e7ff4af358c6dd72e4b3cb67687b204e0");

    let res = state_reader.get_nonce_at(contract_address);
    assert!(res.is_ok());

    // Query with an invalid address
    let contract_address = contract_address!("0x0");

    let res = state_reader.get_nonce_at(contract_address);

    let expected_error = format!(
        "Contract address not found, request: {}",
        json!({
            "jsonrpc": "2.0",
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
