use git_version::git_version;
use target_info::Target;

lazy_static::lazy_static! {
    pub static ref SHORT_VERSION: String = VERSION.replace("HyveDA/", "");
}

/// Returns the current version of this build of HyveDA.
///
/// A plus-sign (`+`) is appended to the git commit if the tree is dirty.
/// Commit hash is omitted if the sources don't include git information.
///
/// ## Example
///
/// `HyveDA/v1.5.1-67da032+`
pub const VERSION: &str = git_version!(
    args = [
        "--always",
        "--dirty=+",
        "--abbrev=7",
        // NOTE: using --match instead of --exclude for compatibility with old Git
        "--match=thiswillnevermatchlol"
    ],
    prefix = "HyveDA/v5.3.0-",
    fallback = "HyveDA/v5.3.0"
);

/// Returns the first eight characters of the latest commit hash for this build.
///
/// No indication is given if the tree is dirty. This is part of the standard
/// for reporting the client version to the execution engine.
pub const COMMIT_PREFIX: &str = git_version!(
    args = [
        "--always",
        "--abbrev=8",
        // NOTE: using --match instead of --exclude for compatibility with old Git
        "--match=thiswillnevermatchlol"
    ],
    prefix = "",
    suffix = "",
    cargo_prefix = "",
    cargo_suffix = "",
    fallback = "00000000"
);

/// Returns `VERSION`, but with platform information appended to the end.
///
/// ## Example
///
/// `HyveDA/v1.5.1-67da032+/x86_64-linux`
pub fn version_with_platform() -> String {
    format!("{}/{}-{}", VERSION, Target::arch(), Target::os())
}

#[cfg(test)]
mod test {
    use super::*;
    use regex::Regex;

    #[test]
    fn version_formatting() {
        let re = Regex::new(r"^HyveDA/v[0-9]+\.[0-9]+\.[0-9]+(-rc.[0-9])?(-[[:xdigit:]]{7})?\+?$")
            .unwrap();
        assert!(re.is_match(VERSION), "version doesn't match regex: {}", VERSION);
    }
}
