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

## Test macro
Adds parametrization capabilty to `#[test]` via `#[args]` attribute macro.

```rust
#[args(
    char_a: 'a';
    char_b: 'b' ! "unexpected character";
)]
#[test]
fn parametrized_test_for(character:_) {
    assert_eq!('a', character, "unexpected character");
}
```

Check [#\[args\]] attribute for full example and usage specification.

[#\[args\]]: https://doc.rust-lang.org/devbox/test/attr.args.html

## Build srcipt
Small utility library for easier file manipulation and external tool invocation in Rust build
sripts (build.rs) that avoids rebuilding up-to-date build artifacts.

```rust

pub fn main() {

    let build = Build::new();
    let websrc = build.manifest_dir().dir("webapp");
    let webwrk = build.out_dir().dir("webapp_build");
    let webwrk_pkg = webwrk.file("package.json");
    let webwrk_pkl = webwrk.file("package-lock.json");

    for unit in websrc.content("*") {
        unit.link_from_inside(&webwrk);
    }

    let npm = Cmd::new("npm").arg("--prefix").arg(webwrk.path());

    webwrk.dir("node_modules").mk_from("Install WebApp node packages", &webwrk_pkg + &webwrk_pkl, ||{
        npm.clone().arg("install").run();
        webwrk.dir("node_modules").touch();
    });

    webwrk.dir("dist").mk_from("Build WebApp using webpack", &webwrk.content("**"), || {
        npm.clone().arg("run").arg("build").run();
        webwrk.dir("dist").touch();
    });
}
```

Check [build] module for fully working example and usage specification.

[build]: https://doc.rust-lang.org/devbox/build/index.html

## License

Licensed under MIT license ([LICENSE](LICENSE) or https://opensource.org/licenses/MIT)

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
Devbox by you, shall be licensed as MIT, without any additional terms or conditions.
