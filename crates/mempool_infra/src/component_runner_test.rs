use assert_matches::assert_matches;
use async_trait::async_trait;
use pretty_assertions::assert_eq;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::component_runner::{ComponentRunner, ComponentStartError};
use papyrus_config::dumping::{ser_param, SerializeConfig};
use papyrus_config::ParamPrivacyInput;
use papyrus_config::{ParamPath, SerializedParam};

mod test_component1 {
    use super::*;

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

    #[derive(Debug)]
    pub struct TestComponent1 {
        pub config: TestConfig1,
    }

    impl TestComponent1 {
        pub fn new(config: TestConfig1) -> Self {
            Self { config }
        }
        async fn local_start(&self) -> Result<(), tokio::io::Error> {
            println!("TestComponent1::local_start(), config: {:#?}", self.config);
            Ok(())
        }
    }

    #[async_trait]
    impl ComponentRunner<TestConfig1> for TestComponent1 {
        async fn start(config: TestConfig1) -> Result<(), ComponentStartError> {
            let component = TestComponent1::new(config);
            println!("TestComponent1::start(), component: {:#?}", component);
            component.local_start().await.map_err(|_err| ComponentStartError::InternalComponentError)
        }
    }

}

mod test_component2 {
    use super::*;

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

    #[derive(Debug)]
    pub struct TestComponent2 {
        pub config: TestConfig2,
    }

    #[async_trait]
    impl ComponentRunner<TestConfig2> for TestComponent2 {
        async fn start(config: TestConfig2) -> Result<(), ComponentStartError> {
            let component = TestComponent2 { config: config.clone() };
            println!("TestComponent2::start(): component: {:#?}", component);
            if config.config2 == 43 {
                Err(ComponentStartError::InternalComponentError)
            } else if config.config2 == 44 {
                Err(ComponentStartError::ComponentConfigError)
            } else {
                Ok(())
            }
        }
    }

}


use test_component1::{TestConfig1, TestComponent1};

#[tokio::test]
async fn test_testruner1() {
    let test_config = TestConfig1 { config1: true };
    assert_matches!(
        TestComponent1::start(test_config.clone()).await,
        Ok(())
    );
}

use test_component2::{TestConfig2, TestComponent2};

#[tokio::test]
async fn test_testruner2() {
    let test_config = TestConfig2 { config2: 42 };
    assert_matches!(
        TestComponent2::start(test_config).await,
        Ok(())
    );

    let test_config = TestConfig2 { config2: 43 };
    assert_matches!(TestComponent2::start(test_config).await, Err(e) => {
        assert_eq!(e, ComponentStartError::InternalComponentError);
    });

    let test_config = TestConfig2 { config2: 44 };
    assert_matches!(TestComponent2::start(test_config).await, Err(e) => {
        assert_eq!(e, ComponentStartError::ComponentConfigError);
    });
}

