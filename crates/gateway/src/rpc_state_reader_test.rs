use serde::Serialize;
use serde_json::json;
use starknet_api::block::{BlockNumber, GasPrice};

use crate::config::RpcStateReaderConfig;
use crate::rpc_objects::{
    BlockHeader, BlockId, GetBlockWithTxHashesParams, ResourcePrice, RpcResponse,
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
    let request_body = mockito::Matcher::Json(json!({
        "jsonrpc": json_rpc_version,
        "id": 0,
        "method": method,
        "params": json!(params),
    }));
    dbg!(serde_json::to_string(&expected_response).unwrap());
    server
        .mock("POST", "/")
        .match_header("Content-Type", "application/json")
        .match_body(request_body)
        .with_status(201)
        .with_body(serde_json::to_string(expected_response).unwrap())
        .create()
}

// TODO(yair): Create case for each method.
#[tokio::test]
async fn test_get_block_info() {
    let mut server = run_rpc_server().await;
    let config = RpcStateReaderConfig { url: server.url(), ..Default::default() };

    let mock = mock_rpc_interaction(
        &mut server,
        &config.json_rpc_version,
        "starknet_getBlockWithTxHashes",
        GetBlockWithTxHashesParams { block_id: BlockId::Latest },
        &RpcResponse::Success(RpcSuccessResponse {
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
    );

    let client = RpcStateReader::from_latest(&config);
    tokio::task::spawn_blocking(move || {
        let block_info = client.get_block_info().unwrap();
        // TODO(yair): Add partial_eq for BlockInfo and assert_eq the whole BlockInfo.
        assert_eq!(block_info.block_number, BlockNumber(100));
    })
    .await
    .unwrap();

    mock.assert_async().await;
}
