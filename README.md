Bandwidth Communication SDK
====
[![Build Status](https://travis-ci.org/bandwidthcom/rust-bandwidth.svg?branch=master)](https://travis-ci.org/bandwidthcom/rust-bandwidth)

A Rust SDK for [Bandwidth's Communication API](http://ap.bandwidth.com/)
that focuses on ease of use and efficiency.

Note that this SDK is not yet feature complete, and is still undergoing major changes.

## Documentation

http://bandwidthcom.github.io/rust-bandwidth

## Usage

This will be uploaded to crates.io when it is more stable.
For now, add the following to your `Cargo.toml`

```toml
[dependencies.bandwidth_communication]
git = "https://github.com/bandwidthcom/rust-bandwidth"
```

and this to your crate root:

```rust
extern crate bandwidth_communication;
```

## Quick Start

Creating a client, which is required for everything else.
```
let client = Client::new(
	"u-0123456789abcdefg", //user id
	"t-0123456789abcdefg", //token
	"0123456789abcdefg0123456789abcdefg" //secret
);
```

Create an application, which lets you receive incoming events such as calls and messages.
```
let app = client.build_application(
	"My Communication App", //application name
	"http://mydomain.com/call", //incoming call callback url
	"http://mydomain.com/msg" //incoming message callback url
)
.disable_auto_answer() //other options can be found in the docs
.create().unwrap(); //must call create to actually create the resource
```


