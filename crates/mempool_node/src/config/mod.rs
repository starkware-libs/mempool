#[cfg(test)]
mod config_test;

use std::collections::BTreeMap;
use std::fs::File;
use std::path::Path;

use clap::Command;
use papyrus_config::dumping::{
    append_sub_config_name,
    ser_param,
    // ser_optional_sub_config,
    // ser_pointer_target_param,
    SerializeConfig,
};
use papyrus_config::loading::load_and_process_config;
use papyrus_config::ParamPrivacyInput;
use papyrus_config::{ConfigError, ParamPath, SerializedParam};
use serde::{Deserialize, Serialize};
use starknet_gateway::GatewayConfig;
use validator::{Validate, ValidationError};

use crate::version::VERSION_FULL;

// The path of the default configuration file, provided as part of the crate.
pub const DEFAULT_CONFIG_PATH: &str = "config/default_config.json";

/// The single crate configuration.
#[derive(Clone, Debug, Serialize, Deserialize, Validate, PartialEq)]
pub struct ModuleExecutionConfig {
    pub execute: bool,
}

impl SerializeConfig for ModuleExecutionConfig {
    fn dump(&self) -> BTreeMap<ParamPath, SerializedParam> {
        BTreeMap::from_iter([ser_param(
            "execute",
            &self.execute,
            "The module execution flag.",
            ParamPrivacyInput::Public,
        )])
    }
}

impl Default for ModuleExecutionConfig {
    fn default() -> Self {
        Self { execute: true }
    }
}

/// The crates configuration.
#[derive(Clone, Debug, Serialize, Deserialize, Validate, PartialEq, Default)]
#[validate(schema(function = "validate_modules_config"))]
pub struct ModulesConfig {
    pub gateway_module: ModuleExecutionConfig,
    pub mempool_module: ModuleExecutionConfig,
}

impl SerializeConfig for ModulesConfig {
    fn dump(&self) -> BTreeMap<ParamPath, SerializedParam> {
        #[allow(unused_mut)]
        let mut sub_configs = vec![
            append_sub_config_name(self.gateway_module.dump(), "gateway_module"),
            append_sub_config_name(self.mempool_module.dump(), "mempool_module"),
        ];

        sub_configs.into_iter().flatten().collect()
    }
}

pub fn validate_modules_config(modules: &ModulesConfig) -> Result<(), ValidationError> {
    if modules.gateway_module.execute || modules.mempool_module.execute {
        return Ok(());
    }

    let mut error = ValidationError::new("Invalid modules configuration.");
    error.message = Some("At least one module should be allowed to execute.".into());
    Err(error)
}

/// The configurations of the various components of the node.
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Validate, Default)]
pub struct MemPoolNodeConfig {
    #[validate]
    pub modules: ModulesConfig,
    #[validate]
    pub gateway_config: GatewayConfig,
}

impl SerializeConfig for MemPoolNodeConfig {
    fn dump(&self) -> BTreeMap<ParamPath, SerializedParam> {
        #[allow(unused_mut)]
        let mut sub_configs = vec![
            append_sub_config_name(self.modules.dump(), "crates"),
            append_sub_config_name(self.gateway_config.dump(), "gateway_config"),
        ];

        sub_configs.into_iter().flatten().collect()
    }
}

impl MemPoolNodeConfig {
    /// Creates a config object. Selects the values from the default file and from resources with
    /// higher priority.
    fn load_and_process_config_file(
        args: Vec<String>,
        config_file_name: Option<&str>,
    ) -> Result<Self, ConfigError> {
        let config_file_name = match config_file_name {
            Some(file_name) => file_name,
            None => DEFAULT_CONFIG_PATH,
        };

        let default_config_file = File::open(Path::new(config_file_name))?;
        load_and_process_config(default_config_file, node_command(), args)
    }

    pub fn load_and_process(args: Vec<String>) -> Result<Self, ConfigError> {
        Self::load_and_process_config_file(args, None)
    }
    pub fn load_and_process_file(
        args: Vec<String>,
        config_file_name: &str,
    ) -> Result<Self, ConfigError> {
        Self::load_and_process_config_file(args, Some(config_file_name))
    }
}

/// The command line interface of this node.
pub fn node_command() -> Command {
    Command::new("Mempool")
        .version(VERSION_FULL)
        .about("Mempool is a StarkNet mempool node written in Rust.")
}
