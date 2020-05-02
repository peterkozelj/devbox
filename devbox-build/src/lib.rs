//! Small utility library for writing Rust build sripts (build.rs).
//!
//! It contains a collection of types that should make writing of build sripts a bit easier.
//! It focuses on file manipulation and generation while checking file stamps to avoid unnecessary
//! build steps. It can replace makefiles for simple things like copying some files and invoking
//! some external compiler or tool like NPM when things have changes.
//!
//! Most methods do not return a Result but simply panic with consistent error messages stoping
//! Cargo build which is desired behaviour for build sripts in most cases. For situations when
//! you do want to recover from errors or implement a better error reporting most method have a twin
//! method suffixed with '_result' that return `Result` values instead.
//!
//! # To install via umbrella devbox crate
//!
//! ```toml
//! [dev-dependencies]
//! devbox = { version = "0.1" }
//! ```
//!
//! # Example
//!
//! Example build sripts builds a web application located in project `root/webapp` inside the target
//! directory using NPM by installing all JS dependencies and running build script through NPM.
//! Built web app is then embedded into Rust binary by packing it as a Rust source code. All
//! the steps are done only on clean builds or when relevant resources change since last build.
//!
//! ```rust,no_run
//! # use devbox_build::{Build, Cmd, Resource};
//!
//! pub fn main() {
//!
//!     let build = Build::new();
//!
//!     //-- Setup web app build dir inside of Rust target directory ----------------------
//!
//!     // Rust does not allow changes outside target directory, so setup a webapp build
//!     // directory using links to source files where nodejs and company can do it's thing
//!
//!     let webrs = build.out_dir().file("webapp.rs");
//!     let websrc = build.manifest_dir().dir("webapp");
//!     let webwrk = build.out_dir().dir("webapp_build");
//!     let webwrk_pkg = webwrk.file("package.json");
//!     let webwrk_pkl = webwrk.file("package-lock.json");
//!     let webwrk_ndm = webwrk.dir("node_modules");
//!     let webwrk_dst = webwrk.dir("dist");
//!
//!     for unit in websrc.content("*") {
//!         unit.link_from_inside(&webwrk);
//!     }
//!
//!     //-- Build webapp using NPM -------------------------------------------------------
//!
//!     let npm = Cmd::new("npm").arg("--prefix").arg(webwrk.path());
//!
//!     webwrk_ndm.mk_from("Install WebApp node packages", &webwrk_pkg + &webwrk_pkl, ||{
//!         npm.clone().arg("install").run();
//!         webwrk_ndm.touch();
//!     });
//!
//!     webwrk_dst.mk_from("Build WebApp using webpack", &webwrk.content("**"), || {
//!         npm.clone().arg("run").arg("build").run();
//!         webwrk_dst.touch();
//!     });
//!
//!     //-- Package webapp into server binary as Rust source code ------------------------
//!
//!     webrs.mk_from("Embed WebApp build into binary", &webwrk_dst, || {
//!         let mappings = webwrk_dst.files("**").into_iter().map(|file|
//!             format!(r#""{}" => Some(include_bytes!("{}")),"#,
//!                 file.path().strip_prefix(&webwrk_dst.path()).unwrap().to_str().unwrap(),
//!                 file.path().to_str().unwrap())
//!         ).fold("".to_owned(), |result, ref s| result + s + "\n" );
//!
//!         webrs.rewrite(format!(r"
//!             pub fn file(path: &str) -> Option<&'static [u8]> {{
//!                 match path {{
//!                     {}
//!                     &_ => None,
//!                 }}
//!             }}
//!         ", mappings));
//!     });
//! }
//! ```

mod build;
mod cmd;
mod fs;
mod res;

pub use build::Build;
pub use cmd::Cmd;
pub use fs::{File, Dir, Unit};
pub use res::{Resource, Set};
