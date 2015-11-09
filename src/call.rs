use {BResult, BError};
use client::{EmptyResponse, JsonResponse, Client};
use std::sync::{Arc, Mutex};
use call_event::CallEvent;
use bridge::Bridge;
use util;
use lazy::Lazy;
use lazy::Lazy::*;
use self::info::CallInfo;
use rustc_serialize::json::Json;
use rustc_serialize::json::ToJson;
use voice::Voice;



#[derive(Clone)]
pub enum State{
	Started,
	Rejected,
	Active,
	Completed,
	Transferring
}

struct Data{
	active_time: Lazy<Option<String>>,
	bridge_id: Lazy<Option<String>>,
	callback_url: Lazy<Option<String>>,
	direction: Lazy<String>,
	from: Lazy<String>,
	recording_file_format: Lazy<Option<String>>,
	recording_enabled: Lazy<bool>,
	start_time: Lazy<String>,
	state: Lazy<State>,
	to: Lazy<String>,
	transcription_enabled: Lazy<bool>,
	display_name: Lazy<Option<String>>,
	preferred_id: Lazy<Option<String>>,
	withhold_caller_name: Lazy<Option<bool>>,
	withhold_caller_number: Lazy<Option<bool>>,
	end_time: Lazy<Option<String>>
}
impl Data{
	fn from_info(info: &CallInfo) -> BResult<Data>{
		Ok(Data{
			active_time: Available(info.activeTime.clone()),
			bridge_id: Available(match info.bridge{
				Some(ref url) => Some(try!(util::get_id_from_location_url(url))),
				None => None
			}),
			callback_url: Available(info.callbackUrl.clone()),
			direction: Available(info.direction.clone()),
			from: Available(info.from.clone()),
			recording_file_format: Available(info.recordingFileFormat.clone()),
			recording_enabled: Available(info.recordingEnabled),
			start_time: Available(info.startTime.clone()),
			state: Available(match info.state.as_ref(){
				"started" => State::Started,
				"rejected" => State::Rejected,
				"active" => State::Active,
				"completed" => State::Completed,
				"transferring" => State::Transferring,
				state @ _ => return Err(BError::unexpected(
					&format!("unknown Call state: {}", state)
				))
			}),
			to: Available(info.to.clone()),
			transcription_enabled: Available(info.transcriptionEnabled),
			display_name: Available(info.displayName.clone()),
			preferred_id: Available(info.preferredId.clone()),
			withhold_caller_name: Available(info.withholdCallerName),
			withhold_caller_number: Available(info.withholdCallerNumber),
			end_time: Available(info.endTime.clone())
		})
	}
}

mod info{
	#![allow(non_snake_case)]
	#[derive(RustcDecodable)]
	pub struct CallInfo{
		pub id: String,
		pub activeTime: Option<String>,
		pub bridge: Option<String>,
		pub callbackUrl: Option<String>,
		pub direction: String,
		pub from: String,
		pub recordingFileFormat: Option<String>,
		pub recordingEnabled: bool,
		pub startTime: String,
		pub state: String,
		pub to: String,
		pub transcriptionEnabled: bool,
		pub displayName: Option<String>,
		pub preferredId: Option<String>,
		pub withholdCallerName: Option<bool>,
		pub withholdCallerNumber: Option<bool>,
		pub endTime: Option<String>
	}
}



#[derive(RustcDecodable)]
pub struct Event{
	id: String,
	time: String,
	name: String,
	data: Option<String>
}
impl Event{
	pub fn get_id(&self) -> String{
		self.id.clone()
	}
	pub fn get_time(&self) -> String{
		self.time.clone()
	}
	pub fn get_name(&self) -> String{
		self.name.clone()
	}
	pub fn get_data(&self) -> Option<String>{
		self.data.clone()
	}
}

