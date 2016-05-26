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

macro_rules! lazy_load {
	($s:expr, $e:ident) => {{
		if !$s.data.lock().unwrap().$e.available(){
			try!($s.load());
		}
		Ok(try!($s.data.lock().unwrap().$e.get()).clone())
	}};
}

pub mod account;
pub mod application;
pub mod call;
pub mod call_event;
pub mod client;
pub mod conference;
pub mod endpoint;
pub mod error;
pub mod message;
pub mod message_event;
pub mod number;

mod auth_token;
mod bridge;
mod domain;
mod environment;
mod lazy;
mod media;
mod util;
mod voice;


pub type CatapultResult<T> = Result<T, error::CatapultError>;

pub use account::Account;
pub use application::Application;
pub use auth_token::AuthToken;
pub use error::CatapultError;
pub use bridge::Bridge;
pub use call::Call;
pub use call::CallQuery;
pub use call_event::CallEvent;
pub use client::Client;
pub use conference::Conference;
pub use domain::Domain;
pub use endpoint::Endpoint;
pub use environment::Environment;
pub use media::Media;
pub use message::Message;
pub use number::Number;
pub use voice::Voice;


pub mod prelude{
	pub use {Account, CatapultError, CatapultResult, Client, Environment, Voice};
	pub use {application, call, call_event, message, number};
}

