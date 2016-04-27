use hyper;
use std::convert;
use rustc_serialize::json;
use std::io;
use url;
use lazy::LazyError;
use std::string::FromUtf8Error;

#[derive(Debug)]
pub enum CatapultError{
	EncoderError(json::EncoderError),
	DecoderError(json::DecoderError),
	NetworkError(hyper::error::Error),
	IoError(io::Error),
	ApiError(ApiError),
	InvalidUrl,
	InternalError(String),
	Unexpected(String),
	BadInput(String),
	Utf8Error
}

#[derive(Debug, RustcDecodable)]
pub struct ApiError{
	category: String,
	code: String,
	message: String
}

impl convert::From<hyper::error::Error> for CatapultError{
	fn from(err: hyper::error::Error) -> CatapultError{
		CatapultError::NetworkError(err)
	}
}
//impl convert::From<FromUtf8Error> for CatapultError{
//	fn from(err: FromUtf8Error) -> CatapultError{
//		CatapultError::SerializationError("Utf8Error".to_string())
//	}
//}
impl convert::From<FromUtf8Error> for CatapultError{
	fn from(_: FromUtf8Error) -> CatapultError{
		CatapultError::Utf8Error
	}
}
impl convert::From<json::EncoderError> for CatapultError{
	fn from(err: json::EncoderError) -> CatapultError{
		CatapultError::EncoderError(err)
	}
}
impl convert::From<json::DecoderError> for CatapultError{
	fn from(err: json::DecoderError) -> CatapultError{
		CatapultError::DecoderError(err)
	}
}
impl convert::From<io::Error> for CatapultError{
	fn from(err: io::Error) -> CatapultError{
		CatapultError::IoError(err)
	}
}
impl convert::From<url::ParseError> for CatapultError{
	fn from(_: url::ParseError) -> CatapultError{
		CatapultError::InvalidUrl
	}
}

impl convert::From<LazyError> for CatapultError{
	fn from(_: LazyError) -> CatapultError{
		CatapultError::InternalError("Lazy::get() failed".to_string())
	}
}

impl CatapultError{
	pub fn api_error(msg: &str) -> CatapultError{
		match json::decode(msg){
			Ok(err) => CatapultError::ApiError(err),
			Err(err) => CatapultError::DecoderError(err)
		}
	}
	pub fn unexpected(msg: &str) -> CatapultError{
		CatapultError::Unexpected(msg.to_owned())
	}
	pub fn bad_input(msg: &str) -> CatapultError{
		CatapultError::BadInput(msg.to_owned())
	}
}
