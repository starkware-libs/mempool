use std::collections::BTreeMap;

use cairo_lang_starknet_classes::compiler_version::VersionId as CairoLangVersionId;
use papyrus_config::dumping::{ser_param, SerializeConfig};
use papyrus_config::{ParamPath, ParamPrivacyInput, SerializedParam};
use serde::{Deserialize, Serialize};
use starknet_api::hash::StarkFelt;
use validator::Validate;

#[derive(Debug)]
pub enum VersionIdError {
    InvalidVersion { index: usize, felt_status: String },
}

// TODO(Arni): Share this struct with the Cairo lang crate.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, Validate, PartialEq)]
pub struct VersionId {
    pub major: usize,
    pub minor: usize,
    pub patch: usize,
}

impl VersionId {
    pub const MIN: Self = Self { major: 0, minor: 0, patch: 0 };
    pub const MAX: Self = Self { major: usize::MAX, minor: usize::MAX, patch: usize::MAX };
}

impl From<&VersionId> for CairoLangVersionId {
    fn from(version: &VersionId) -> Self {
        CairoLangVersionId { major: version.major, minor: version.minor, patch: version.patch }
    }
}

impl std::fmt::Display for VersionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        CairoLangVersionId::from(self).fmt(f)
    }
}

impl VersionId {
    pub fn from_sierra_program(sierra_program: &[StarkFelt]) -> Result<Self, VersionIdError> {
        let length_of_version = sierra_program.len();

        if length_of_version < 3 {
            return Err(VersionIdError::InvalidVersion {
                index: length_of_version,
                felt_status: "missing".into(),
            });
        }

        fn get_version_component_from_sierra_program(
            index: usize,
            sierra_program: &[StarkFelt],
        ) -> Result<usize, VersionIdError> {
            let felt = sierra_program[index];
            felt.try_into().map_err(|_| VersionIdError::InvalidVersion {
                index,
                felt_status: format!("out of range: {}", felt),
            })
        }

        Ok(VersionId {
            major: get_version_component_from_sierra_program(0, sierra_program)?,
            minor: get_version_component_from_sierra_program(1, sierra_program)?,
            patch: get_version_component_from_sierra_program(2, sierra_program)?,
        })
    }
}

#[cfg(test)]
impl VersionId {
    pub fn into_sierra_program(&self) -> Vec<StarkFelt> {
        vec![
            StarkFelt::from(u64::try_from(self.major).unwrap()),
            StarkFelt::from(u64::try_from(self.minor).unwrap()),
            StarkFelt::from(u64::try_from(self.patch).unwrap()),
        ]
    }
}

impl PartialOrd for VersionId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.major != other.major {
            return Some(self.major.cmp(&other.major));
        }
        if self.minor != other.minor {
            return Some(self.minor.cmp(&other.minor));
        }
        self.patch.partial_cmp(&other.patch)
    }
}

impl SerializeConfig for VersionId {
    fn dump(&self) -> BTreeMap<ParamPath, SerializedParam> {
        BTreeMap::from_iter([
            ser_param(
                "major",
                &self.major,
                "The major version of the configuration.",
                ParamPrivacyInput::Public,
            ),
            ser_param(
                "minor",
                &self.minor,
                "The minor version of the configuration.",
                ParamPrivacyInput::Public,
            ),
            ser_param(
                "patch",
                &self.patch,
                "The patch version of the configuration.",
                ParamPrivacyInput::Public,
            ),
        ])
    }
}
