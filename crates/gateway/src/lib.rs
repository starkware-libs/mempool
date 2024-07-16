pub mod communication;
mod compilation;
mod compiler_version;
pub mod config;
pub mod errors;
pub mod gateway;
mod rpc_objects;
mod rpc_state_reader;
#[cfg(test)]
mod rpc_state_reader_test;
mod state_reader;
#[cfg(test)]
mod state_reader_test_utils;
mod stateful_transaction_validator;
mod stateless_transaction_validator;
#[cfg(test)]
mod test_utils;
mod utils;

// TODO(yair): Find a way to detect changes in the RPC spec
// (https://github.com/starkware-libs/starknet-specs/blob/master/api/starknet_write_api.json).
// Currently, the spec file is copied manually.
