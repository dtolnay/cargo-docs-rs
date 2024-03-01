# cargo docs-rs

[<img alt="github" src="https://img.shields.io/badge/github-dtolnay/cargo--docs--rs-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/dtolnay/cargo-docs-rs)
[<img alt="crates.io" src="https://img.shields.io/crates/v/cargo-docs-rs.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/cargo-docs-rs)
[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/dtolnay/cargo-docs-rs/ci.yml?branch=master&style=for-the-badge" height="20">](https://github.com/dtolnay/cargo-docs-rs/actions?query=branch%3Amaster)
[<img alt="test" src="https://img.shields.io/github/actions/workflow/status/dtolnay/cargo-docs-rs/test.yml?branch=master&style=for-the-badge" height="20">](https://github.com/dtolnay/cargo-docs-rs/actions?query=branch%3Amaster)

Run `cargo rustdoc` with the same options that would be used by docs.rs, taking
into account the `package.metadata.docs.rs` configured in Cargo.toml.

## Example

If the following GitHub Actions job succeeds, it's likely that docs.rs will
succeed in building your crate's documentation.

```yaml
# .github/workflows/ci.yml

name: test suite
on: [push, pull_request]

jobs:
  # ...

  doc:
    name: Documentation
    runs-on: ubuntu-latest
    env:
      RUSTDOCFLAGS: -Dwarnings
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@nightly
      - uses: dtolnay/install@cargo-docs-rs
      - run: cargo docs-rs
```

<br>

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>
