use std::net::SocketAddr;
use std::sync::Arc;

use blockifier::abi::abi_utils::get_fee_token_var_address;
use blockifier::context::{BlockContext, ChainInfo};
use blockifier::test_utils::contracts::FeatureContract;
use blockifier::test_utils::{
    CairoVersion, BALANCE, CURRENT_BLOCK_TIMESTAMP, DEFAULT_ETH_L1_GAS_PRICE,
    DEFAULT_STRK_L1_GAS_PRICE, TEST_SEQUENCER_ADDRESS,
};
use blockifier::transaction::objects::FeeType;
use cairo_lang_starknet_classes::casm_contract_class::CasmContractClass;
use indexmap::{indexmap, IndexMap};
use lazy_static::lazy_static;
use papyrus_common::pending_classes::PendingClasses;
use papyrus_common::BlockHashAndNumber;
use papyrus_rpc::{run_server, RpcConfig};
use papyrus_storage::body::BodyStorageWriter;
use papyrus_storage::class::ClassStorageWriter;
use papyrus_storage::compiled_class::CasmStorageWriter;
use papyrus_storage::header::HeaderStorageWriter;
use papyrus_storage::state::StateStorageWriter;
use papyrus_storage::{open_storage, StorageConfig, StorageReader};
use starknet_api::block::{
    BlockBody, BlockHeader, BlockNumber, BlockTimestamp, GasPrice, GasPricePerToken,
};
use starknet_api::core::{ClassHash, ContractAddress, PatriciaKey, SequencerContractAddress};
use starknet_api::deprecated_contract_class::ContractClass as DeprecatedContractClass;
use starknet_api::hash::{StarkFelt, StarkHash};
use starknet_api::state::{StorageKey, ThinStateDiff};
use starknet_api::{contract_address, patricia_key, stark_felt};
use starknet_client::reader::PendingData;
use starknet_gateway::config::RpcStateReaderConfig;
use starknet_gateway::rpc_state_reader::RpcStateReaderFactory;
use strum::IntoEnumIterator;
use tempfile::tempdir;
use test_utils::starknet_api_test_utils::{deploy_account_tx, deployed_account_contract_address};
use tokio::sync::RwLock;

type ContractClassesMap =
    (Vec<(ClassHash, DeprecatedContractClass)>, Vec<(ClassHash, CasmContractClass)>);

lazy_static! {
    static ref DEPLOY_ACCCOUNT_TX_CONTRACT_ADDRESS: ContractAddress = {
        let deploy_tx = deploy_account_tx();
        deployed_account_contract_address(&deploy_tx)
    };
}

/// StateReader for integration tests.
///
/// Creates a papyrus storage reader and Spawns a papyrus rpc server for it.
/// A variable number of identical accounts and test contracts are initialized and funded.
pub async fn rpc_test_state_reader_factory(
    n_initialized_account_contracts: u16,
) -> RpcStateReaderFactory {
    const RPC_SPEC_VERION: &str = "V0_7";
    const JSON_RPC_VERSION: &str = "2.0";
    let block_context = BlockContext::create_for_testing();
    let account_contract_cairo0 = FeatureContract::AccountWithoutValidations(CairoVersion::Cairo0);
    let test_contract_cairo0 = FeatureContract::TestContract(CairoVersion::Cairo0);
    let account_contract_cairo1 = FeatureContract::AccountWithoutValidations(CairoVersion::Cairo1);
    let test_contract_cairo1 = FeatureContract::TestContract(CairoVersion::Cairo1);
    let erc20 = FeatureContract::ERC20;
    let fund_accounts = vec![*DEPLOY_ACCCOUNT_TX_CONTRACT_ADDRESS];

    let storage_reader = initialize_papyrus_test_state(
        block_context.chain_info(),
        BALANCE,
        &[
            (erc20, 1),
            (account_contract_cairo0, 1),
            (test_contract_cairo0, 1),
            (account_contract_cairo1, n_initialized_account_contracts),
            (test_contract_cairo1, 1),
        ],
        fund_accounts,
    );
    let addr = run_papyrus_rpc_server(storage_reader).await;

    RpcStateReaderFactory {
        config: RpcStateReaderConfig {
            url: format!("http://{addr:?}/rpc/{RPC_SPEC_VERION}"),
            json_rpc_version: JSON_RPC_VERSION.to_string(),
        },
    }
}

