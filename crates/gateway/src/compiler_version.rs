use cairo_lang_starknet_classes::compiler_version::VersionId as CairoLangVersionId;
use starknet_api::hash::StarkFelt;
use thiserror::Error;
use validator::Validate;

#[derive(Debug, Error)]
#[cfg_attr(test, derive(PartialEq))]
pub enum VersionIdError {
    #[error("{message}")]
    InvalidVersion { message: String },
}

// TODO(Arni): Share this struct with the Cairo lang crate.
#[derive(Clone, Copy, Debug, Validate, PartialEq)]
pub struct VersionId {
    pub major: usize,
    pub minor: usize,
    pub patch: usize,
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
        let sierra_program_length = sierra_program.len();

        if sierra_program_length < 3 {
            return Err(VersionIdError::InvalidVersion {
                message: format!(
                    "Sierra program is too short. got program of length {} which is not long \
                     enough to hold the version field.",
                    sierra_program_length
                ),
            });
        }

        fn get_version_component(
            index: usize,
            sierra_program: &[StarkFelt],
        ) -> Result<usize, VersionIdError> {
            let felt = sierra_program[index];
            felt.try_into().map_err(|_| VersionIdError::InvalidVersion {
                message: format!("Felt at index {index} out of range of usize. Got: {}", felt),
            })
        }

        Ok(VersionId {
            major: get_version_component(0, sierra_program)?,
            minor: get_version_component(1, sierra_program)?,
            patch: get_version_component(2, sierra_program)?,
        })
    }
}