#[derive(Clone)]
enum GatherPromptType{
	Sentence{
		text: String,
		voice: Voice
	},
	File{
		url: String
	}
}
#[derive(Clone)]
struct GatherPrompt{
	bargeable: bool,
	loop_audio: bool,
	prompt_type: GatherPromptType
	
}
pub struct GatherConfig{
	max_digits: u32,
	inter_digit_timeout: u32,
	terminating_digits: String,
	tag: Option<String>,
	prompt: Option<GatherPrompt>
}
impl GatherConfig{
	pub fn max_digits<'a>(&'a mut self, value: u32) -> &'a GatherConfig{
		self.max_digits = value;
		self
	}
	pub fn inter_digit_timeout<'a>(&'a mut self, value: u32) -> &'a GatherConfig{
		self.inter_digit_timeout = value;
		self
	}
	pub fn terminating_digits<'a>(&'a mut self, digits: &str) -> &'a GatherConfig{
		self.terminating_digits = digits.to_string();
		self
	}
	pub fn prompt_sentence<'a>(&'a mut self, sentence: &str, loop_audio: bool, bargeable: bool, voice: Voice) -> &'a GatherConfig{
		self.prompt = Some(GatherPrompt{
			prompt_type: GatherPromptType::Sentence{
				text: sentence.to_string(),
				voice: voice
			},
			bargeable: bargeable,
			loop_audio: loop_audio
		});
		self
	}
	
}
struct Config{
	pub call_timeout: Option<u64>,
	pub callback_url: Option<String>,
	pub callback_timeout: Option<u64>,
	pub callback_http_method: Option<String>,
	pub fallback_url: Option<String>,
	pub bridge_id: Option<String>,
	pub conference_id: Option<String>,
	pub recording_enabled: bool,
	pub recording_max_duration: Option<u64>,
	pub transcription_enabled: bool,
	pub tag: Option<String>	
}
impl Config{
	pub fn new() -> Config{
		Config{
			call_timeout: None,
			callback_url: None,
			callback_timeout: None,
			callback_http_method: None,
			fallback_url: None,
			bridge_id: None,
			conference_id: None,
			recording_enabled: false,
			recording_max_duration: None,
			transcription_enabled: false,
			tag: None	
		}
	}
}
pub struct CallBuilder{
	client: Client,
	from: String,
	to: String,
	config: Config
}
impl CallBuilder{
	pub fn call_timeout(mut self, secs: u64) -> Self{
		self.config.call_timeout = Some(secs); self
	}
	pub fn callback_url(mut self, url: &str) -> Self{
		self.config.callback_url = Some(url.to_owned()); self
	}
	pub fn callback_timeout(mut self, millis: u64) -> Self{
		self.config.callback_timeout = Some(millis); self
	}
	pub fn callback_http_method(mut self, method: &str) -> Self{
		self.config.callback_http_method = Some(method.to_owned()); self 
	}
	pub fn fallback_url(mut self, url: &str) -> Self{
		self.config.fallback_url = Some(url.to_owned()); self
	}
	pub fn bridge_id(mut self, id: &str) -> Self{
		self.config.bridge_id = Some(id.to_owned()); self
	}
	pub fn conference_id(mut self, id: &str) -> Self{
		self.config.conference_id = Some(id.to_owned()); self
	}
	pub fn enable_recording(mut self) -> Self{
		self.config.recording_enabled = true; self
	}
	pub fn recording_max_duration(mut self, duration: u64) -> Self{
		self.config.recording_max_duration = Some(duration); self
	}
	pub fn enable_transcription(mut self) -> Self{
		self.config.transcription_enabled = true; self
	}
	pub fn tag(mut self, tag: &str) -> Self{
		self.config.tag = Some(tag.to_owned()); self
	}
	pub fn create(&self) -> BResult<Call>{
		let path = "users/".to_string() + &self.client.get_user_id() + "/calls";
		let json = json!({
			"from" => (self.from),
			"to" => (self.to),
			"callTimeout" => (self.config.call_timeout),
			"callbackUrl" => (self.config.callback_url),
			"callbackTimeout" => (self.config.callback_timeout),
			"callbackHttpMethod" => (self.config.callback_http_method),
			"fallbackUrl" => (self.config.fallback_url),
			"bridgeId" => (self.config.bridge_id),
			"conferenceId" => (self.config.conference_id),
			"recordingEnabled" => (self.config.recording_enabled),
			"recordingMaxDuration" => (self.config.recording_max_duration),
			"transcriptionEnabled" => (self.config.transcription_enabled),
			"tag" => (self.config.tag)
		});
		let res:EmptyResponse = try!(self.client.raw_post_request(&path, (), json));
		let id = try!(util::get_id_from_location_header(&res.headers));
		Ok(Call{
			id: id,
			client: self.client.clone(),
			data: Arc::new(Mutex::new(Data{
				active_time: NotLoaded,
				bridge_id: NotLoaded,
				callback_url: Available(self.config.callback_url.clone()),
				direction: Available("out".to_string()),
				from: Available(self.from.to_string()),
				to: Available(self.to.to_string()),
				recording_file_format: NotLoaded,
				recording_enabled: NotLoaded,
				start_time: NotLoaded,
				state: NotLoaded,
				transcription_enabled: NotLoaded,
				display_name: NotLoaded,
				preferred_id: NotLoaded,
				withhold_caller_name: NotLoaded,
				withhold_caller_number: NotLoaded,
				end_time: NotLoaded
			}))
		})
	}
}

