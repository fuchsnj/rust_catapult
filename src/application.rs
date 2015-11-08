use client::{EmptyResponse, JsonResponse, Client};
use BResult;
use util;
use lazy::Lazy;
use lazy::Lazy::*;
use std::sync::{Arc, Mutex};
use std::collections::BTreeMap;
use rustc_serialize::json::{Json, ToJson};
use error::BError;
use self::info::ApplicationInfo;

pub struct Application{
	id: String,
	client: Client,
	data: Arc<Mutex<Data>>
}
struct Data{
	name: Lazy<String>,
	incoming_call_url: Lazy<Option<String>>,
	incoming_call_url_callback_timeout: Lazy<Option<u64>>,
	incoming_call_fallback_url: Lazy<Option<String>>,
	incoming_message_url: Lazy<Option<String>>,
	incoming_message_url_callback_timeout: Lazy<Option<u64>>,
	incoming_message_fallback_url: Lazy<Option<String>>,
	callback_http_method: Lazy<Option<String>>,
	auto_answer: Lazy<Option<bool>>
}

#[derive(Clone)]
pub struct Config{
	pub name: String,
	pub incoming_call_url: Option<String>,
	pub incoming_call_url_callback_timeout: Option<u64>,
	pub incoming_call_fallback_url:  Option<String>,
	pub incoming_message_url: Option<String>,
	pub incoming_message_url_callback_timeout: Option<u64>,
	pub incoming_message_fallback_url: Option<String>,
	pub callback_http_method: Option<String>,
	pub auto_answer: Option<bool>
}

impl Config{
	pub fn new(name: &str) -> Config{
		Config{
			name: name.to_string(),
			incoming_call_url: None,
			incoming_call_url_callback_timeout: None,
			incoming_call_fallback_url: None,
			incoming_message_url: None,
			incoming_message_url_callback_timeout: None,
			incoming_message_fallback_url: None,
			callback_http_method: None,
			auto_answer: None
		}
	}
}

mod info{
	#![allow(non_snake_case)]
	#[derive(RustcEncodable, RustcDecodable, Clone)]
	pub struct ApplicationInfo{
		pub name: String,
		pub incomingCallUrl: Option<String>,
		pub incomingCallUrlCallbackTimeout: Option<u64>,
		pub incomingCallFallbackUrl:  Option<String>,
		pub incomingMessageUrl: Option<String>,
		pub incomingMessageUrlCallbackTimeout: Option<u64>,
		pub incomingMessageFallbackUrl: Option<String>,
		pub callbackHttpMethod: Option<String>,
		pub autoAnswer: Option<bool>
	}
}


