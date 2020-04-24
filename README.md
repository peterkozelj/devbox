[![Crates.io](https://img.shields.io/crates/v/devbox.svg)](https://crates.io/crates/devbox)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

# devbox

A collection of small build related librarires intended to be used as `dev-dependencies`

## Install
To get started, add the following to Cargo.toml under `dev-dependencies`:
```toml
[dev-dependencies]
devbox = { version = "0.1" }
```

## Features

### Test macro
This is a macro complementing Rust's standard `#[test]` macro that adds test parametrization
capabilty to test functions. Macro emits a new standard Rust test for each set of named
arguments (also called a case):
```rust
#[devbox::test(
    char_a: 97, 'a';
    char_b: 98, 'b';
)]
fn parametrized_test_for(code:_, letter:_) {
    assert_eq!(code, letter as u8);
}
```

Should produce:
```txt
test parametrized_test_for__char_a ... ok
test parametrized_test_for__char_b ... ok
```

Macro can be applied mutiple times to a test function forming a cartesian product.
See the macro documentation for detailed description and example.

### Build srcipt
A module for writing Rust build sripts (build.rs) contains a collection of types that should
make writing of build sripts a bit easier. It focuses on file manipulation and generation while
checking file stamps to avoid unnecessary build steps. It resembels *Makefiles* in what it
tries to do.

Example build sripts builds a web application located in project `root/webapp` inside the target
directory using NPM by installing all JS dependencies and running build script through NPM.
Built web app is then embedded into Rust binary by packing it as a Rust source code. All
the steps are done only on clean build or if relevant resources were changed since last build.

```rust
use std::{io::Write, process::Command};
use devbox::build::{Build, MkFrom};
let build = Build::new();

//-- Setup web app build dir inside of Rust target directory -----------------------------------

// Rust does not allow changes outside target directory, so setup a webapp build directoy
// using links to source files where nodejs and company can do it's thing

let webrs = build.out.file("webapp.rs");
let websrc = build.root.dir("webapp");
let webwrk = build.out.dir("webapp_build").linked_from_inside(&build.target);
let webwrk_pkg = webwrk.file("package.json");
let webwrk_pkl = webwrk.file("package-lock.json");
let webwrk_ndm = webwrk.dir("node_modules");
let webwrk_dst = webwrk.dir("dist");

for unit in websrc.content("*") {
    unit.link_from_inside(&webwrk);
}

//-- Build webapp using NPM --------------------------------------------------------------------

webwrk_ndm.mk_from("Install WebApp node packages", &webwrk_pkg + &webwrk_pkl, ||{
    Command::new("npm")
        .arg("--prefix").arg(webwrk.path())
        .arg("install")
        .status().unwrap();
    webwrk_ndm.touch();
});

webwrk_dst.mk_from("Build WebApp using webpack", &webwrk.content("**"), || {
    Command::new("npm")
        .arg("--prefix").arg(webwrk.path())
        .arg("run")
        .arg("build")
        .status().unwrap();
    webwrk_dst.touch();
});

//-- Package webapp into server binary as Rust source code -------------------------------------

webrs.mk_from_safe("Embed WebApp build into binary", &webwrk_dst, || {
    let mappings = webwrk_dst.files("**").into_iter().map(|file|
        format!(r#""{}" => Some(include_bytes!("{}")),"#,
            file.path().strip_prefix(&webwrk_dst.path()).unwrap().to_str().unwrap(),
            file.path().to_str().unwrap())
    ).fold("".to_owned(), |result, ref s| result + s + "\n" );

    webrs.create().write_all(format!(r"
        pub fn file(path: &str) -> Option<&'static [u8]> {{
            match path {{
                {}
                &_ => None,
            }}
        }}
    ", mappings).as_bytes())?;

    Ok(())
});
```

## License

Licensed under MIT license ([LICENSE](LICENSE) or https://opensource.org/licenses/MIT)

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
Devbox by you, shall be licensed as MIT, without any additional terms or conditions.