#[derive(Clone)]
pub struct Call{
	id: String,
	client: Client,
	data: Arc<Mutex<Data>> 
}

impl Call{
	pub fn load(&self) -> BResult<()>{
		let path = "users/".to_string() + &self.client.get_user_id() + "/calls/" + &self.id;
		let res:JsonResponse<CallInfo> = try!(self.client.raw_get_request(&path, (), ()));
		let mut data = self.data.lock().unwrap();
		*data = try!(Data::from_info(&res.body));
		Ok(())
	}

	pub fn build(client: &Client, from: &str, to: &str) -> CallBuilder{
		CallBuilder{
			client: client.clone(),
			from: from.to_owned(),
			to: to.to_owned(),
			config: Config::new()
		}
	}
	pub fn get_calls_from_bridge(bridge: &Bridge) -> BResult<Vec<Call>>{
		let client = bridge.get_client();
		let path = "users/".to_string() + &client.get_user_id() + "/bridges/" + &bridge.get_id() + "/calls";
		let res:JsonResponse<Vec<CallInfo>> = try!(client.raw_get_request(&path, (), ()));
		let mut output = vec!();
		for info in res.body{
			output.push(Call{
				id: info.id.clone(),
				client: client.clone(),
				data: Arc::new(Mutex::new(try!(Data::from_info(&info))))
			});
		}
		Ok(output)
	}
	pub fn from_call_event(event: &CallEvent) -> Call{
		Call{
			id: event.get_call_id(),
			client: event.get_client(),
			data: Arc::new(Mutex::new(Data{
				active_time: NotLoaded,
				bridge_id: NotLoaded,
				callback_url: NotLoaded,
				direction: NotLoaded,
				from: Lazy::load_if_available(event.get_from()),
				to: Lazy::load_if_available(event.get_to()),
				recording_file_format: NotLoaded,
				recording_enabled: NotLoaded,
				start_time: NotLoaded,
				state: NotLoaded,
				transcription_enabled: NotLoaded,
				display_name: Available(event.get_display_name()),
				preferred_id: Available(event.get_preferred_id()),
				withhold_caller_name: Available(event.get_withhold_caller_name()),
				withhold_caller_number: Available(event.get_withhold_caller_number()),
				end_time: NotLoaded
			}))
		}
	}
	pub fn add_to_new_bridge(&self, bridge_audio: bool, additional_phone_ids: &Vec<String>) -> BResult<Bridge>{
		let mut calls = additional_phone_ids.clone();
		calls.push(self.get_id());
		Bridge::create(&self.client, bridge_audio, &calls)
	}
	
	/* Actions */
	fn update(&self, json_data: &Json) -> BResult<()>{
		let path = "users/".to_string() + &self.client.get_user_id() + "/calls/" + &self.id;
		let _:EmptyResponse = try!(self.client.raw_post_request(&path, (), json_data));
		Ok(())
	}
	pub fn hang_up(&self) -> BResult<()>{
		let mut data = self.data.lock().unwrap();
		data.state = NotLoaded;
		data.end_time = NotLoaded;
		self.update(&json!({
			"state" => "completed"
		}))
	}
	
