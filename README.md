Bandwidth Application Platform SDK
====

A Rust SDK for [Bandwidth's Communication API](https://catapult.inetwork.com)
that focuses on ease of use and efficiency.

## Documentation

This will eventually be hosted. For now you can generate documentation locally
```
cargo doc
```

## Usage

For now, add the following to your `Cargo.toml`

```toml
[dependencies.bandwidth]
git = "https://github.com/inetCatapult/rust-bandwidth.git"
```

and this to your crate root:

```rust
extern crate bandwidth_communication;
```
