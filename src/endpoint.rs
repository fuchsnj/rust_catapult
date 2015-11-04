use {Client, BResult, Domain};
use application::Application;
use auth_token::AuthToken;
use client::{JsonResponse, EmptyResponse};
use std::sync::{Mutex, Arc};
use lazy::Lazy;
use lazy::Lazy::*;
use util;
use std::collections::BTreeMap;
use rustc_serialize::json::{Json, ToJson};
use self::info::EndpointInfo;

pub struct Endpoint{
	id: String,
	domain_id: String,
	client: Client,
	data: Arc<Mutex<Data>>
}

struct Data{
	name: Lazy<String>,
	description: Lazy<Option<String>>,
	enabled: Lazy<bool>,
	application_id: Lazy<String>,
	realm: Lazy<String>,
	username: Lazy<String>,
	sip_uri: Lazy<String>,
	password: Lazy<String> //write only
}

pub struct Config{
	pub name: String,
	pub description: Option<String>,
	pub enabled: bool,
	pub password: String
}



mod info{
	#![allow(non_snake_case)]
	#[derive(RustcDecodable)]
	pub struct Credentials{
		pub realm: String,
		pub username: String
	}

	#[derive(RustcDecodable)]
	pub struct EndpointInfo{
		pub id: String,
		pub name: String,
		pub description: Option<String>,
		pub domainId: String,
		pub enabled: bool,
		pub applicationId: String,
		pub credentials: Credentials,
		pub sipUri: String
	}
}

impl Endpoint{
	pub fn load(&self) -> BResult<()>{
		let path = "users/".to_string() + &self.client.get_user_id() + "/domains/" + &self.domain_id + "/endpoints/" + &self.id;
		let res:JsonResponse<EndpointInfo> = try!(self.client.raw_get_request(&path, (), ()));
		let mut data = self.data.lock().unwrap();
		data.name = Available(res.body.name);
		data.description = Available(res.body.description);
		data.enabled = Available(res.body.enabled);
		data.application_id= Available(res.body.applicationId);
		data.realm = Available(res.body.credentials.realm);
		data.username = Available(res.body.credentials.username);
		data.sip_uri = Available(res.body.sipUri);
		Ok(())
	}
	pub fn save(&self) -> BResult<()>{
		let path = "users/".to_string() + &self.client.get_user_id() + "/domains/" + &self.domain_id + "/endpoints/" + &self.id;
		let data = self.data.lock().unwrap();
		let mut map = BTreeMap::new();
		if let Some(description) = data.description.peek(){
			map.insert("description".to_string(), description.to_json());
		}
		if let Some(enabled) = data.enabled.peek(){
			map.insert("enabled".to_string(), enabled.to_json());
		}
		if let Some(application_id) = data.application_id.peek(){
			map.insert("applicationId".to_string(), application_id.to_json());
		}
		if let Some(password) = data.password.peek(){
			map.insert("credentials".to_string(), json!({
				"password": (password)
			}));
		}
		let json = Json::Object(map);
		let _:EmptyResponse = try!(self.client.raw_post_request(&path, (), json));
		Ok(())
	}
	pub fn get_by_id(client: &Client, id: &str, domain_id: &str) -> Endpoint{
		Endpoint{
			id: id.to_string(),
			domain_id: domain_id.to_string(),
			client: client.clone(),
			data: Arc::new(Mutex::new(Data{
				name: NotLoaded,
				description: NotLoaded,
				enabled: NotLoaded,
				application_id: NotLoaded,
				realm: NotLoaded,
				username: NotLoaded,
				sip_uri: NotLoaded,
				password: NotLoaded
			}))
		}
	}
	pub fn create(client: &Client, domain_id: &str, app_id: &str, config: &Config) -> BResult<Endpoint>{
		let json = json!({
			"name": (config.name),
			"description": (config.description),
			"domainId": (domain_id),
			"applicationId": (app_id),
			"enabled": (config.enabled),
			"credentials": (json!({
				"password": (config.password)
			}))
		});
		let path = "users/".to_string() + &client.get_user_id() + "/domains/" + domain_id + "/endpoints";
		let res:EmptyResponse = try!(client.raw_post_request(&path, (), json));
		let id = try!(util::get_id_from_location_header(&res.headers));
		Ok(Endpoint{
			id: id,
			domain_id: domain_id.to_string(),
			client: client.clone(),
			data: Arc::new(Mutex::new(Data{
				name: Available(config.name.clone()),
				description: Available(config.description.clone()),
				enabled: Available(config.enabled),
				application_id: Available(app_id.to_string()),
				realm: NotLoaded,
				username: NotLoaded,
				sip_uri: NotLoaded,
				password: NotLoaded
			}))
		})
	}
	pub fn create_auth_token(&self) -> BResult<AuthToken>{
		AuthToken::create(self)
	}
	
	/* Getters */
	pub fn get_id(&self) -> String{
		self.id.clone()
	}
	pub fn get_domain(&self) -> Domain{
		Domain::get_by_id(&self.client, &self.domain_id)
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
	pub fn get_description(&self) -> BResult<Option<String>>{
		if !self.data.lock().unwrap().description.available(){
			try!(self.load());
		}
		Ok(try!(self.data.lock().unwrap().description.get()).clone())
	}
	pub fn get_enabled(&self) -> BResult<bool>{
		if !self.data.lock().unwrap().enabled.available(){
			try!(self.load());
		}
		Ok(try!(self.data.lock().unwrap().enabled.get()).clone())
	}
	pub fn get_application_id(&self) -> BResult<String>{
		if !self.data.lock().unwrap().application_id.available(){
			try!(self.load());
		}
		Ok(try!(self.data.lock().unwrap().application_id.get()).clone())
	}
	pub fn get_realm(&self) -> BResult<String>{
		if !self.data.lock().unwrap().realm.available(){
			try!(self.load());
		}
		Ok(try!(self.data.lock().unwrap().realm.get()).clone())
	}
	pub fn get_username(&self) -> BResult<String>{
		if !self.data.lock().unwrap().username.available(){
			try!(self.load());
		}
		Ok(try!(self.data.lock().unwrap().username.get()).clone())
	}
	pub fn get_sip_uri(&self) -> BResult<String>{
		if !self.data.lock().unwrap().sip_uri.available(){
			try!(self.load());
		}
		Ok(try!(self.data.lock().unwrap().sip_uri.get()).clone())
	}
	
	pub fn get_application(&self) -> BResult<Application>{
		let app_id = try!(self.get_application_id());
		Ok(Application::get_by_id(&self.client, &app_id))
	}
	
	/* Setters */
	pub fn set_description(&self, value: Option<&str>){
		self.data.lock().unwrap().description = Available(value.map(|a|a.to_string()));
	}
	pub fn set_enabled(&self, value: bool){
		self.data.lock().unwrap().enabled = Available(value);
	}
	pub fn set_application_id(&self, id: &str){
		self.data.lock().unwrap().application_id = Available(id.to_string());
	}
	pub fn set_password(&self, value: &str){
		self.data.lock().unwrap().password = Available(value.to_string());
	}
}