impl Application{  
	pub fn load(&self) -> BResult<()>{
		
		//if id = empty string, this will return all apps
		if self.get_id().len() == 0{
			return Err(BError::bad_input("invalid app id"))
		}
		
		let path = "users/".to_string() + &self.client.get_user_id() + "/applications/" + &self.id;
		let res:JsonResponse<ApplicationInfo> = try!(self.client.raw_get_request(&path, (), ()));
		let mut data = self.data.lock().unwrap();
		data.name = Available(res.body.name);
		data.incoming_call_url = Available(res.body.incomingCallUrl);
		data.incoming_call_url_callback_timeout = Available(res.body.incomingCallUrlCallbackTimeout);
		data.incoming_call_fallback_url = Available(res.body.incomingCallFallbackUrl);
		data.incoming_message_url = Available(res.body.incomingMessageUrl);
		data.incoming_message_url_callback_timeout = Available(res.body.incomingMessageUrlCallbackTimeout);
		data.incoming_message_fallback_url = Available(res.body.incomingMessageFallbackUrl);
		data.callback_http_method = Available(res.body.callbackHttpMethod);
		data.auto_answer = Available(res.body.autoAnswer);
		Ok(())
	}
	pub fn save(&self) -> BResult<()>{
		let path = "users/".to_string() + &self.client.get_user_id() + "/applications/" + &self.id;
		let data = self.data.lock().unwrap();
		let mut map = BTreeMap::new();
		if let Some(name) = data.name.peek(){
			map.insert("name".to_string(), name.to_json());
		}
		if let Some(incoming_call_url) = data.incoming_call_url.peek(){
			map.insert("incomingCallUrl".to_string(), incoming_call_url.to_json());
		}
		if let Some(incoming_call_url_callback_timeout) = data.incoming_call_url_callback_timeout.peek(){
			map.insert("incomingCallUrlCallbackTimeout".to_string(), incoming_call_url_callback_timeout.to_json());
		}
		if let Some(incoming_call_fallback_url) = data.incoming_call_fallback_url.peek(){
			map.insert("incomingCallFallbackUrl".to_string(), incoming_call_fallback_url.to_json());
		}
		if let Some(incoming_message_url) = data.incoming_message_url.peek(){
			map.insert("incomingMessageUrl".to_string(), incoming_message_url.to_json());
		}
		if let Some(incoming_message_url_callback_timeout) = data.incoming_message_url_callback_timeout.peek(){
			map.insert("incomingMessageUrlCallbackTimeout".to_string(), incoming_message_url_callback_timeout.to_json());
		}
		if let Some(incoming_message_fallback_url) = data.incoming_message_fallback_url.peek(){
			map.insert("incomingMessageFallbackUrl".to_string(), incoming_message_fallback_url.to_json());
		}
		if let Some(callback_http_method) = data.callback_http_method.peek(){
			map.insert("callbackHttpMethod".to_string(), callback_http_method.to_json());
		}
		if let Some(auto_answer) = data.auto_answer.peek(){
			map.insert("autoAnswer".to_string(), auto_answer.to_json());
		}
		let json = Json::Object(map);
		let _:EmptyResponse = try!(self.client.raw_post_request(&path, (), json));
		Ok(())
	}
	pub fn create(client: &Client, config: &Config) -> BResult<Application>{
		let path = "users/".to_string() + &client.get_user_id() + "/applications";
		let json = json!({
			"name" => (config.name),
			"incomingCallUrl" => (config.incoming_call_url),
			"incomingCallUrlCallbackTimeout" => (config.incoming_call_url_callback_timeout),
			"incomingCallFallbackUrl" =>  (config.incoming_call_fallback_url),
			"incomingMessageUrl" => (config.incoming_message_url),
			"incomingMessageUrlCallbackTimeout" => (config.incoming_message_url_callback_timeout),
			"incomingMessageFallbackUrl" => (config.incoming_message_fallback_url),
			"callbackHttpMethod" => (config.callback_http_method),
			"autoAnswer" => (config.auto_answer)
		});
		let res:EmptyResponse = try!(client.raw_post_request(&path, (), json));
		let id = try!(util::get_id_from_location_header(&res.headers));
		Ok(Application{
			id: id,
			client: client.clone(),
			data: Arc::new(Mutex::new(Data{
				name: Available(config.name.clone()),
				incoming_call_url: Available(config.incoming_call_url.clone()),
				incoming_call_url_callback_timeout: Available(config.incoming_call_url_callback_timeout),
				incoming_call_fallback_url: Available(config.incoming_call_fallback_url.clone()),
				incoming_message_url: Available(config.incoming_message_url.clone()),
				incoming_message_url_callback_timeout: Available(config.incoming_message_url_callback_timeout),
				incoming_message_fallback_url: Available(config.incoming_message_fallback_url.clone()),
				callback_http_method: Available(config.callback_http_method.clone()),
				auto_answer: Available(config.auto_answer)
			}))
		})
	}
	pub fn get_by_id(client: &Client, id: &str) -> Application{
		Application{
			id: id.to_string(),
			client: client.clone(),
			data: Arc::new(Mutex::new(Data{
				name: NotLoaded,
				incoming_call_url: NotLoaded,
				incoming_call_url_callback_timeout: NotLoaded,
				incoming_call_fallback_url: NotLoaded,
				incoming_message_url: NotLoaded,
				incoming_message_url_callback_timeout: NotLoaded,
				incoming_message_fallback_url: NotLoaded,
				callback_http_method: NotLoaded,
				auto_answer: NotLoaded
			}))
		}
	}
	
