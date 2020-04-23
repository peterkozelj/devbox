use std::env;
use std::path::{Path};

mod fs;
mod res;

pub use fs::{File, Dir, Unit};
pub use res::{MkFrom, Resource, Set};

////////////////////////////////////////////////////////////////////////////////////////////////////
// Build
////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
pub struct Build {
    pub root: Dir,
    pub out: Dir,
    pub profile: String,
}

impl Build {
    pub fn new() -> Self {
        Build {
            out: Dir::new(Path::new(&env::var("OUT_DIR").unwrap()).to_path_buf()),
            root: Dir::new(env::current_dir().unwrap()),
            profile: env::var("PROFILE").unwrap(),
        }
    }
}