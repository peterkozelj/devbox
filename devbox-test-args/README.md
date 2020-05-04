[![Crates.io](https://img.shields.io/crates/v/devbox-test-args.svg)](https://crates.io/crates/devbox-test-args)
[![MIT licensed](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

# devbox-test-args

Adds parametrization capabilty to `#[test]` via `#[args]` attribute macro.

## To install via umbrella devbox crate

```toml
[dev-dependencies]
devbox = { version = "0.1" }
```

## Simplest example

```rust
#[args(
    char_a: 'a';
    char_b: 'b' ! "wrong char";
)]
#[test]
fn parametrized_test_for(character:_) {
    assert_eq!('a', character, "wrong char");
}
```

Check [#\[args\]] attribute for full example and usage specification.

[#\[args\]]: https://doc.rust-lang.org/devbox_test_args/attr.args.html

## License

Licensed under MIT license ([LICENSE](LICENSE) or https://opensource.org/licenses/MIT)

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
Devbox by you, shall be licensed as MIT, without any additional terms or conditions.
