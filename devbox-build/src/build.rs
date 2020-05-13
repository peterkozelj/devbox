use std::env;

pub use super::cmd::Cmd;
pub use super::fs::{File, Dir, Unit};
pub use super::res::{Resource, Set};

//-- Build -----------------------------------------------------------------------------------------

/// Represents the build itself encaspsulating Cargo environment variables
///
/// You start the srcript by creating new `Build` instance which provides further access to
/// project directories and standard Cargo environment variables.
///
/// For more info on Cargo environement variables check [Cargo env variables]
/// [Cargo env variables]: https://doc.rust-lang.org/cargo/reference/environment-variables.html
///
#[derive(Debug)]
pub struct Build {
}

impl Build {

    /// Create new Build instance
    pub fn new() -> Self {
        Build {}
    }

    /// Current directory where the build has been run from
    pub fn current_dir(&self) -> Dir {
        Dir::new(env::current_dir().unwrap()).unwrap()
    }
}

/// Accessors for envionment variables set by Cargo when running the script
///
impl Build {

    /// Cargo executable command
    pub fn cargo_cmd(&self) -> Cmd { Cmd::new(env::var("CARGO").unwrap()) }

    /// Rust compler executable command
    pub fn rustc_cmd(&self) -> Cmd { Cmd::new(env::var("RUSTC").unwrap()) }

    /// Rust linker executable command
    pub fn rustc_linker_cmd(&self) -> Cmd { Cmd::new(env::var("RUSTC_LINKER").unwrap()) }

    /// Rust doc executable command
    pub fn rustdoc_cmd(&self) -> Cmd { Cmd::new(env::var("RUSTDOC").unwrap()) }

    /// Directory containing the project manifest
    pub fn manifest_dir(&self) -> Dir { Dir::new(env::var("CARGO_MANIFEST_DIR").unwrap()).unwrap() }

    /// Project manifest `links` value
    pub fn manifest_links(&self) -> String { env::var("CARGO_MANIFEST_LINKS").unwrap() }

    /// Directory in which all output should be placed
    pub fn out_dir(&self) -> Dir { Dir::new(env::var("OUT_DIR").unwrap()).unwrap() }

    /// True if cargo profile is `release` (run with --release)
    pub fn is_release_build(&self) -> bool { env::var("PROFILE").unwrap() == "release" }

    /// True if care is being build with `feature` enabled
    pub fn has_feature<P:AsRef<str>>(&self, feature: P) -> bool {
        Self::prefixed_env_var("CARGO_FEATURE_", feature).is_ok()
    }

    /// Achitecure triple of the machine running the build
    pub fn host_triple(&self) -> String { env::var("HOST").unwrap() }

    /// Achitecure triple of the build binaries
    pub fn target_triple(&self) -> String { env::var("TARGET").unwrap() }

    /// Number of threads to be used by the build
    pub fn num_jobs(&self) -> u16 { u16::from_str_radix(&env::var("NUM_JOBS").unwrap(), 10).unwrap() }

    /// Return configuration (check Cargo documentation above for more info)
    pub fn cfg<P:AsRef<str>>(&self, cfg: P) -> Option<String> {
        Self::prefixed_env_var("CARGO_CFG_", cfg).ok()
    }

    fn prefixed_env_var<P:AsRef<str>>(prefix: &str, name: P) -> Result<String, std::env::VarError> {
        let name = name.as_ref().to_owned().to_uppercase().replace("-", "_");
        env::var(format!("{}_{}", prefix, name))
    }
}