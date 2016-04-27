use {CatapultResult, CatapultError};
use client::{EmptyResponse, ByteResponse};
use client::Client;
use hyper::header;
use lazy::Lazy;
use lazy::Lazy::*;
use std::sync::{Arc, Mutex};
use hyper::header::Headers;

pub trait ToBytes{
	fn to_bytes(self) -> Vec<u8>;
}
impl ToBytes for Vec<u8>{
	fn to_bytes(self) -> Vec<u8>{
		self
	}
}
impl ToBytes for String{
	fn to_bytes(self) -> Vec<u8>{
		self.into_bytes()
	}
}
impl<'a> ToBytes for &'a str{
	fn to_bytes(self) -> Vec<u8>{
		self.to_owned().into_bytes()
	}
}

#[derive(Clone)]
pub struct Media{
	client: Client,
	filename: String,
	data: Arc<Mutex<Data>>
}
impl Media{
	pub fn create<T>(client: &Client, filename: &str, data: T) -> CatapultResult<Media>
	where T: ToBytes{
		
		let bytes = data.to_bytes();
		let path = "users/".to_string() + &client.get_user_id() + "/media/" + &filename;
		let res:EmptyResponse = try!(client.raw_put_request(&path, (), bytes));
		let data = try!(Self::load_metadata_from_headers(&res.headers));
		
		Ok(Media{
			client: client.clone(),
			filename: filename.to_owned(),
			data: Arc::new(Mutex::new(data))
		})
	}
	pub fn get(client: &Client, filename: &str) -> Media{
		Media{
			client: client.clone(),
			filename: filename.to_owned(),
			data: Arc::new(Mutex::new(Data{
				content_type: NotLoaded,
				date: NotLoaded,
				content_length: NotLoaded
			}))
		}
	}
	fn load_metadata_from_headers(headers: &Headers) -> CatapultResult<Data>{
		let content_type;
		let content_length;
		let date;
		match headers.get::<header::ContentType>(){
			Some(content_type_header) => {
				let mime = &content_type_header.0;
				content_type = mime.0.as_str().to_owned() + "/" + mime.1.as_str();
			},
			None => return Err(CatapultError::unexpected("Content-Type header missing on media"))
		};
		match headers.get::<header::ContentLength>(){
			Some(content_length_header) => {
				content_length = content_length_header.0;
			},
			None => return Err(CatapultError::unexpected("Content-Length header missing on media"))
		};
		match headers.get::<header::Date>(){
			Some(date_header) => {
				date = date_header.0.to_string();
			},
			None => return Err(CatapultError::unexpected("Date header missing on media"))
		};

		Ok(Data{
			content_type: Available(content_type),
			date: Available(date),
			content_length: Available(content_length)
		})
	}
	pub fn load(&self) -> CatapultResult<()>{
		let path = "users/".to_string() + &self.client.get_user_id() + "/media/" + &self.filename;
		let res:EmptyResponse = try!(self.client.raw_head_request(&path, (), ()));
		let mut data = self.data.lock().unwrap();
		*data = try!(Self::load_metadata_from_headers(&res.headers));
		Ok(())
	}
	
	/* Getters */
	pub fn get_client(&self) -> Client{
		self.client.clone()
	}
	pub fn get_filename(&self) -> String{
		self.filename.clone()
	}
	
	pub fn get_content_type(&self) -> CatapultResult<String>{
		if !self.data.lock().unwrap().content_type.available(){
			try!(self.load());
		}
		Ok(try!(self.data.lock().unwrap().content_type.get()).clone())
	}
	pub fn get_date(&self) -> CatapultResult<String>{
		if !self.data.lock().unwrap().date.available(){
			try!(self.load());
		}
		Ok(try!(self.data.lock().unwrap().date.get()).clone())
	}
	pub fn get_content_length(&self) -> CatapultResult<u64>{
		if !self.data.lock().unwrap().content_length.available(){
			try!(self.load());
		}
		Ok(try!(self.data.lock().unwrap().content_length.get()).clone())
	}
	pub fn get_contents(&self) -> CatapultResult<Vec<u8>>{
		let path = "users/".to_string() + &self.client.get_user_id() + "/media/" + &self.filename;
		let res:ByteResponse = try!(self.client.raw_get_request(&path, (), ()));
		try!(Self::load_metadata_from_headers(&res.headers));
		Ok(res.body)
	}
	pub fn get_contents_as_string(&self) -> CatapultResult<String>{
		let body = try!(self.get_contents());
		Ok(try!(String::from_utf8(body)))
	}
	
}
struct Data{
	content_type: Lazy<String>,
	date: Lazy<String>,
	content_length: Lazy<u64>
}