use clap::Command;
use papyrus_config::dumping::SerializeConfig;
use papyrus_config::loading::load_and_process_config;
use serde::Deserialize;
use std::fmt::Debug;
use std::fs::File;
use std::path::PathBuf;
use validator::Validate;

pub fn test_valid_config_body<
    T: for<'a> Deserialize<'a> + SerializeConfig + Validate + PartialEq + Debug,
>(
    expected_config: T,
    config_file_path: PathBuf,
    fix: bool,
) {
    if fix {
        expected_config
            .dump_to_file(&vec![], config_file_path.to_str().unwrap())
            .unwrap();
    }

    let config_file = File::open(config_file_path).unwrap();
    let loaded_config =
        load_and_process_config::<T>(config_file, Command::new(""), vec![]).unwrap();

    assert!(loaded_config.validate().is_ok());
    assert_eq!(loaded_config, expected_config);
}
