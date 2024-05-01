use serde::{Deserialize, Serialize};
use starknet_api::{core::ContractAddress, state::StorageKey};

// Starknet Spec error codes:
// TODO(yael 30/4/2024): consider turning these into an enum.
pub const RPC_ERROR_BLOCK_NOT_FOUND: u16 = 24;
pub const RPC_ERROR_CONTRACT_ADDRESS_NOT_FOUND: u16 = 20;

#[derive(Deserialize, Serialize)]
pub enum BlockTag {
    /// The most recent fully constructed block
    #[serde(rename = "latest")]
    Latest,
    /// Currently constructed block
    #[serde(rename = "pending")]
    Pending,
}

#[derive(Deserialize, Serialize)]
#[serde(untagged)]
pub enum BlockId {
    Tag(BlockTag),
    // There are additional options in the spec that are not implemented here
}

#[derive(Serialize, Deserialize)]
pub struct GetNonceParams {
    pub block_id: BlockId,
    pub contract_address: ContractAddress,
}

#[derive(Serialize, Deserialize)]
pub struct GetStorageAtParams {
    pub contract_address: ContractAddress,
    pub key: StorageKey,
    pub block_id: BlockId,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum RpcResponse {
    Success(RpcSuccessResponse),
    Error(RpcErrorResponse),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RpcSuccessResponse {
    pub jsonrpc: Option<String>,
    pub result: String,
    pub id: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RpcErrorResponse {
    pub jsonrpc: Option<String>,
    pub error: RpcError,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RpcError {
    pub code: u16,
    pub message: String,
}
