use std::clone::Clone;
use std::net::SocketAddr;
use std::panic;
use std::sync::Arc;

use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use blockifier::execution::contract_class::{ClassInfo, ContractClassV1};
use cairo_felt::Felt252;
use cairo_lang_starknet_classes::contract_class::{
    ContractClass as CairoLangContractClass, ContractEntryPoint as CairoLangContractEntryPoint,
    ContractEntryPoints as CairoLangContractEntryPoints,
};
use cairo_lang_utils::bigint::BigUintAsHex;
use num_bigint::BigUint;
use starknet_api::core::CompiledClassHash;
use starknet_api::external_transaction::{
    ContractClass as StarknetApiContractClass, EntryPointByType as StarknetApiEntryPointByType,
    ExternalDeclareTransaction, ExternalTransaction,
};
use starknet_api::hash::StarkFelt;
use starknet_api::state::EntryPoint as StarknetApiEntryPoint;
use starknet_api::transaction::TransactionHash;
use starknet_mempool_types::mempool_types::{Account, MempoolInput, SharedMempoolClient};
use starknet_sierra_compile::compile::{compile_sierra_to_casm, CompilationUtilError};

use crate::config::{GatewayConfig, GatewayNetworkConfig};
use crate::errors::{GatewayError, GatewayRunError};
use crate::starknet_api_test_utils::get_sender_address;
use crate::state_reader::StateReaderFactory;
use crate::stateful_transaction_validator::StatefulTransactionValidator;
use crate::stateless_transaction_validator::StatelessTransactionValidator;
use crate::utils::external_tx_to_thin_tx;

#[cfg(test)]
#[path = "gateway_test.rs"]
pub mod gateway_test;

pub type GatewayResult<T> = Result<T, GatewayError>;

pub struct Gateway {
    config: GatewayConfig,
    app_state: AppState,
}

#[derive(Clone)]
pub struct AppState {
    pub stateless_tx_validator: StatelessTransactionValidator,
    pub stateful_tx_validator: Arc<StatefulTransactionValidator>,
    pub state_reader_factory: Arc<dyn StateReaderFactory>,
    pub mempool_client: SharedMempoolClient,
}

impl Gateway {
    pub fn new(
        config: GatewayConfig,
        state_reader_factory: Arc<dyn StateReaderFactory>,
        mempool_client: SharedMempoolClient,
    ) -> Self {
        let app_state = AppState {
            stateless_tx_validator: StatelessTransactionValidator {
                config: config.stateless_tx_validator_config.clone(),
            },
            stateful_tx_validator: Arc::new(StatefulTransactionValidator {
                config: config.stateful_tx_validator_config.clone(),
            }),
            state_reader_factory,
            mempool_client,
        };
        Gateway { config, app_state }
    }

    pub async fn run(self) -> Result<(), GatewayRunError> {
        // Parses the bind address from GatewayConfig, returning an error for invalid addresses.
        let GatewayNetworkConfig { ip, port } = self.config.network_config;
        let addr = SocketAddr::new(ip, port);
        let app = self.app();

        // Create a server that runs forever.
        Ok(axum::Server::bind(&addr).serve(app.into_make_service()).await?)
    }

    pub fn app(self) -> Router {
        Router::new()
            .route("/is_alive", get(is_alive))
            .route("/add_tx", post(add_tx))
            .with_state(self.app_state)
        // TODO: when we need to configure the router, like adding banned ips, add it here via
        // `with_state`.
    }
}

// Gateway handlers.

async fn is_alive() -> GatewayResult<String> {
    unimplemented!("Future handling should be implemented here.");
}

async fn add_tx(
    State(app_state): State<AppState>,
    Json(tx): Json<ExternalTransaction>,
) -> GatewayResult<Json<TransactionHash>> {
    let mempool_input = tokio::task::spawn_blocking(move || {
        process_tx(
            app_state.stateless_tx_validator,
            app_state.stateful_tx_validator.as_ref(),
            app_state.state_reader_factory.as_ref(),
            tx,
        )
    })
    .await??;

    let tx_hash = mempool_input.tx.tx_hash;

    app_state
        .mempool_client
        .add_tx(mempool_input)
        .await
        .map_err(|e| GatewayError::MessageSendError(e.to_string()))?;
    // TODO: Also return `ContractAddress` for deploy and `ClassHash` for Declare.
    Ok(Json(tx_hash))
}

fn process_tx(
    stateless_tx_validator: StatelessTransactionValidator,
    stateful_tx_validator: &StatefulTransactionValidator,
    state_reader_factory: &dyn StateReaderFactory,
    tx: ExternalTransaction,
) -> GatewayResult<MempoolInput> {
    // TODO(Arni, 1/5/2024): Perform congestion control.

    // Perform stateless validations.
    stateless_tx_validator.validate(&tx)?;

    // Compile Sierra to Casm.
    let optional_class_info = get_optional_class_info(&tx)?;

    // TODO(Yael, 19/5/2024): pass the relevant deploy_account_hash.
    let tx_hash =
        stateful_tx_validator.run_validate(state_reader_factory, &tx, optional_class_info, None)?;

    // TODO(Arni): Add the Sierra and the Casm to the mempool input.
    Ok(MempoolInput {
        tx: external_tx_to_thin_tx(&tx, tx_hash),
        account: Account { sender_address: get_sender_address(&tx), ..Default::default() },
    })
}