	pub fn answer_incoming(&self) -> BResult<()>{
		let mut data = self.data.lock().unwrap();
		data.state = NotLoaded;
		data.active_time = NotLoaded;
		self.update(&json!({
			"state" => "active"
		}))
	}
	pub fn reject_incoming(&self) -> BResult<()>{
		let mut data = self.data.lock().unwrap();
		data.state = NotLoaded;
		data.active_time = NotLoaded;
		self.update(&json!({
			"state" => "rejected"
		}))
	}
	pub fn enable_recording(&self, enable: bool) -> BResult<()>{
		let mut data = self.data.lock().unwrap();
		data.recording_file_format = NotLoaded;
		data.recording_enabled = Available(enable);
		data.transcription_enabled = NotLoaded;
		self.update(&json!({
			"recordingEnabled" => (enable)
		}))
	}
	pub fn play_audio_file(&self, url: &str, loop_audio: bool, tag: Option<&str>) -> BResult<()>{
		let path = "users/".to_string() + &self.client.get_user_id() + "/calls/" + &self.id + "/audio";
		let json = json!({
			"fileUrl" => (url),
			"loopEnabled" => (loop_audio),
			"tag" => (tag.map(|a|a.to_string()))
		});
		let _:EmptyResponse = try!(self.client.raw_post_request(&path, (), json));
		Ok(())
	}
	pub fn stop_audio_file(&self) -> BResult<()>{
		let path = "users/".to_string() + &self.client.get_user_id() + "/calls/" + &self.id + "/audio";
		let json = json!({
			"fileUrl" => ""
		});
		let _:EmptyResponse = try!(self.client.raw_post_request(&path, (), json));
		Ok(())
	}
	pub fn speak_sentence(&self, sentence: &str, loop_audio: bool, voice: Voice, tag: Option<&str>) -> BResult<()>{
		let path = "users/".to_string() + &self.client.get_user_id() + "/calls/" + &self.id + "/audio";
		let json = json!({
			"sentence" => (sentence),
			"loopEnabled" => (loop_audio),
			"voice" => (voice.get_name()),
			"tag" => (tag.map(|a|a.to_string()))
		});
		let _:EmptyResponse = try!(self.client.raw_post_request(&path, (), json));
		Ok(())
	}
	pub fn send_dtmf(&self, digits: &str) -> BResult<()>{
		let path = "users/".to_string() + &self.client.get_user_id() + "/calls/" + &self.id + "/dtmf";
		let json = json!({
			"dtmfOut" => (digits)
		});
		let _:EmptyResponse = try!(self.client.raw_post_request(&path, (), json));
		Ok(())
	}
	
	pub fn gather_dtmf(&self, config: &GatherConfig) -> BResult<()>{
		let path = "users/".to_string() + &self.client.get_user_id() + "/calls/" + &self.id + "/gather";
		let prompt = config.prompt.clone().map(|prompt|{
			match prompt.prompt_type{
				GatherPromptType::Sentence{text, voice} => {
					json!({
						"sentence" => (text),
						"loopEnabled" => (prompt.loop_audio),
						"bargeable" => (prompt.bargeable),
						"voice" => (voice.get_name())
					})
				},
				GatherPromptType::File{url} => {
					json!({
						"fileUrl" => (url),
						"loopEnabled" => (prompt.loop_audio),
						"bargeable" => (prompt.bargeable)
					})
				}
			}
		});
		let json = json!({
			"maxDigits" => (config.max_digits),
			"interDigitTimeout" => (config.inter_digit_timeout),
			"terminatingDigits" => (config.terminating_digits),
			"tag" => (config.tag),
			"prompt" => (prompt)
		});
		
		let _:EmptyResponse = try!(self.client.raw_post_request(&path, (), json));
		//let id = try!(util::get_id_from_location_header(&res.headers));
		Ok(())
	}
	
