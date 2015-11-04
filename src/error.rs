use hyper;
use std::convert;
use rustc_serialize::json;
use std::io;
use url;
use lazy::LazyError;

#[derive(Debug)]
pub enum BError{
	EncoderError(json::EncoderError),
	DecoderError(json::DecoderError),
	NetworkError(hyper::error::Error),
	IoError(io::Error),
	ApiError(ApiError),
	InvalidUrl,
	InternalError(String),
	Unexpected(String)
}

#[derive(Debug, RustcDecodable)]
pub struct ApiError{
	category: String,
	code: String,
	message: String
}

impl convert::From<hyper::error::Error> for BError{
	fn from(err: hyper::error::Error) -> BError{
		BError::NetworkError(err)
	}
}
//impl convert::From<FromUtf8Error> for BError{
//	fn from(err: FromUtf8Error) -> BError{
//		BError::SerializationError("Utf8Error".to_string())
//	}
//}

impl convert::From<json::EncoderError> for BError{
	fn from(err: json::EncoderError) -> BError{
		BError::EncoderError(err)
	}
}
impl convert::From<json::DecoderError> for BError{
	fn from(err: json::DecoderError) -> BError{
		BError::DecoderError(err)
	}
}
impl convert::From<io::Error> for BError{
	fn from(err: io::Error) -> BError{
		BError::IoError(err)
	}
}
impl convert::From<url::ParseError> for BError{
	fn from(_: url::ParseError) -> BError{
		BError::InvalidUrl
	}
}

impl convert::From<LazyError> for BError{
	fn from(_: LazyError) -> BError{
		BError::InternalError("Lazy::get() failed".to_string())
	}
}

impl BError{
	pub fn api_error(msg: &str) -> BError{
		match json::decode(msg){
			Ok(err) => BError::ApiError(err),
			Err(err) => BError::DecoderError(err)
		}
	}
	pub fn unexpected(msg: &str) -> BError{
		BError::Unexpected(msg.to_string())
	}
}
