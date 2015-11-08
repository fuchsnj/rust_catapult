#![doc(html_logo_url = "https://raw.githubusercontent.com/bandwidthcom/rust-bandwidth/master/img/bandwidth.jpg")]

extern crate hyper;
extern crate rustc_serialize;
extern crate url;

macro_rules! json {
  (null) => (json::Null);
  ([ $($values:tt),* ]) => (json::List(vec![ $(json!($values)),* ]));
  ({ $($keys:expr => $values:tt),* }) => ({
    let kv_pairs = vec![ $(($keys.to_string(), json!($values))),* ];
    ::rustc_serialize::json::Json::Object(kv_pairs.into_iter().collect())
  });
  ($ex:expr) => ({
  	  use ::rustc_serialize::json::ToJson;
	  $ex.to_json()
  });
}

pub mod client;
pub mod application;
pub mod error;
pub mod number;
pub mod domain;
pub mod endpoint;
pub mod auth_token;
pub mod call;
pub mod call_event;
pub mod bridge;
pub mod voice;

mod lazy;
mod util;
mod environment;

pub type BResult<T> = Result<T, error::BError>;

pub use client::Client;
pub use application::Application;
pub use error::BError;
pub use number::Number;
pub use domain::Domain;
pub use endpoint::Endpoint;
pub use auth_token::AuthToken;
pub use call::Call;
pub use call_event::CallEvent;
pub use bridge::Bridge;
pub use voice::Voice;
pub use environment::Environment;

pub mod prelude{
	pub use {Application, BError, BResult, Bridge, Call, CallEvent, Client, Domain,
		Endpoint, Environment, Number, Voice};
	pub use {application, bridge, call, call_event, endpoint, number, voice};
}

