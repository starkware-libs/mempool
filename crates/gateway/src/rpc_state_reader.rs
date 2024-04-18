use blockifier::execution::contract_class::ContractClass;
use blockifier::state::errors::StateError;
use blockifier::state::state_api::{StateReader, StateResult};
use reqwest::blocking::Client as BlockingClient;
use serde_json::json;
use starknet_api::core::{ChainId, ClassHash, CompiledClassHash, ContractAddress, Nonce};
use starknet_api::hash::StarkFelt;
use starknet_api::state::StorageKey;
use url::Url;

use crate::errors::RpcStateReaderResult;
use crate::rpc_objects::{
    BlockId, BlockTag, GetNonceParams, RpcResponse, RPC_ERROR_BLOCK_NOT_FOUND,
    RPC_ERROR_CONTRACT_ADDRESS_NOT_FOUND,
};

#[cfg(test)]
#[path = "rpc_state_reader_test.rs"]
mod rpc_state_reader_test;

pub struct RpcStateReader {
    pub url: Url,
    pub json_rpc_version: String,
    pub id: u64,
}

impl RpcStateReader {
    pub fn new(chain_id: ChainId, spec_version: &str) -> RpcStateReaderResult<Self> {
        let url = match chain_id.0.as_str() {
            "SN_MAIN" => Url::parse("https://papyrus-for-mempool-mainnet.sw-dev.io/rpc/")?
                .join(spec_version)?,
            "SN_SEPOLIA" => Url::parse("https://papyrus-for-mempool-sepolia.sw-dev.io/rpc/")?
                .join(spec_version)?,
            "SN_INTEGRATION_SEPOLIA" => {
                Url::parse("https://papyrus-for-mempool-integration-sepolia.sw-dev.io/rpc/")?
                    .join(spec_version)?
            }
            _ => panic!("Unsupported chain id"),
        };
        Ok(Self {
            url,
            json_rpc_version: "2.0".to_string(),
            id: 0,
        })
    }

    // Note: This function is blocking though it is sending a request to the rpc server and waiting
    // for the response.
    pub fn send_rpc_request(&self, request_body: serde_json::Value) -> Result<String, StateError> {
        let client = BlockingClient::new();
        let response = client
            .post(self.url.clone())
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .map_err(|e| {
                StateError::StateReadError(format!("Rpc request failed with error {:?}", e))
            })?;

        if response.status().is_success() {
            let response_text = response
                .text()
                .map_err(|_| StateError::StateReadError("Bad rpc response".to_string()))?;

            let rpc_response: RpcResponse = serde_json::from_str(&response_text)
                .map_err(|_| StateError::StateReadError("Bad rpc response".to_string()))?;

            match rpc_response {
                RpcResponse::Success(rpc_success_response) => Ok(rpc_success_response.result),
                RpcResponse::Error(rpc_error_response) => match rpc_error_response.error.code {
                    RPC_ERROR_BLOCK_NOT_FOUND => {
                        Err(StateError::StateReadError("Block not found".to_string()))
                    }
                    RPC_ERROR_CONTRACT_ADDRESS_NOT_FOUND => Err(StateError::StateReadError(
                        format!("Contract address not found, request: {}", request_body),
                    )),
                    _ => Err(StateError::StateReadError(format!(
                        "unexpected error code {}",
                        rpc_error_response.error.code
                    ))),
                },
            }
        } else {
            let error_code = response.status();
            Err(StateError::StateReadError(format!(
                "RPC ERROR, code {}",
                error_code
            )))
        }
    }
}

impl StateReader for RpcStateReader {
    #[allow(unused_variables)]
    fn get_storage_at(
        &self,
        contract_address: ContractAddress,
        key: StorageKey,
    ) -> StateResult<StarkFelt> {
        todo!()
    }

    fn get_nonce_at(&self, contract_address: ContractAddress) -> StateResult<Nonce> {
        let get_nonce_params = GetNonceParams {
            block_id: BlockId::Tag(BlockTag::Latest),
            contract_address,
        };
        let request_body = json!({
            "jsonrpc": self.json_rpc_version,
            "id": self.id,
            "method": "starknet_getNonce",
            "params": json!(get_nonce_params),
        });

        let result = self.send_rpc_request(request_body)?;
        let nonce: Nonce = serde_json::from_value(json!(result))
            .map_err(|_| StateError::StateReadError("Bad rpc result".to_string()))?;
        Ok(nonce)
    }

    #[allow(unused_variables)]
    fn get_compiled_contract_class(&self, class_hash: ClassHash) -> StateResult<ContractClass> {
        todo!()
    }

    #[allow(unused_variables)]
    fn get_class_hash_at(&self, contract_address: ContractAddress) -> StateResult<ClassHash> {
        todo!()
    }

    #[allow(unused_variables)]
    fn get_compiled_class_hash(&self, class_hash: ClassHash) -> StateResult<CompiledClassHash> {
        todo!()
    }
}