fn get_optional_class_info(tx: &ExternalTransaction) -> GatewayResult<Option<ClassInfo>> {
    let declare_tx = match tx {
        ExternalTransaction::Declare(declare_tx) => declare_tx,
        _ => return Ok(None),
    };

    let ExternalDeclareTransaction::V3(tx) = declare_tx;
    let starknet_api_contract_class = tx.contract_class.clone();
    let sierra_program_length = starknet_api_contract_class.sierra_program.len();
    let abi_length = starknet_api_contract_class.abi.len();
    let cairo_lang_contract_class =
        starknet_api_contract_class_to_cairo_lang_contract_class(starknet_api_contract_class);

    // Compile Sierra to Casm.
    let catch_unwind_result =
        panic::catch_unwind(|| compile_sierra_to_casm(cairo_lang_contract_class));
    let casm_contract_class = match catch_unwind_result {
        Ok(compilation_result) => compilation_result?,
        Err(_) => {
            // TODO(Arni): Log the panic.
            return Err(GatewayError::CompilationUtilError(CompilationUtilError::CompilationPanic));
        }
    };

    // TODO: Handle unwrap.
    let hash_result =
        CompiledClassHash(felt_to_stark_felt(&casm_contract_class.compiled_class_hash()));
    if hash_result != tx.compiled_class_hash {
        return Err(GatewayError::CompiledClassHashMismatch {
            supplied: tx.compiled_class_hash,
            hash_result,
        });
    }

    // Convert Casm contract class to Starknet contract class directly.
    let raw_contract_class = serde_json::to_string(&casm_contract_class).unwrap();
    let contact_class_v1: ContractClassV1 =
        ContractClassV1::try_from_json_string(&raw_contract_class).unwrap();

    let blockifier_contract_class = contact_class_v1.into();
    let class_info = ClassInfo::new(&blockifier_contract_class, sierra_program_length, abi_length)
        .expect("Expects a Cairo 1 contract class");
    Ok(Some(class_info))
}

fn starknet_api_contract_class_to_cairo_lang_contract_class(
    starknet_api_contract_class: StarknetApiContractClass,
) -> CairoLangContractClass {
    let sierra_program = starknet_api_contract_class
        .sierra_program
        .into_iter()
        .map(stark_felt_to_big_uint_as_hex)
        .collect();
    let contract_class_version = starknet_api_contract_class.contract_class_version;
    let entry_points_by_type = entry_point_by_type_to_contract_entry_points(
        starknet_api_contract_class.entry_points_by_type,
    );

    CairoLangContractClass {
        sierra_program,
        sierra_program_debug_info: None,
        contract_class_version,
        entry_points_by_type,
        // The Abi is irrelevant to the computlation.
        abi: None,
    }
}

fn entry_point_by_type_to_contract_entry_points(
    entry_points_by_type: StarknetApiEntryPointByType,
) -> CairoLangContractEntryPoints {
    let StarknetApiEntryPointByType { constructor, external, l1handler } = entry_points_by_type;
    CairoLangContractEntryPoints {
        external: starknet_api_entry_points_to_contract_entry_points(external),
        l1_handler: starknet_api_entry_points_to_contract_entry_points(l1handler),
        constructor: starknet_api_entry_points_to_contract_entry_points(constructor),
    }
}

fn starknet_api_entry_points_to_contract_entry_points(
    entry_points: Vec<StarknetApiEntryPoint>,
) -> Vec<CairoLangContractEntryPoint> {
    entry_points.into_iter().map(entry_point_into_contract_entry_point).collect()
}

fn entry_point_into_contract_entry_point(
    entry_point: StarknetApiEntryPoint,
) -> CairoLangContractEntryPoint {
    CairoLangContractEntryPoint {
        selector: stark_felt_to_big_uint(entry_point.selector.0),
        function_idx: entry_point.function_idx.0,
    }
}

fn stark_felt_to_big_uint_as_hex(stark_felt: StarkFelt) -> BigUintAsHex {
    BigUintAsHex { value: stark_felt_to_big_uint(stark_felt) }
}

fn stark_felt_to_big_uint(stark_felt: StarkFelt) -> BigUint {
    // When the value of radix is 256, the following function always returns a Some value.
    let radix = 256;
    BigUint::from_radix_be(stark_felt.bytes(), radix).expect("Unexpected None value.")
}

// TODO(Arni): Move to starknet_api. This function already exists in the blockifier repo.
pub fn felt_to_stark_felt(felt: &Felt252) -> StarkFelt {
    let biguint = format!("{:#x}", felt.to_biguint());
    StarkFelt::try_from(biguint.as_str()).expect("Felt252 must be in StarkFelt's range.")
}