fn initialize_papyrus_test_state(
    chain_info: &ChainInfo,
    initial_balances: u128,
    contract_instances: &[(FeatureContract, u16)],
    fund_additional_accounts: Vec<ContractAddress>,
) -> StorageReader {
    let state_diff = prepare_state_diff(
        chain_info,
        contract_instances,
        initial_balances,
        fund_additional_accounts,
    );

    let (cairo0_contract_classes, cairo1_contract_classes) =
        prepare_compiled_contract_classes(contract_instances);

    write_state_to_papyrus_storage(state_diff, &cairo0_contract_classes, &cairo1_contract_classes)
}

fn prepare_state_diff(
    chain_info: &ChainInfo,
    contract_instances: &[(FeatureContract, u16)],
    initial_balances: u128,
    fund_accounts: Vec<ContractAddress>,
) -> ThinStateDiff {
    let erc20 = FeatureContract::ERC20;
    let erc20_class_hash = erc20.get_class_hash();

    // Declare and deploy ERC20 contracts.
    let mut deployed_contracts = indexmap! {
        chain_info.fee_token_address(&FeeType::Eth) => erc20_class_hash,
        chain_info.fee_token_address(&FeeType::Strk) => erc20_class_hash
    };
    let mut deprecated_declared_classes = Vec::from([erc20.get_class_hash()]);

    let mut storage_diffs = IndexMap::new();
    let mut declared_classes = IndexMap::new();
    for (contract, n_instances) in contract_instances.iter() {
        for instance in 0..*n_instances {
            // Declare and deploy the contracts
            match cairo_version(contract) {
                CairoVersion::Cairo0 => {
                    deprecated_declared_classes.push(contract.get_class_hash());
                }
                CairoVersion::Cairo1 => {
                    declared_classes.insert(contract.get_class_hash(), Default::default());
                }
            }
            deployed_contracts
                .insert(contract.get_instance_address(instance), contract.get_class_hash());
            fund_feature_account_contract(
                &mut storage_diffs,
                contract,
                instance,
                initial_balances,
                chain_info,
            );
        }
    }

    fund_accounts.iter().for_each(|address| {
        fund_account(&mut storage_diffs, address, initial_balances, chain_info)
    });

    ThinStateDiff {
        storage_diffs,
        deployed_contracts,
        declared_classes,
        deprecated_declared_classes,
        ..Default::default()
    }
}

fn prepare_compiled_contract_classes(
    contract_instances: &[(FeatureContract, u16)],
) -> ContractClassesMap {
    let mut cairo0_contract_classes = Vec::new();
    let mut cairo1_contract_classes = Vec::new();
    for (contract, _) in contract_instances.iter() {
        match cairo_version(contract) {
            CairoVersion::Cairo0 => {
                cairo0_contract_classes.push((
                    contract.get_class_hash(),
                    serde_json::from_str(&contract.get_raw_class()).unwrap(),
                ));
            }
            CairoVersion::Cairo1 => {
                cairo1_contract_classes.push((
                    contract.get_class_hash(),
                    serde_json::from_str(&contract.get_raw_class()).unwrap(),
                ));
            }
        }
    }
    (cairo0_contract_classes, cairo1_contract_classes)
}

fn write_state_to_papyrus_storage(
    state_diff: ThinStateDiff,
    cairo0_contract_classes: &[(ClassHash, DeprecatedContractClass)],
    cairo1_contract_classes: &[(ClassHash, CasmContractClass)],
) -> StorageReader {
    let block_number = BlockNumber(0);
    let block_header = test_block_header(block_number);

    let mut storage_config = StorageConfig::default();
    let tempdir = tempdir().unwrap();
    storage_config.db_config.path_prefix = tempdir.path().to_path_buf();
    let (storage_reader, mut storage_writer) = open_storage(storage_config).unwrap();

    let cairo0_contract_classes =
        cairo0_contract_classes.iter().map(|(x, y)| (*x, y)).collect::<Vec<_>>();

    let mut write_txn = storage_writer.begin_rw_txn().unwrap();
    for (class_hash, casm) in cairo1_contract_classes {
        write_txn = write_txn.append_casm(class_hash, casm).unwrap();
    }
    write_txn
        .append_header(block_number, &block_header)
        .unwrap()
        .append_body(block_number, BlockBody::default())
        .unwrap()
        .append_state_diff(block_number, state_diff)
        .unwrap()
        .append_classes(block_number, &[], cairo0_contract_classes.as_slice())
        .unwrap()
        .commit()
        .unwrap();

    storage_reader
}

