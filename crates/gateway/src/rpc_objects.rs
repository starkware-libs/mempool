use std::num::NonZeroU128;

use blockifier::{
    blockifier::block::{BlockInfo, GasPrices},
    state::errors::StateError,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use starknet_api::{
    block::{BlockHash, BlockNumber, BlockTimestamp, GasPrice},
    core::{ContractAddress, GlobalRoot},
    data_availability::L1DataAvailabilityMode,
    transaction::TransactionHash,
};

// Starknet Spec error codes:
// TODO(yael 30/4/2024): consider turning these into an enum.
pub const RPC_ERROR_BLOCK_NOT_FOUND: u16 = 24;
pub const RPC_ERROR_CONTRACT_ADDRESS_NOT_FOUND: u16 = 20;

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub enum Tag {
    #[serde(rename = "latest")]
    Latest,
    #[serde(rename = "pending")]
    Pending,
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub enum BlockHashOrNumber {
    #[serde(rename = "block_hash")]
    Hash(BlockHash),
    #[serde(rename = "block_number")]
    Number(BlockNumber),
}

#[derive(Copy, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum BlockId {
    HashOrNumber(BlockHashOrNumber),
    Tag(Tag),
}

#[derive(Serialize, Deserialize)]
pub struct GetNonceParams {
    pub block_id: BlockId,
    pub contract_address: ContractAddress,
}

#[derive(Serialize, Deserialize)]
pub struct GetBlockWithTxHashesParams {
    pub block_id: BlockId,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum BlockStatus {
    #[serde(rename = "PENDING")]
    Pending,
    #[serde(rename = "ACCEPTED_ON_L2")]
    AcceptedOnL2,
    #[serde(rename = "ACCEPTED_ON_L1")]
    AcceptedOnL1,
    #[serde(rename = "REJECTED")]
    Rejected,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ResourcePrice {
    pub price_in_wei: GasPrice,
    pub price_in_fri: GasPrice,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BlockHeader {
    pub block_hash: BlockHash,
    pub parent_hash: BlockHash,
    pub block_number: BlockNumber,
    pub sequencer_address: ContractAddress,
    pub new_root: GlobalRoot,
    pub timestamp: BlockTimestamp,
    pub l1_gas_price: ResourcePrice,
    pub l1_data_gas_price: ResourcePrice,
    pub l1_da_mode: L1DataAvailabilityMode,
    pub starknet_version: String,
}

impl TryInto<BlockInfo> for BlockHeader {
    type Error = StateError;
    fn try_into(self) -> Result<BlockInfo, Self::Error> {
        Ok(BlockInfo {
            block_number: self.block_number,
            sequencer_address: self.sequencer_address,
            block_timestamp: self.timestamp,
            gas_prices: GasPrices {
                eth_l1_gas_price: parse_gas_price(self.l1_gas_price.price_in_wei)?,
                strk_l1_gas_price: parse_gas_price(self.l1_gas_price.price_in_fri)?,
                eth_l1_data_gas_price: parse_gas_price(self.l1_data_gas_price.price_in_wei)?,
                strk_l1_data_gas_price: parse_gas_price(self.l1_data_gas_price.price_in_fri)?,
            },
            use_kzg_da: matches!(self.l1_da_mode, L1DataAvailabilityMode::Blob),
        })
    }
}

fn parse_gas_price(gas_price: GasPrice) -> Result<NonZeroU128, StateError> {
    NonZeroU128::new(gas_price.0).ok_or(StateError::StateReadError(
        "Couldn't parse gas_price".to_string(),
    ))
}

#[derive(Debug, Deserialize, Serialize)]
pub struct BlockWithTxHashes {
    pub status: BlockStatus,
    #[serde(flatten)]
    pub header: BlockHeader,
    pub transactions: Vec<TransactionHash>,
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
    pub result: Value,
    pub id: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RpcErrorResponse {
    pub jsonrpc: Option<String>,
    pub error: RpcSpecError,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RpcSpecError {
    pub code: u16,
    pub message: String,
}
