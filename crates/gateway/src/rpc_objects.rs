use serde::{Deserialize, Serialize};
use starknet_api::core::ContractAddress;

#[derive(Deserialize, Serialize)]
pub enum Tag {
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
    Tag(Tag),
    // There are additional options in the spec that are not implemented here
}

#[derive(Serialize, Deserialize)]
pub struct GetNonceParams {
    pub block_id: BlockId,
    pub contract_address: ContractAddress,
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
