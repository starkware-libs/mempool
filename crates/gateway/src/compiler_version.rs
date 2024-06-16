use cairo_lang_starknet_classes::compiler_version::VersionId as CairoLangVersionId;
use validator::Validate;

// TODO(Arni): Share this struct with the Cairo lang crate.
#[derive(Clone, Copy, Debug, Validate, PartialEq)]
pub struct VersionId {
    pub major: usize,
    pub minor: usize,
    pub patch: usize,
}

impl From<VersionId> for CairoLangVersionId {
    fn from(version: VersionId) -> Self {
        CairoLangVersionId { major: version.major, minor: version.minor, patch: version.patch }
    }
}

impl std::fmt::Display for VersionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let version_id: CairoLangVersionId = (*self).into();
        version_id.fmt(f)
    }
}
