use {CatapultResult, CatapultError};
use client::{EmptyResponse, JsonResponse, Client};
use std::sync::{Arc, Mutex};
use util;
use lazy::Lazy;
use lazy::Lazy::*;
use rustc_serialize::json::Json;
use rustc_serialize::json::ToJson;
use conference::Conference;
use std::collections::BTreeMap;
use self::info::MemberInfo;
use voice::Voice;

#[derive(Clone)]
pub enum State{
	Active,
	Completed
}
pub struct Member{
	id: String,
	conf: Conference,
	data: Arc<Mutex<Data>>
}
impl Member{
	fn post(&self, json: Json) -> CatapultResult<()>{
		let path = "users/".to_string() + &self.conf.get_client().get_user_id() + "/conferences/"
			+ &self.conf.get_id() + "/members/" + &self.id;
		let _:EmptyResponse = try!(self.conf.get_client().raw_post_request(&path, (), &json));
		Ok(())
	}
	pub fn save(&self) -> CatapultResult<()>{
		let mut map = BTreeMap::new();
		{
			let data = self.data.lock().unwrap();
			if let Some(value) = data.join_tone.peek(){
				map.insert("joinTone".to_string(), value.to_json());
			}
			if let Some(value) = data.leaving_tone.peek(){
				map.insert("leavingTone".to_string(), value.to_json());
			}
			if let Some(value) = data.mute.peek(){
				map.insert("mute".to_string(), value.to_json());
			}
			if let Some(value) = data.hold.peek(){
				map.insert("hold".to_string(), value.to_json());
			}
		}
		self.post(Json::Object(map))
	}
	pub fn load(&self) -> CatapultResult<()>{
		//if id = empty string, this will return all members
		if self.get_id().len() == 0{
			return Err(CatapultError::bad_input("invalid member id"))
		}
		let path = "users/".to_string() + &self.conf.get_client().get_user_id() + "/conferences/"
			+ &self.conf.get_id() + "/members/" + &self.id;
		let res:JsonResponse<MemberInfo> = try!(self.conf.get_client().raw_get_request(&path, (), ()));
		let mut data = self.data.lock().unwrap();
		*data = try!(Data::from_info(&res.body));
		Ok(())
	}
	
	/* Actions */
	pub fn remove(&self) -> CatapultResult<()>{
		try!(self.post(json!({
			"state" => "completed"
		})));
		self.data.lock().unwrap().state = Available(State::Completed);
		Ok(())
	}
	pub fn speak_sentence(&self, sentence: &str, loop_audio: bool, voice: Voice, tag: Option<&str>) -> CatapultResult<()>{
		let client = self.conf.get_client();
		let user_id = client.get_user_id();
		let conf_id = self.conf.get_id();
		let id = self.get_id();
		let path = "users/".to_string() + &user_id + "/conferences/" + &conf_id + "/members/" + &id + "/audio";
		let json = json!({
			"sentence" => (sentence),
			"loopEnabled" => (loop_audio),
			"voice" => (voice.get_name()),
			"tag" => (tag.map(|a|a.to_string()))
		});
		let _:EmptyResponse = try!(client.raw_post_request(&path, (), &json));
		Ok(())
	}
	pub fn play_audio_file(&self, url: &str, loop_audio: bool, tag: Option<&str>) -> CatapultResult<()>{
		let client = self.conf.get_client();
		let user_id = client.get_user_id();
		let conf_id = self.conf.get_id();
		let id = self.get_id();
		let path = "users/".to_string() + &user_id + "/conferences/" + &conf_id + "/members/" + &id + "/audio";
		let json = json!({
			"fileUrl" => (url),
			"loopEnabled" => (loop_audio),
			"tag" => (tag.map(|a|a.to_string()))
		});
		let _:EmptyResponse = try!(client.raw_post_request(&path, (), &json));
		Ok(())
	}
	///Stops either an audio file playing, or a sentence being spoken
	///This is the only way to stop audio in a loop
	pub fn stop_audio(&self) -> CatapultResult<()>{
		let client = self.conf.get_client();
		let user_id = client.get_user_id();
		let conf_id = self.conf.get_id();
		let id = self.get_id();
		let path = "users/".to_string() + &user_id + "/conferences/" + &conf_id + "/members/" + &id + "/audio";
		let json = json!({
			"fileUrl" => ""
		});
		let _:EmptyResponse = try!(client.raw_post_request(&path, (), &json));
		Ok(())
	}
	
	/* Setters */
	pub fn set_mute(&self, value: bool){
		self.data.lock().unwrap().mute = Available(value);
	}
	pub fn set_hold(&self, value: bool) {
		self.data.lock().unwrap().hold = Available(value);
	}
	pub fn join_tone(&self, value: bool){
		self.data.lock().unwrap().join_tone = Available(value);
	}
	pub fn leaving_tone(&self, value: bool){
		self.data.lock().unwrap().leaving_tone = Available(value);
	}
	
