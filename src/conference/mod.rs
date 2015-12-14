mod member;

pub use self::member::{Member, MemberBuilder};

use {BResult, BError};
use client::{EmptyResponse, JsonResponse, Client};
use std::sync::{Arc, Mutex};
use util;
use lazy::Lazy;
use lazy::Lazy::*;
use self::info::ConferenceInfo;
use rustc_serialize::json::Json;
use rustc_serialize::json::ToJson;
use std::collections::BTreeMap;
use voice::Voice;

#[derive(Clone, Debug)]
pub enum State{
	///Conference was created and has no members
	Created,
	///Conference was created and has one or more active members.
	///As soon as the first member is added to a conference the state is changed to active.
	Active,
	///Once the conference is completed, it can no longer be used.
	Completed
}

struct Data{
	active_members: Lazy<u64>,
	created_time: Lazy<String>,
	from: Lazy<String>,
	state: Lazy<State>,
	callback_http_method: Lazy<String>,
	hold: Lazy<bool>,
	mute: Lazy<bool>,
	callback_url: Lazy<Option<String>>,
	fallback_url: Lazy<Option<String>>,
	callback_timeout: Lazy<u64>,
	tag: Lazy<Option<String>>
}
impl Data{
	fn from_info(info: &ConferenceInfo) -> BResult<Data>{
		Ok(Data{
			active_members: Available(info.activeMembers),
			created_time: Available(info.createdTime.to_owned()),
			from: Available(info.from.to_owned()),
			state: Available(match info.state.as_ref(){
				"created" => State::Created,
				"active" => State::Active,
				"completed" => State::Completed,
				state @ _ => return Err(BError::unexpected(
					&format!("unknown Conference state: {}", state)
				))
			}),
			callback_http_method: Available(info.callbackHttpMethod.to_owned()),
			hold: Available(info.hold),
			mute: Available(info.mute),
			callback_url: Available(info.callbackUrl.to_owned()),
			fallback_url: Available(info.fallbackUrl.to_owned()),
			callback_timeout: Available(info.callbackTimeout),
			tag: Available(info.tag.to_owned())
		})
	}
}

mod info{
	#![allow(non_snake_case)]
	#[derive(RustcDecodable)]
	pub struct ConferenceInfo{
		pub id: String,
		pub activeMembers: u64,
		pub callbackUrl: Option<String>,
		pub fallbackUrl: Option<String>,
		pub callbackHttpMethod: String,
		pub callbackTimeout: u64,
		pub createdTime: String,
		pub from: String,
		pub hold: bool,
		pub mute: bool,
		pub state: String,
		pub tag: Option<String>
	}
}

pub struct ConferenceBuilder{
	client: Client,
	from: String,
	callback_url: Option<String>,
	callback_http_method: String,
	callback_timeout: Option<u64>,
	fallback_url: Option<String>,
	tag: Option<String>
}
impl ConferenceBuilder{
	pub fn callback_url(mut self, url: &str) -> Self{
		self.callback_url = Some(url.to_owned()); self
	}
	pub fn callback_timeout(mut self, millis: u64) -> Self{
		self.callback_timeout = Some(millis); self
	}
	pub fn use_get_http_method(mut self) -> Self{
		self.callback_http_method = "GET".to_owned(); self
	}
	pub fn fallback_url(mut self, url: &str) -> Self{
		self.fallback_url = Some(url.to_owned()); self
	}
	pub fn tag(mut self, tag: &str) -> Self{
		self.tag = Some(tag.to_owned()); self
	}
	pub fn create(self) -> BResult<Conference>{
		let path = "users/".to_string() + &self.client.get_user_id() + "/conferences";
		let json = json!({
			"from" => (self.from),
			"callbackUrl" => (self.callback_url),
			"callbackHttpMethod" => (self.callback_http_method),
			"callbackTimeout" => (self.callback_timeout),
			"fallbackUrl" => (self.fallback_url),
			"tag" => (self.tag)
		});
		let res:EmptyResponse = try!(self.client.raw_post_request(&path, (), &json));
		let id = try!(util::get_id_from_location_header(&res.headers));
		Ok(Conference{
			id: id,
			client: self.client,
			data: Arc::new(Mutex::new(Data{
				active_members: Available(0),
				created_time: NotLoaded,
				from: Available(self.from.to_owned()),
				state: Available(State::Created),
				callback_http_method: Available(self.callback_http_method.to_owned()),
				hold: NotLoaded,
				mute: NotLoaded,
				callback_url: Available(self.callback_url.to_owned()),
				fallback_url: Available(self.fallback_url.to_owned()),
				callback_timeout: Lazy::load_if_available(self.callback_timeout),
				tag: Available(self.tag.to_owned())
			}))
		})
	}
}

