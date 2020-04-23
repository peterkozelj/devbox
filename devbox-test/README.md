[![Crates.io](https://img.shields.io/crates/v/devbox-test.svg)](https://crates.io/crates/devbox-test)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

# devbox-test

A library of test related macros and functions

## Test macro
This is a macro complementing Rust's standard `#[test]` macro that adds test parametrization
capabilty to test functions. Macro emits a new standard Rust test for each set of named
arguments (also called a case):
```rust
#[devbox_test::test(
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

## License

Licensed under MIT license ([LICENSE](LICENSE) or https://opensource.org/licenses/MIT)

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
Devbox by you, shall be licensed as MIT, without any additional terms or conditions.
