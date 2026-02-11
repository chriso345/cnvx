pub(crate) fn version(
    _command: &crate::args::VersionCommand,
) -> Result<(), Box<dyn std::error::Error>> {
    let ver = get_version();
    println!("CNVX {}", ver);

    Ok(())
}

pub fn get_version() -> CnvxVersion {
    let raw = env!("CNVX_VERSION");
    let commit = option_env!("CNVX_COMMIT_SHA");
    CnvxVersion { raw, commit }
}

#[derive(Debug, Clone, Copy)]
pub struct CnvxVersion {
    /// Raw, unmodified version string.
    raw: &'static str,
    /// The raw commit hash.
    commit: Option<&'static str>,
}

impl std::fmt::Display for CnvxVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(commit) = self.commit {
            let short_sha = &commit.trim()[..7];
            write!(f, "v{}+{}.dirty", self.raw, short_sha)
        } else {
            write!(f, "v{}", self.raw)
        }
    }
}