#[derive(Clone)]
pub struct Conference{
	id: String,
	client: Client,
	data: Arc<Mutex<Data>> 
}

impl Conference{
	pub fn load(&self) -> BResult<()>{
		let path = "users/".to_string() + &self.client.get_user_id() + "/conferences/" + &self.id;
		let res:JsonResponse<ConferenceInfo> = try!(self.client.raw_get_request(&path, (), ()));
		let mut data = self.data.lock().unwrap();
		*data = try!(Data::from_info(&res.body));
		Ok(())
	}
	pub fn save(&self) -> BResult<()>{
		let user_id = self.client.get_user_id();
		let path = "users/".to_string() + &user_id + "/conferences/" + &self.get_id();
		let mut map = BTreeMap::new();
		{
			let data = self.data.lock().unwrap();
			if let Some(value) = data.hold.peek(){
				map.insert("hold".to_string(), value.to_json());
			}
			if let Some(value) = data.mute.peek(){
				map.insert("mute".to_string(), value.to_json());
			}
			if let Some(value) = data.callback_url.peek(){
				map.insert("callbackUrl".to_string(), value.to_json());
			}
			if let Some(value) = data.callback_http_method.peek(){
				map.insert("callbackHttpMethod".to_string(), value.to_json());
			}
			if let Some(value) = data.callback_timeout.peek(){
				map.insert("callbackTimeout".to_string(), value.to_json());
			}
			if let Some(value) = data.fallback_url.peek(){
				map.insert("fallbackUrl".to_string(), value.to_json());
			}
			if let Some(value) = data.tag.peek(){
				map.insert("tag".to_string(), value.to_json());
			}
		}
		let json = Json::Object(map);
		let _:EmptyResponse = try!(self.client.raw_post_request(&path, (), &json));
		Ok(())
	}
	pub fn get(client: &Client, id: &str) -> Conference{
		Conference{
			id: id.to_owned(),
			client: client.clone(),
			data: Arc::new(Mutex::new(Data{
				active_members: NotLoaded,
				created_time: NotLoaded,
				from: NotLoaded,
				state: NotLoaded,
				callback_http_method: NotLoaded,
				hold: NotLoaded,
				mute: NotLoaded,
				callback_url: NotLoaded,
				fallback_url: NotLoaded,
				callback_timeout: NotLoaded,
				tag: NotLoaded
			}))
		}
	}
	pub fn build(client: &Client, from: &str) -> ConferenceBuilder{
		ConferenceBuilder{
			client: client.clone(),
			from: from.to_owned(),
			callback_url: None,
			callback_http_method: "POST".to_owned(),
			callback_timeout: None,
			fallback_url: None,
			tag: None
		}
	}
	
	
	/* Getters */
	pub fn get_id(&self) -> String{
		self.id.clone()
	}
	pub fn get_client(&self) -> Client{
		self.client.clone()
	}
	pub fn list_members(&self) -> BResult<Vec<Member>>{
		Member::list_members_from_conference(self)
	}
	
