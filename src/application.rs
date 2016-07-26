use client::{EmptyResponse, JsonResponse, Client};
use CatapultResult;
use util;
use lazy::Lazy;
use lazy::Lazy::*;
use std::sync::{Arc, Mutex};
use std::collections::BTreeMap;
use rustc_serialize::json::{Json, ToJson};
use error::CatapultError;
use self::info::ApplicationInfo;
use rustc_serialize::json;


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
impl Data{
	fn from_info(client: &Client, info: ApplicationInfo) -> CatapultResult<Data>{
		Ok(Data{
			name: Available(info.name),
			incoming_call_url: Available(info.incomingCallUrl),
			incoming_call_url_callback_timeout: Available(info.incomingCallUrlCallbackTimeout),
			incoming_call_fallback_url: Available(info.incomingCallFallbackUrl),
			incoming_message_url: Available(info.incomingMessageUrl),
			incoming_message_url_callback_timeout: Available(info.incomingMessageUrlCallbackTimeout),
			incoming_message_fallback_url: Available(info.incomingMessageFallbackUrl),
			callback_http_method: Available(info.callbackHttpMethod),
			auto_answer: Available(info.autoAnswer)
		})
	}
}

mod info{
	#![allow(non_snake_case)]
	#[derive(RustcEncodable, RustcDecodable, Clone)]
	pub struct ApplicationInfo{
		pub id: String,
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
pub struct ApplicationBuilder{
	client: Client,
	name: String,
	incoming_message_url: String,
	incoming_call_url: String,
	incoming_call_url_callback_timeout: Option<u64>,
	incoming_call_fallback_url:  Option<String>,
	incoming_message_url_callback_timeout: Option<u64>,
	incoming_message_fallback_url: Option<String>,
	callback_http_method: String,
	auto_answer: bool
}
impl ApplicationBuilder{
	pub fn incoming_call_url_callback_timeout(mut self, millis: u64) -> Self{
		self.incoming_call_url_callback_timeout = Some(millis); self
	}
	pub fn incoming_call_fallback_url(mut self, url: &str) -> Self{
		self.incoming_call_fallback_url = Some(url.to_owned()); self
	}
	pub fn incoming_message_url_callback_timeout(mut self, millis: u64) -> Self{
		self.incoming_message_url_callback_timeout = Some(millis); self
	}
	pub fn incoming_message_fallback_url(mut self, url: &str) -> Self{
		self.incoming_message_fallback_url = Some(url.to_owned()); self
	}
	/// HTTP method defaults to POST and you receive data as a JSON body.
	/// Use this to switch to GET and receive data as uri query params
	pub fn use_get_http_method(mut self) -> Self{
		self.callback_http_method = "GET".to_owned(); self
	}
	pub fn disable_auto_answer(mut self) -> Self{
		self.auto_answer = false; self
	}
	pub fn create(self) -> CatapultResult<Application>{
		let path = "users/".to_string() + &self.client.get_user_id() + "/applications";
		let json = json!({
			"name" => (self.name),
			"incomingCallUrl" => (self.incoming_call_url),
			"incomingCallUrlCallbackTimeout" => (self.incoming_call_url_callback_timeout),
			"incomingCallFallbackUrl" =>  (self.incoming_call_fallback_url),
			"incomingMessageUrl" => (self.incoming_message_url),
			"incomingMessageUrlCallbackTimeout" => (self.incoming_message_url_callback_timeout),
			"incomingMessageFallbackUrl" => (self.incoming_message_fallback_url),
			"callbackHttpMethod" => (self.callback_http_method),
			"autoAnswer" => (self.auto_answer)
		});
		let res:EmptyResponse = try!(self.client.raw_post_request(&path, (), &json));
		let id = try!(util::get_id_from_location_header(&res.headers));
		Ok(Application{
			id: id,
			client: self.client.clone(),
			data: Arc::new(Mutex::new(Data{
				name: Available(self.name.clone()),
				incoming_call_url: Available(Some(self.incoming_call_url.clone())),
				incoming_call_url_callback_timeout: Available(self.incoming_call_url_callback_timeout),
				incoming_call_fallback_url: Available(self.incoming_call_fallback_url.clone()),
				incoming_message_url: Available(Some(self.incoming_message_url.clone())),
				incoming_message_url_callback_timeout: Available(self.incoming_message_url_callback_timeout),
				incoming_message_fallback_url: Available(self.incoming_message_fallback_url.clone()),
				callback_http_method: Available(Some(self.callback_http_method.clone())),
				auto_answer: Available(Some(self.auto_answer))
			}))
		})
	}
}

pub struct QueryResult{
	client: Client,
	data: Vec<Application>,
	next_url: Option<String>
}
impl QueryResult{
	pub fn get_applications(&self) -> &Vec<Application>{
		&self.data
	}
	pub fn has_next(&self) -> bool{
		self.next_url.is_some()
	}
	pub fn next(&self) -> Option<CatapultResult<QueryResult>>{
		self.next_url.as_ref().map(|ref url|{
			Application::list(&self.client, &url, ())
		})
	}
}

pub struct Query{
	client: Client,
	size: Option<u32>
}
impl Query{
	pub fn size(mut self, size: u32) -> Query{
		self.size = Some(size); self
	}
	pub fn submit(&self) -> CatapultResult<QueryResult>{
		let path = "users/".to_string() + &self.client.get_user_id() + "/applications";
	
		let mut map = BTreeMap::new();
		if let Some(size) = self.size{
			map.insert("size".to_owned(), size.to_json());
		}
		let json = Json::Object(map);
		Application::list(&self.client, &path, json)
	}
}



pub struct Application{
	id: String,
	client: Client,
	data: Arc<Mutex<Data>>
}
impl Application{ 
	//Constructors
	pub fn build(client: &Client, name: &str, call_url: &str, msg_url: &str) -> ApplicationBuilder{
		ApplicationBuilder{
			client: client.clone(),
			name: name.to_owned(),
			incoming_message_url: msg_url.to_owned(),
			incoming_call_url: call_url.to_owned(),
			incoming_call_url_callback_timeout: None,
			incoming_call_fallback_url: None,
			incoming_message_url_callback_timeout: None,
			incoming_message_fallback_url: None,
			callback_http_method: "POST".to_owned(),
			auto_answer: true
		}
	}
	fn list<P: json::ToJson>(client: &Client, path: &str, params: P) -> CatapultResult<QueryResult>{
		let res:JsonResponse<Vec<ApplicationInfo>> = try!(client.raw_get_request(&path, params, ()));
		let mut output = vec!();
		for info in res.body{
			output.push(Application{
				id: info.id.clone(),
				client: client.clone(),
				data: Arc::new(Mutex::new(try!(Data::from_info(&client, info))))
			});
		}
		let next_url = try!(util::get_next_link_from_headers(&res.headers));
		Ok(QueryResult{
			client: client.clone(),
			data: output,
			next_url: next_url
		})
	}
	pub fn query(client: &Client) -> Query{
		Query{
			client: client.clone(),
			size: None
		}
	}
	pub fn delete(&self) -> CatapultResult<()>{
		let path = "users/".to_string() + &self.client.get_user_id() + "/applications/" + &self.id;
		let _:EmptyResponse = try!(self.client.raw_delete_request(&path, ()));
		Ok(())
	}
	pub fn get(client: &Client, id: &str) -> Application{
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
	
	pub fn load(&self) -> CatapultResult<()>{
		//if id = empty string, this will return all apps
		if self.get_id().len() == 0{
			return Err(CatapultError::bad_input("invalid app id"))
		}
		let path = "users/".to_string() + &self.client.get_user_id() + "/applications/" + &self.id;
		let res:JsonResponse<ApplicationInfo> = try!(self.client.raw_get_request(&path, (), ()));
		let mut data = self.data.lock().unwrap();
		*data = try!(Data::from_info(&self.client, res.body));
		Ok(())
	}
	pub fn save(&self) -> CatapultResult<()>{
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
		let _:EmptyResponse = try!(self.client.raw_post_request(&path, (), &json));
		Ok(())
	}
	
	/* Getters */
	pub fn get_id(&self) -> String{
		self.id.clone()
	}
	pub fn get_client(&self) -> Client{
		self.client.clone()
	}
	pub fn get_name(&self) -> CatapultResult<String>{
		lazy_load!(self, name)
	}
	pub fn get_incoming_call_url(&self) -> CatapultResult<Option<String>>{
		lazy_load!(self, incoming_call_url)
	}
	pub fn get_incoming_call_url_callback_timeout(&self) -> CatapultResult<Option<u64>>{
		lazy_load!(self, incoming_call_url_callback_timeout)
	}
	pub fn get_incoming_call_fallback_url(&self) -> CatapultResult<Option<String>>{
		lazy_load!(self, incoming_call_fallback_url)
	}
	pub fn get_incoming_message_url(&self) -> CatapultResult<Option<String>>{
		lazy_load!(self, incoming_message_url)
	}
	pub fn get_incoming_message_url_callback_timeout(&self) -> CatapultResult<Option<u64>>{
		lazy_load!(self, incoming_message_url_callback_timeout)
	}
	pub fn get_callback_http_method(&self) -> CatapultResult<Option<String>>{
		lazy_load!(self, callback_http_method)
	}
	pub fn get_auto_answer(&self) -> CatapultResult<Option<bool>>{
		lazy_load!(self, auto_answer)
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