// TODO (Yael 19/6/2024): make this function public in Blockifier and remove it from here.
fn cairo_version(contract: &FeatureContract) -> CairoVersion {
    match contract {
        FeatureContract::AccountWithLongValidate(version)
        | FeatureContract::AccountWithoutValidations(version)
        | FeatureContract::Empty(version)
        | FeatureContract::FaultyAccount(version)
        | FeatureContract::TestContract(version) => *version,
        FeatureContract::ERC20 => CairoVersion::Cairo0,
        _ => panic!("{contract:?} contract has no configurable version."),
    }
}

fn test_block_header(block_number: BlockNumber) -> BlockHeader {
    BlockHeader {
        block_number,
        sequencer: SequencerContractAddress(contract_address!(TEST_SEQUENCER_ADDRESS)),
        l1_gas_price: GasPricePerToken {
            price_in_wei: GasPrice(DEFAULT_ETH_L1_GAS_PRICE),
            price_in_fri: GasPrice(DEFAULT_STRK_L1_GAS_PRICE),
        },
        l1_data_gas_price: GasPricePerToken {
            price_in_wei: GasPrice(DEFAULT_ETH_L1_GAS_PRICE),
            price_in_fri: GasPrice(DEFAULT_STRK_L1_GAS_PRICE),
        },
        timestamp: BlockTimestamp(CURRENT_BLOCK_TIMESTAMP),
        ..Default::default()
    }
}

fn fund_feature_account_contract(
    storage_diffs: &mut IndexMap<ContractAddress, IndexMap<StorageKey, StarkFelt>>,
    contract: &FeatureContract,
    instance: u16,
    initial_balances: u128,
    chain_info: &ChainInfo,
) {
    match contract {
        FeatureContract::AccountWithLongValidate(_)
        | FeatureContract::AccountWithoutValidations(_)
        | FeatureContract::FaultyAccount(_) => {
            fund_account(
                storage_diffs,
                &contract.get_instance_address(instance),
                initial_balances,
                chain_info,
            );
        }
        _ => (),
    }
}

fn fund_account(
    storage_diffs: &mut IndexMap<ContractAddress, IndexMap<StorageKey, StarkFelt>>,
    account_address: &ContractAddress,
    initial_balances: u128,
    chain_info: &ChainInfo,
) {
    let key_value = indexmap! {
        get_fee_token_var_address(*account_address) => stark_felt!(initial_balances),
    };
    for fee_type in FeeType::iter() {
        storage_diffs
            .entry(chain_info.fee_token_address(&fee_type))
            .or_default()
            .extend(key_value.clone());
    }
}

// TODO(Yael 5/6/2024): remove this function and use the one from papyrus test utils once we have
// mono-repo.
fn get_test_highest_block() -> Arc<RwLock<Option<BlockHashAndNumber>>> {
    Arc::new(RwLock::new(None))
}

// TODO(Yael 5/6/2024): remove this function and use the one from papyrus test utils once we have
// mono-repo.
fn get_test_pending_data() -> Arc<RwLock<PendingData>> {
    Arc::new(RwLock::new(PendingData::default()))
}

// TODO(Yael 5/6/2024): remove this function and use the one from papyrus test utils once we have
// mono-repo.
fn get_test_pending_classes() -> Arc<RwLock<PendingClasses>> {
    Arc::new(RwLock::new(PendingClasses::default()))
}

async fn run_papyrus_rpc_server(storage_reader: StorageReader) -> SocketAddr {
    let rpc_config = RpcConfig::default();
    let (addr, handle) = run_server(
        &rpc_config,
        get_test_highest_block(),
        get_test_pending_data(),
        get_test_pending_classes(),
        storage_reader,
        "NODE VERSION",
    )
    .await
    .unwrap();
    // Spawn the server handle to keep the server running, otherwise the server will stop once the
    // handler is out of scope.
    tokio::spawn(handle.stopped());
    addr
}
