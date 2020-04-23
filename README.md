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
TBD

## License

Licensed under MIT license ([LICENSE](LICENSE) or https://opensource.org/licenses/MIT)

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
Devbox by you, shall be licensed as MIT, without any additional terms or conditions.