	/* Getters */
	pub fn get_id(&self) -> String{
		self.id.clone()
	}
	pub fn get_client(&self) -> Client{
		self.conf.get_client().clone()
	}
	pub fn get_added_time(&self) -> CatapultResult<String>{
		if !self.data.lock().unwrap().added_time.available(){
			try!(self.load());
		}
		Ok(try!(self.data.lock().unwrap().added_time.get()).clone())
	}
	pub fn get_removed_time(&self) -> CatapultResult<Option<String>>{
		if !self.data.lock().unwrap().removed_time.available(){
			try!(self.load());
		}
		Ok(try!(self.data.lock().unwrap().removed_time.get()).clone())
	}
	pub fn get_join_tone(&self) -> CatapultResult<bool>{
		if !self.data.lock().unwrap().join_tone.available(){
			try!(self.load());
		}
		Ok(try!(self.data.lock().unwrap().join_tone.get()).clone())
	}
	pub fn get_leaving_tone(&self) -> CatapultResult<bool>{
		if !self.data.lock().unwrap().leaving_tone.available(){
			try!(self.load());
		}
		Ok(try!(self.data.lock().unwrap().leaving_tone.get()).clone())
	}
	pub fn get_mute(&self) -> CatapultResult<bool>{
		if !self.data.lock().unwrap().mute.available(){
			try!(self.load());
		}
		Ok(try!(self.data.lock().unwrap().mute.get()).clone())
	}
	pub fn get_hold(&self) -> CatapultResult<bool>{
		if !self.data.lock().unwrap().hold.available(){
			try!(self.load());
		}
		Ok(try!(self.data.lock().unwrap().hold.get()).clone())
	}
	pub fn get_state(&self) -> CatapultResult<State>{
		if !self.data.lock().unwrap().state.available(){
			try!(self.load());
		}
		Ok(try!(self.data.lock().unwrap().state.get()).clone())
	}
	pub fn list_members_from_conference(conf: &Conference) -> CatapultResult<Vec<Member>>{
		let client = conf.get_client();
		let path = "users/".to_string() + &client.get_user_id() + "/conferences/"
			+ &conf.get_id() + "/members";
		let res:JsonResponse<Vec<MemberInfo>> = try!(client.raw_get_request(&path, (), ()));
		let mut output = vec!();
		for info in res.body{
			output.push(Member{
				id: info.id.clone(),
				conf: conf.clone(),
				data: Arc::new(Mutex::new(try!(Data::from_info(&info))))
			})
		}
		Ok(output)
	}
}

struct Data{
	added_time: Lazy<String>,
	removed_time: Lazy<Option<String>>,
	join_tone: Lazy<bool>,
	leaving_tone: Lazy<bool>,
	mute: Lazy<bool>,
	hold: Lazy<bool>,
	state: Lazy<State>
}
impl Data{
	fn from_info(info: &MemberInfo) -> CatapultResult<Data>{
		Ok(Data{
			added_time: Available(info.addedTime.to_owned()),
			removed_time: Available(info.removedTime.to_owned()),
			join_tone: Available(info.joinTone),
			leaving_tone: Available(info.leavingTone),
			mute: Available(info.mute),
			hold: Available(info.hold),
			state: Available(match info.state.as_ref(){
				"active" => State::Active,
				"completed" => State::Completed,
				state @ _ => return Err(CatapultError::unexpected(
					&format!("unknown member state: {}", state)
				))
			}),
		})
	}
}

mod info{
	#![allow(non_snake_case)]
	#[derive(RustcEncodable, RustcDecodable, Clone)]
	pub struct MemberInfo{
		pub addedTime: String,
		pub removedTime: Option<String>,
		pub hold: bool,
		pub id: String,
		pub mute: bool,
		pub state: String,
		pub joinTone: bool,
		pub leavingTone: bool
	}
}
pub struct MemberBuilder{
	conf: Conference,
	call_id: String,
	join_tone: bool,
	leaving_tone: bool,
	mute: bool,
	hold: bool
}
impl MemberBuilder{
	pub fn no_join_tone(mut self) -> Self{
		self.join_tone = false; self
	}
	pub fn no_leaving_tone(mut self) -> Self{
		self.leaving_tone = false; self
	}
	pub fn mute(mut self) -> Self{
		self.mute = true; self
	}
	pub fn hold(mut self) -> Self{
		self.hold = true; self
	}
	pub fn new(conf: &Conference, call_id: &str) -> MemberBuilder{
		MemberBuilder{
			conf: conf.clone(),
			call_id: call_id.to_owned(),
			join_tone: true,
			leaving_tone: true,
			mute: false,
			hold: false
		}
	}
	pub fn create(self) -> CatapultResult<Member>{
		let path = "users/".to_string() + &self.conf.get_client().get_user_id() + "/conferences/" + &self.conf.get_id() + "/members";
		let json = json!({
			"callId" => (self.call_id),
			"joinTone" => (self.join_tone),
			"leavingTone" => (self.leaving_tone),
			"mute" => (self.mute),
			"hold" => (self.hold)
		});
		println!("sending json: {:?}", json);
		let res:EmptyResponse = try!(self.conf.get_client().raw_post_request(&path, (), &json));
		let id = try!(util::get_id_from_location_header(&res.headers));
		println!("member id: {}", id);
		Ok(Member{
			id: id,
			conf: self.conf.clone(),
			data: Arc::new(Mutex::new(Data{
				join_tone: NotLoaded,//Catapult seems to have a bug with this
				leaving_tone: NotLoaded,//Catapult seems to have a bug with this
				mute: Available(self.mute),
				hold: Available(self.hold),
				state: NotLoaded,
				added_time: NotLoaded,
				removed_time: NotLoaded
			}))
		})
	}
}