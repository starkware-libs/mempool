#![allow(unused_imports)]
#![allow(dead_code)]

use assert_matches::assert_matches;
use async_trait::async_trait;
use pretty_assertions::assert_eq;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::BTreeMap;

use crate::{get_config, ComponentRunner, ComponentStartError, ExtendConfig};
use papyrus_config::dumping::{ser_param, SerializeConfig};
use papyrus_config::loading::load_and_process_config;
use papyrus_config::ParamPrivacyInput;
use papyrus_config::{ConfigError, ParamPath, SerializedParam};

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

struct TestComponent1 {}

#[async_trait]
impl ComponentRunner for TestComponent1 {
    async fn start_component(
        &self,
        config: Option<Box<&(dyn ExtendConfig + Sync + Send)>>,
    ) -> Result<(), ComponentStartError> {
        let config = match config {
            Some(config) => config,
            None => return Err(ComponentStartError::ComponentConfigError),
        };
        let component_config: &TestConfig1 = match config.as_any().downcast_ref() {
            Some(config) => config,
            None => return Err(ComponentStartError::ComponentConfigError),
        };
        println!("TestComponent1::start_component(): {:#?}", component_config);
        Ok(())
    }
}

struct TestComponent2 {}

#[async_trait]
impl ComponentRunner for TestComponent2 {
    async fn start_component(
        &self,
        config: Option<Box<&(dyn ExtendConfig + Sync + Send)>>,
    ) -> Result<(), ComponentStartError> {
        let component_config: &TestConfig2;
        if let Some(config) = get_config(config) {
            component_config = config;
        } else {
            return Err(ComponentStartError::ComponentConfigError);
        }
        println!("TestComponent2::start_component(): {:#?}", component_config);

        if component_config.config2 != 42 {
            Err(ComponentStartError::InternalComponentError)
        } else {
            Ok(())
        }
    }
}

struct TestComponent3 {}

#[async_trait]
impl ComponentRunner for TestComponent3 {
    async fn start_component(
        &self,
        config: Option<Box<&(dyn ExtendConfig + Sync + Send)>>,
    ) -> Result<(), ComponentStartError> {
        let component_config: &TestConfig3;
        if let Some(config) = get_config(config) {
            component_config = config;
        } else {
            return Err(ComponentStartError::ComponentConfigError);
        }

        println!("TestComponent3::start_component(): {:#?}", component_config);

        Ok(())
    }
}

static TEST_COMPONENT_1: TestComponent1 = TestComponent1 {};
static TEST_COMPONENT_2: TestComponent2 = TestComponent2 {};
static TEST_COMPONENT_3: TestComponent3 = TestComponent3 {};

#[tokio::test]
async fn test_testruner1() {
    let test_config = TestConfig1 { config1: true };
    assert_matches!(
        TEST_COMPONENT_1
            .start_component(Some(Box::new(&test_config)))
            .await,
        Ok(())
    );
}

#[tokio::test]
async fn test_testruner2() {
    let test_config = TestConfig2 { config2: 42 };
    assert_matches!(
        TEST_COMPONENT_2
            .start_component(Some(Box::new(&test_config)))
            .await,
        Ok(())
    );

    let test_config = TestConfig2 { config2: 43 };
    assert_matches!(TEST_COMPONENT_2.start_component(Some(Box::new(&test_config))).await, Err(e) => {
        assert_eq!(e, ComponentStartError::InternalComponentError);
    });

    let test_config = TestConfig1 { config1: true };
    assert_matches!(TEST_COMPONENT_2.start_component(Some(Box::new(&test_config))).await, Err(e) => {
        assert_eq!(e, ComponentStartError::ComponentConfigError);
    });
}

#[tokio::test]
async fn test_testruner3() {
    let test_config = TestConfig3 { config3: 1.1 };
    assert_matches!(
        TEST_COMPONENT_3
            .start_component(Some(Box::new(&test_config)))
            .await,
        Ok(())
    );
}

pub struct CommonTestConfig {
    test_config_1: TestConfig1,
    test_config_2: TestConfig2,
    test_config_3: TestConfig3,
}

#[tokio::test]
async fn test_run_from_vector() {
    let common_config = CommonTestConfig {
        test_config_1: TestConfig1 { config1: true },
        test_config_2: TestConfig2 { config2: 43 },
        test_config_3: TestConfig3 { config3: 1.1 },
    };

    let components: Vec<Box<&dyn ComponentRunner>> = vec![
        Box::new(&TEST_COMPONENT_1),
        Box::new(&TEST_COMPONENT_2),
        Box::new(&TEST_COMPONENT_2),
        Box::new(&TEST_COMPONENT_3),
    ];

    let configs: Vec<Box<&(dyn ExtendConfig + Send + Sync)>> = vec![
        Box::new(&common_config.test_config_1),
        Box::new(&common_config.test_config_1),
        Box::new(&common_config.test_config_2),
        Box::new(&common_config.test_config_3),
    ];

    let ret_value = [
        Ok(()),
        Err(ComponentStartError::ComponentConfigError),
        Err(ComponentStartError::InternalComponentError),
        Ok(()),
    ];

    for (i, (component, config)) in components.into_iter().zip(configs.into_iter()).enumerate() {
        let ret_val = component.start_component(Some(config)).await;
        assert_eq!(ret_val, ret_value[i]);
    }
}