	/* Getters */
	pub fn get_id(&self) -> String{
		self.id.clone()
	}
	pub fn get_client(&self) -> Client{
		self.client.clone()
	}
	pub fn get_active_time(&self) -> BResult<Option<String>>{
		if !self.data.lock().unwrap().active_time.available(){
			try!(self.load());
		}
		Ok(try!(self.data.lock().unwrap().active_time.get()).clone())
	}
	pub fn get_bridge_id(&self) -> BResult<Option<String>>{
		if !self.data.lock().unwrap().bridge_id.available(){
			try!(self.load());
		}
		Ok(try!(self.data.lock().unwrap().bridge_id.get()).clone())
	}
	pub fn get_callback_url(&self) -> BResult<Option<String>>{
		if !self.data.lock().unwrap().callback_url.available(){
			try!(self.load());
		}
		Ok(try!(self.data.lock().unwrap().callback_url.get()).clone())
	}
	pub fn get_direction(&self) -> BResult<String>{
		if !self.data.lock().unwrap().direction.available(){
			try!(self.load());
		}
		Ok(try!(self.data.lock().unwrap().direction.get()).clone())
	}
	pub fn get_from(&self) -> BResult<String>{
		if !self.data.lock().unwrap().from.available(){
			try!(self.load());
		}
		Ok(try!(self.data.lock().unwrap().from.get()).clone())
	}
	pub fn get_recording_file_format(&self) -> BResult<Option<String>>{
		if !self.data.lock().unwrap().recording_file_format.available(){
			try!(self.load());
		}
		Ok(try!(self.data.lock().unwrap().recording_file_format.get()).clone())
	}
	pub fn get_recording_enabled(&self) -> BResult<bool>{
		if !self.data.lock().unwrap().recording_enabled.available(){
			try!(self.load());
		}
		Ok(try!(self.data.lock().unwrap().recording_enabled.get()).clone())
	}
	pub fn get_start_time(&self) -> BResult<String>{
		if !self.data.lock().unwrap().start_time.available(){
			try!(self.load());
		}
		Ok(try!(self.data.lock().unwrap().start_time.get()).clone())
	}
	pub fn get_state(&self) -> BResult<State>{
		if !self.data.lock().unwrap().state.available(){
			try!(self.load());
		}
		Ok(try!(self.data.lock().unwrap().state.get()).clone())
	}
	pub fn get_to(&self) -> BResult<String>{
		if !self.data.lock().unwrap().to.available(){
			try!(self.load());
		}
		Ok(try!(self.data.lock().unwrap().to.get()).clone())
	}
	pub fn get_transcription_enabled(&self) -> BResult<bool>{
		if !self.data.lock().unwrap().transcription_enabled.available(){
			try!(self.load());
		}
		Ok(try!(self.data.lock().unwrap().transcription_enabled.get()).clone())
	}
	pub fn get_display_name(&self) -> BResult<Option<String>>{
		if !self.data.lock().unwrap().display_name.available(){
			try!(self.load());
		}
		Ok(try!(self.data.lock().unwrap().display_name.get()).clone())
	}
	pub fn get_preferred_id(&self) -> BResult<Option<String>>{
		if !self.data.lock().unwrap().preferred_id.available(){
			try!(self.load());
		}
		Ok(try!(self.data.lock().unwrap().preferred_id.get()).clone())
	}
	pub fn get_withhold_caller_name(&self) -> BResult<Option<bool>>{
		if !self.data.lock().unwrap().withhold_caller_name.available(){
			try!(self.load());
		}
		Ok(try!(self.data.lock().unwrap().withhold_caller_name.get()).clone())
	}
	pub fn get_withhold_caller_number(&self) -> BResult<Option<bool>>{
		if !self.data.lock().unwrap().withhold_caller_number.available(){
			try!(self.load());
		}
		Ok(try!(self.data.lock().unwrap().withhold_caller_number.get()).clone())
	}
	
	pub fn get_bridge(&self) -> BResult<Option<Bridge>>{
		Ok(match try!(self.get_bridge_id()){
			Some(id) => Some(Bridge::get_by_id(&self.client, &id)),
			None => None
		})
	}
	pub fn get_events(&self) -> BResult<Vec<Event>>{
		let path = "users/".to_string() + &self.client.get_user_id() + "/calls/" + &self.id + "/events";
		let res:JsonResponse<Vec<Event>> = try!(self.client.raw_get_request(&path, (), ()));
		Ok(res.body)
	}
	pub fn get_event(&self, id: &str) -> BResult<Event>{
		let path = "users/".to_string() + &self.client.get_user_id() + "/calls/" + &self.id + "/events/" + id;
		let res:JsonResponse<Event> = try!(self.client.raw_get_request(&path, (), ()));
		Ok(res.body)
	}
}