	/* Getters */
	pub fn get_id(&self) -> String{
		self.id.clone()
	}
	pub fn get_client(&self) -> Client{
		self.client.clone()
	}
	pub fn get_name(&self) -> BResult<String>{
		if !self.data.lock().unwrap().name.available(){
			try!(self.load());
		}
		Ok(try!(self.data.lock().unwrap().name.get()).clone())
	}
	pub fn get_incoming_call_url(&self) -> BResult<Option<String>>{
		if !self.data.lock().unwrap().incoming_call_url.available(){
			try!(self.load());
		}
		Ok(try!(self.data.lock().unwrap().incoming_call_url.get()).clone())
	}
	pub fn get_incoming_call_url_callback_timeout(&self) -> BResult<Option<u64>>{
		if !self.data.lock().unwrap().incoming_call_url_callback_timeout.available(){
			try!(self.load());
		}
		Ok(try!(self.data.lock().unwrap().incoming_call_url_callback_timeout.get()).clone())
	}
	pub fn get_incoming_call_fallback_url(&self) -> BResult<Option<String>>{
		if !self.data.lock().unwrap().incoming_call_fallback_url.available(){
			try!(self.load());
		}
		Ok(try!(self.data.lock().unwrap().incoming_call_fallback_url.get()).clone())
	}
	pub fn get_incoming_message_url(&self) -> BResult<Option<String>>{
		if !self.data.lock().unwrap().incoming_message_url.available(){
			try!(self.load());
		}
		Ok(try!(self.data.lock().unwrap().incoming_message_url.get()).clone())
	}
	pub fn get_incoming_message_url_callback_timeout(&self) -> BResult<Option<u64>>{
		if !self.data.lock().unwrap().incoming_message_url_callback_timeout.available(){
			try!(self.load());
		}
		Ok(try!(self.data.lock().unwrap().incoming_message_url_callback_timeout.get()).clone())
	}
	pub fn get_callback_http_method(&self) -> BResult<Option<String>>{
		if !self.data.lock().unwrap().callback_http_method.available(){
			try!(self.load());
		}
		Ok(try!(self.data.lock().unwrap().callback_http_method.get()).clone())
	}
	pub fn get_auto_answer(&self) -> BResult<Option<bool>>{
		if !self.data.lock().unwrap().auto_answer.available(){
			try!(self.load());
		}
		Ok(try!(self.data.lock().unwrap().auto_answer.get()).clone())
	}
	
	/* Setters */
	pub fn set_name(&self, value: &str){
		self.data.lock().unwrap().name = Available(value.to_string());
	}
	pub fn set_incoming_call_url(&self, value: Option<&str>){
		self.data.lock().unwrap().incoming_call_url = Available(value.map(|a|a.to_string()));
	}
	pub fn set_incoming_call_url_callback_timeout(&self, value: Option<u64>){
		self.data.lock().unwrap().incoming_call_url_callback_timeout = Available(value);
	}
	pub fn set_incoming_call_fallback_url(&self, value: Option<&str>){
		self.data.lock().unwrap().incoming_call_fallback_url = Available(value.map(|a|a.to_string()));
	}
	pub fn set_incoming_message_url(&self, value: Option<&str>){
		self.data.lock().unwrap().incoming_message_url = Available(value.map(|a|a.to_string()));
	}
	pub fn set_incoming_message_url_callback_timeout(&self, value: Option<u64>){
		self.data.lock().unwrap().incoming_message_url_callback_timeout = Available(value);
	}
	pub fn set_incoming_message_fallback_url(&self, value: Option<&str>){
		self.data.lock().unwrap().incoming_message_fallback_url = Available(value.map(|a|a.to_string()));
	}
	pub fn set_callback_http_method(&self, value: Option<&str>){
		self.data.lock().unwrap().callback_http_method = Available(value.map(|a|a.to_string()));
	}
	pub fn set_auto_answer(&self, value: Option<bool>){
		self.data.lock().unwrap().auto_answer = Available(value);
	}
}