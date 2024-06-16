use cairo_lang_starknet_classes::compiler_version::VersionId as CairoLangVersionId;
use starknet_api::hash::StarkFelt;
use validator::Validate;

#[derive(Debug)]
pub enum VersionIdError {
    InvalidVersion { index: usize, felt_status: String },
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
