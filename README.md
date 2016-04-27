Bandwidth Communication SDK
====
[![Build Status](https://travis-ci.org/bandwidthcom/rust-bandwidth.svg?branch=master)](https://travis-ci.org/bandwidthcom/rust-bandwidth)

A Rust SDK for [Catapult](http://ap.bandwidth.com/) (Bandwidth's Application Platform)
that focuses on ease of use and efficiency.

Note that this SDK is not yet feature complete, and is still undergoing major changes.

## Documentation

http://bandwidthcom.github.io/rust-bandwidth/catapult

## Usage

This will be uploaded to crates.io when it is more stable.
For now, add the following to your `Cargo.toml`

```toml
[dependencies.catapult]
git = "https://github.com/bandwidthcom/rust-bandwidth"
```

and this to your crate root:

```rust
extern crate catapult;
```

## Quick Start

Create a client, which is required for everything else.
If you don't have credentials, you can signup [here](https://catapult.inetwork.com/pages/signup.jsf)
```rust
let client = Client::new(
	"u-0123456789abcdefg", //user id
	"t-0123456789abcdefg", //token
	"0123456789abcdefg0123456789abcdefg" //secret
);
```

Create an application, which lets you receive incoming events such as calls and messages.
```rust
let app = client.build_application(
	"My Communication App", //application name
	"http://mydomain.com/call", //incoming call callback url
	"http://mydomain.com/msg" //incoming message callback url
)
.disable_auto_answer() //other options can be found in the docs
.create().unwrap(); //must call create to actually create the resource
```

Send a message
```rust
let msg = client.build_message(
	"+19195550000", //from number (Must be a Catapult number in your account)
	"+19195551111", // to number (E164 format)
	"This is a message!" //text content
)
.tag("You can use the tag to set custom meta-data for a message")
.media("http://some-publicly-accessible-uri")
.media("http://or-an-internal-media-url") //sum of media must be less than 2,000,000 bytes
.create().unwrap();
let msg_id = msg.get_id();//this is the ID of your created message
```

