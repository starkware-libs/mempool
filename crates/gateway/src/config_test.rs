use assert_matches::assert_matches;
use papyrus_config::validators::{ParsedValidationError, ParsedValidationErrors};
use validator::Validate;

use super::StatelessTransactionValidatorConfig;
use crate::compiler_version::VersionId;

#[test]
fn test_stateless_transaction_validator_config_validation() {
    let mut config = StatelessTransactionValidatorConfig {
        max_sierra_version: VersionId { major: 1, minor: 2, patch: 0 },
        ..Default::default()
    };
    assert_matches!(config.validate(), Ok(()));

    config.max_sierra_version.patch = 1;
    assert_matches!(config.validate().unwrap_err(), validation_errors => {
        let parsed_errors = ParsedValidationErrors::from(validation_errors);
        assert_eq!(parsed_errors.0.len(), 1);
        let parsed_validation_error = &parsed_errors.0[0];
        assert_matches!(
            parsed_validation_error,
            ParsedValidationError { param_path, code, message, params}
            if (
                param_path == "__all__" &&
                code == "Invalid max_sierra_version." &&
                params.is_empty() &&
                *message == Some("The patch version should be 0.".to_string())
            )
        )
    });
}