	pub fn get_active_members(&self) -> BResult<u64>{
		if !self.data.lock().unwrap().active_members.available(){
			try!(self.load());
		}
		Ok(try!(self.data.lock().unwrap().active_members.get()).clone())
	}
	pub fn get_created_time(&self) -> BResult<String>{
		if !self.data.lock().unwrap().created_time.available(){
			try!(self.load());
		}
		Ok(try!(self.data.lock().unwrap().created_time.get()).clone())
	}
	pub fn get_from(&self) -> BResult<String>{
		if !self.data.lock().unwrap().from.available(){
			try!(self.load());
		}
		Ok(try!(self.data.lock().unwrap().from.get()).clone())
	}
	pub fn get_state(&self) -> BResult<State>{
		if !self.data.lock().unwrap().state.available(){
			try!(self.load());
		}
		Ok(try!(self.data.lock().unwrap().state.get()).clone())
	}
	pub fn get_callback_http_method(&self) -> BResult<String>{
		if !self.data.lock().unwrap().callback_http_method.available(){
			try!(self.load());
		}
		Ok(try!(self.data.lock().unwrap().callback_http_method.get()).clone())
	}
	pub fn get_hold(&self) -> BResult<bool>{
		if !self.data.lock().unwrap().hold.available(){
			try!(self.load());
		}
		Ok(try!(self.data.lock().unwrap().hold.get()).clone())
	}
	pub fn get_mute(&self) -> BResult<bool>{
		if !self.data.lock().unwrap().mute.available(){
			try!(self.load());
		}
		Ok(try!(self.data.lock().unwrap().mute.get()).clone())
	}
	pub fn get_callback_url(&self) -> BResult<Option<String>>{
		if !self.data.lock().unwrap().callback_url.available(){
			try!(self.load());
		}
		Ok(try!(self.data.lock().unwrap().callback_url.get()).clone())
	}
	pub fn get_fallback_url(&self) -> BResult<Option<String>>{
		if !self.data.lock().unwrap().fallback_url.available(){
			try!(self.load());
		}
		Ok(try!(self.data.lock().unwrap().fallback_url.get()).clone())
	}
	pub fn get_callback_timeout(&self) -> BResult<u64>{
		if !self.data.lock().unwrap().callback_timeout.available(){
			try!(self.load());
		}
		Ok(try!(self.data.lock().unwrap().callback_timeout.get()).clone())
	}
	pub fn get_tag(&self) -> BResult<Option<String>>{
		if !self.data.lock().unwrap().tag.available(){
			try!(self.load());
		}
		Ok(try!(self.data.lock().unwrap().tag.get()).clone())
	}
	
	
	/* Actions */
	
	///add a member to a conference. The call_id MUST be in the active state.
	///mute/hold default to false, join/leaving tone default to true
	pub fn build_member(&self, call_id: &str) -> MemberBuilder{
		MemberBuilder::new(self, call_id)
	}
	pub fn end(&self) -> BResult<()>{
		let user_id = self.client.get_user_id();
		let path = "users/".to_string() + &user_id + "/conferences/" + &self.get_id();
		let json = json!({
			"state" => "completed"
		});
		let _:EmptyResponse = try!(self.client.raw_post_request(&path, (), &json));
		Ok(())
	}
	
	pub fn speak_sentence(&self, sentence: &str, loop_audio: bool, voice: Voice, tag: Option<&str>) -> BResult<()>{
		let user_id = self.client.get_user_id();
		let path = "users/".to_string() + &user_id + "/conferences/" + &self.get_id() + "/audio";
		let json = json!({
			"sentence" => (sentence),
			"loopEnabled" => (loop_audio),
			"voice" => (voice.get_name()),
			"tag" => (tag.map(|a|a.to_string()))
		});
		let _:EmptyResponse = try!(self.client.raw_post_request(&path, (), &json));
		Ok(())
	}
	pub fn play_audio_file(&self, url: &str, loop_audio: bool, tag: Option<&str>) -> BResult<()>{
		let user_id = self.client.get_user_id();
		let path = "users/".to_string() + &user_id + "/conferences/" + &self.get_id() + "/audio";
		let json = json!({
			"fileUrl" => (url),
			"loopEnabled" => (loop_audio),
			"tag" => (tag.map(|a|a.to_string()))
		});
		let _:EmptyResponse = try!(self.client.raw_post_request(&path, (), &json));
		Ok(())
	}
	
	/* Setters */
	pub fn set_mute(&self, value: bool){
		self.data.lock().unwrap().mute = Available(value);
	}
	pub fn set_hold(&self, value: bool) {
		self.data.lock().unwrap().hold = Available(value);
	}
	pub fn set_callback_method_get(&self) {
		self.data.lock().unwrap().callback_http_method = Available("GET".to_owned());
	}
	pub fn set_callback_method_post(&self) {
		self.data.lock().unwrap().callback_http_method = Available("POST".to_owned());
	}
	pub fn set_callback_timeout(&self, millis: u64) {
		self.data.lock().unwrap().callback_timeout = Available(millis);
	}
	pub fn set_callback_url(&self, value: Option<&str>) {
		self.data.lock().unwrap().callback_url = Available(value.map(|a|a.to_owned()));
	}
	pub fn set_fallback_url(&self, value: Option<&str>) {
		self.data.lock().unwrap().fallback_url = Available(value.map(|a|a.to_owned()));
	}
	pub fn set_tag(&self, value: Option<&str>) {
		self.data.lock().unwrap().tag = Available(value.map(|a|a.to_owned()));
	}
}

