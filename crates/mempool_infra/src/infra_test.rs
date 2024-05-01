use assert_matches::assert_matches;
use async_trait::async_trait;
use pretty_assertions::assert_eq;
use serde::{Deserialize, Serialize};

use std::collections::BTreeMap;

use crate::{ComponentRunner, ComponentStartError};
use papyrus_config::dumping::{ser_param, SerializeConfig};

use papyrus_config::ParamPrivacyInput;
use papyrus_config::{ParamPath, SerializedParam};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TestConfig1 {
    pub config1: bool,
}

impl SerializeConfig for TestConfig1 {
    fn dump(&self) -> BTreeMap<ParamPath, SerializedParam> {
        BTreeMap::from_iter([ser_param(
            "test1",
            &self.config1,
            "...",
            ParamPrivacyInput::Public,
        )])
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TestConfig2 {
    pub config2: u32,
}

impl SerializeConfig for TestConfig2 {
    fn dump(&self) -> BTreeMap<ParamPath, SerializedParam> {
        BTreeMap::from_iter([ser_param(
            "test2",
            &self.config2,
            "...",
            ParamPrivacyInput::Public,
        )])
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct TestConfig3 {
    pub config3: f64,
}

impl SerializeConfig for TestConfig3 {
    fn dump(&self) -> BTreeMap<ParamPath, SerializedParam> {
        BTreeMap::from_iter([ser_param(
            "test2",
            &self.config3,
            "...",
            ParamPrivacyInput::Public,
        )])
    }
}

trait ExtendedConfigTrait: SerializeConfig + std::fmt::Debug + Send + Sync {}

impl ExtendedConfigTrait for TestConfig1 {}
impl ExtendedConfigTrait for TestConfig2 {}

#[async_trait]
impl ComponentRunner for TestComponent1 {
    async fn start_component(&self) -> Result<(), ComponentStartError> {
        println!("TestComponent1::start_component(): {:#?}", self.config);
        Ok(())
    }
}

#[async_trait]
impl ComponentRunner for TestComponent2 {
    async fn start_component(&self) -> Result<(), ComponentStartError> {
        println!("TestComponent2::start_component(): {:#?}", self.config);
        if self.config.config2 == 42 {
            return Err(ComponentStartError::InternalComponentError);
        } else {
            return Ok(());
        };
    }
}

pub struct TestComponent1 {
    pub config: TestConfig1,
}

pub struct TestComponent2 {
    pub config: TestConfig2,
}

#[tokio::test]
async fn test_testruner1() {
    let test_config = TestConfig1 { config1: true };
    let test_component = TestComponent1 {
        config: test_config,
    };
    assert_matches!(test_component.start_component().await, Ok(()));
}

#[tokio::test]
async fn test_testruner2() {
    let test_config = TestConfig2 { config2: 16 };
    let test_component = TestComponent2 {
        config: test_config,
    };
    assert_matches!(test_component.start_component().await, Ok(()));
}

#[tokio::test]
async fn test_run_from_vector() {
    let test_config_1 = TestConfig1 { config1: true };
    let test_config_2 = TestConfig2 { config2: 17 };
    let erroneous_test_config_2 = TestConfig2 { config2: 42 };

    let test_component_1 = TestComponent1 {
        config: test_config_1,
    };
    let test_component_2 = TestComponent2 {
        config: test_config_2,
    };
    let erroneous_test_component_2 = TestComponent2 {
        config: erroneous_test_config_2,
    };

    let components: Vec<Box<dyn ComponentRunner>> = vec![
        Box::new(test_component_1),
        Box::new(test_component_2),
        Box::new(erroneous_test_component_2),
    ];

    let expected_results: Vec<Result<(), ComponentStartError>> = vec![
        Ok(()),
        Ok(()),
        Err(ComponentStartError::InternalComponentError),
    ];

    for (component, expected_result) in components.iter().zip(expected_results.iter()) {
        let ret_val = component.start_component().await;
        assert_eq!(ret_val, *expected_result);
    }
}
