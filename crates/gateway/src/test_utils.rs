use starknet_types_core::felt::Felt;

use crate::compiler_version::VersionId;

pub fn create_sierra_program(version_id: &VersionId) -> Vec<Felt> {
    vec![
        Felt::from(u64::try_from(version_id.major).unwrap()),
        Felt::from(u64::try_from(version_id.minor).unwrap()),
        Felt::from(u64::try_from(version_id.patch).unwrap()),
    ]
}
