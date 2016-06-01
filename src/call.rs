use {CatapultResult, CatapultError};
use client::{EmptyResponse, JsonResponse, Client};
use std::sync::{Arc, Mutex};
use call_event::CallEvent;
use bridge::Bridge;
use util;
use lazy::Lazy;
use lazy::Lazy::*;
use self::info::CallInfo;
use rustc_serialize::json::{ToJson, Json};
use rustc_serialize::json;
use voice::Voice;
use std::collections::BTreeMap;

#[derive(Clone, Debug)]
pub enum State{
	Started,
	Rejected,
	Active,
	Completed,
	Transferring
}
impl State{
	pub fn to_string(&self) -> &'static str{
		use self::State::*;
		match *self{
			Started => "started",
			Rejected => "rejected",
			Active => "active",
			Completed => "completed",
			Transferring => "transferring"
		}
	}
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
	fn from_info(info: &CallInfo) -> CatapultResult<Data>{
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
				state @ _ => return Err(CatapultError::unexpected(
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
	pub fn create(self) -> CatapultResult<Call>{
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
		let res:EmptyResponse = try!(self.client.raw_post_request(&path, (), &json));
		let id = try!(util::get_id_from_location_header(&res.headers));
		Ok(Call{
			id: id,
			client: self.client,
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
fn get_call_list<P: json::ToJson>(client: &Client, path: &str, params: P) -> CatapultResult<QueryResult>{
	let res:JsonResponse<Vec<CallInfo>> = try!(client.raw_get_request(&path, params, ()));
	let mut output = vec!();
	for info in res.body{
		output.push(Call{
			id: info.id.clone(),
			client: client.clone(),
			data: Arc::new(Mutex::new(try!(Data::from_info(&info))))
		});
	}
	let next_url = try!(util::get_next_link_from_headers(&res.headers));
	Ok(QueryResult{
		client: client.clone(),
		data: output,
		next_url: next_url
	})
}
pub struct QueryResult{
	client: Client,
	data: Vec<Call>,
	next_url: Option<String>
}
impl QueryResult{
	pub fn get_calls(&self) -> &Vec<Call>{
		&self.data
	}
	pub fn has_next(&self) -> bool{
		self.next_url.is_some()
	}
	pub fn next(&self) -> Option<CatapultResult<QueryResult>>{
		self.next_url.as_ref().map(|ref url|{
			get_call_list(&self.client, &url, ())
		})
	}
}

pub struct Query{
	client: Client,
	from: Option<String>,
	to: Option<String>,
	size: Option<u32>,
	state: Option<State>,
	sort_order: Option<String>
}
impl Query{
	pub fn from(mut self, from: &str) -> Query{
		self.from = Some(from.to_string()); self 
	}
	pub fn to(mut self, to: &str) -> Query{
		self.to = Some(to.to_string()); self 
	}
	pub fn state(mut self, state: State) -> Query{
		self.state = Some(state); self
	}
	pub fn size(mut self, size: u32) -> Query{
		self.size = Some(size); self
	}
	pub fn sort_desc(mut self) -> Query{
		self.sort_order = Some("desc".to_string()); self
	}
	pub fn submit(self) -> CatapultResult<QueryResult>{
		let path = "users/".to_string() + &self.client.get_user_id() + "/calls";
		
		let mut map = BTreeMap::new();
		if let Some(from) = self.from{
			map.insert("from".to_string(), from.to_json());
		}
		if let Some(to) = self.to{
			map.insert("to".to_string(), to.to_json());
		}
		if let Some(state) = self.state{
			map.insert("state".to_string(), state.to_string().to_json());
		}
		if let Some(size) = self.size{
			map.insert("size".to_string(), size.to_json());
		}
		if let Some(sort_order) = self.sort_order{
			map.insert("sortOrder".to_string(), sort_order.to_json());
		}
		let json = Json::Object(map);
		
		get_call_list(&self.client, &path, json)
	}
}

#[derive(Clone)]
pub struct Call{
	id: String,
	client: Client,
	data: Arc<Mutex<Data>> 
}

impl Call{
	pub fn load(&self) -> CatapultResult<()>{
		let path = "users/".to_string() + &self.client.get_user_id() + "/calls/" + &self.id;
		let res:JsonResponse<CallInfo> = try!(self.client.raw_get_request(&path, (), ()));
		let mut data = self.data.lock().unwrap();
		*data = try!(Data::from_info(&res.body));
		Ok(())
	}
	pub fn query(client: &Client) -> Query{
		Query{
			client: client.clone(),
			from: None,
			to: None,
			size: None,
			state: None,
			sort_order: None
		}
	}
	pub fn build(client: &Client, from: &str, to: &str) -> CallBuilder{
		CallBuilder{
			client: client.clone(),
			from: from.to_owned(),
			to: to.to_owned(),
			config: Config::new()
		}
	}
	pub fn get_calls_from_bridge(bridge: &Bridge) -> CatapultResult<Vec<Call>>{
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
	pub fn add_to_new_bridge(&self, bridge_audio: bool, additional_phone_ids: &Vec<String>) -> CatapultResult<Bridge>{
		let mut calls = additional_phone_ids.clone();
		calls.push(self.get_id());
		Bridge::create(&self.client, bridge_audio, &calls)
	}
	
	/* Actions */
	fn update(&self, json_data: &Json) -> CatapultResult<()>{
		let path = "users/".to_string() + &self.client.get_user_id() + "/calls/" + &self.id;
		let _:EmptyResponse = try!(self.client.raw_post_request(&path, (), json_data));
		Ok(())
	}
	pub fn hang_up(&self) -> CatapultResult<()>{
		let mut data = self.data.lock().unwrap();
		data.state = NotLoaded;
		data.end_time = NotLoaded;
		self.update(&json!({
			"state" => "completed"
		}))
	}
	
	pub fn answer_incoming(&self) -> CatapultResult<()>{
		let mut data = self.data.lock().unwrap();
		data.state = NotLoaded;
		data.active_time = NotLoaded;
		self.update(&json!({
			"state" => "active"
		}))
	}
	pub fn reject_incoming(&self) -> CatapultResult<()>{
		let mut data = self.data.lock().unwrap();
		data.state = NotLoaded;
		data.active_time = NotLoaded;
		self.update(&json!({
			"state" => "rejected"
		}))
	}
	pub fn enable_recording(&self, enable: bool) -> CatapultResult<()>{
		let mut data = self.data.lock().unwrap();
		data.recording_file_format = NotLoaded;
		data.recording_enabled = Available(enable);
		data.transcription_enabled = NotLoaded;
		self.update(&json!({
			"recordingEnabled" => (enable)
		}))
	}
	pub fn play_audio_file(&self, url: &str, loop_audio: bool, tag: Option<&str>) -> CatapultResult<()>{
		let path = "users/".to_string() + &self.client.get_user_id() + "/calls/" + &self.id + "/audio";
		let json = json!({
			"fileUrl" => (url),
			"loopEnabled" => (loop_audio),
			"tag" => (tag.map(|a|a.to_string()))
		});
		let _:EmptyResponse = try!(self.client.raw_post_request(&path, (), &json));
		Ok(())
	}
	///Stops either an audio file playing, or a sentence being spoken
	///This is the only way to stop audio in a loop
	pub fn stop_audio(&self) -> CatapultResult<()>{
		let path = "users/".to_string() + &self.client.get_user_id() + "/calls/" + &self.id + "/audio";
		let json = json!({
			"fileUrl" => ""
		});
		let _:EmptyResponse = try!(self.client.raw_post_request(&path, (), &json));
		Ok(())
	}
	pub fn speak_sentence(&self, sentence: &str, loop_audio: bool, voice: Voice, tag: Option<&str>) -> CatapultResult<()>{
		let path = "users/".to_string() + &self.client.get_user_id() + "/calls/" + &self.id + "/audio";
		let json = json!({
			"sentence" => (sentence),
			"loopEnabled" => (loop_audio),
			"voice" => (voice.get_name()),
			"tag" => (tag.map(|a|a.to_string()))
		});
		let _:EmptyResponse = try!(self.client.raw_post_request(&path, (), &json));
		Ok(())
	}
	pub fn send_dtmf(&self, digits: &str) -> CatapultResult<()>{
		let path = "users/".to_string() + &self.client.get_user_id() + "/calls/" + &self.id + "/dtmf";
		let json = json!({
			"dtmfOut" => (digits)
		});
		let _:EmptyResponse = try!(self.client.raw_post_request(&path, (), &json));
		Ok(())
	}
	
	pub fn gather_dtmf(&self, config: &GatherConfig) -> CatapultResult<()>{
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
		
		let _:EmptyResponse = try!(self.client.raw_post_request(&path, (), &json));
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
	pub fn get_active_time(&self) -> CatapultResult<Option<String>>{
		lazy_load!(self, active_time)
	}
	pub fn get_bridge_id(&self) -> CatapultResult<Option<String>>{
		lazy_load!(self, bridge_id)
	}
	pub fn get_callback_url(&self) -> CatapultResult<Option<String>>{
		lazy_load!(self, callback_url)
	}
	pub fn get_direction(&self) -> CatapultResult<String>{
		lazy_load!(self, direction)
	}
	pub fn get_from(&self) -> CatapultResult<String>{
		lazy_load!(self, from)
	}
	pub fn get_recording_file_format(&self) -> CatapultResult<Option<String>>{
		lazy_load!(self, recording_file_format)
	}
	pub fn get_recording_enabled(&self) -> CatapultResult<bool>{
		lazy_load!(self, recording_enabled)
	}
	pub fn get_start_time(&self) -> CatapultResult<String>{
		lazy_load!(self, start_time)
	}
	pub fn get_state(&self) -> CatapultResult<State>{
		lazy_load!(self, state)
	}
	pub fn get_to(&self) -> CatapultResult<String>{
		lazy_load!(self, to)
	}
	pub fn get_transcription_enabled(&self) -> CatapultResult<bool>{
		lazy_load!(self, transcription_enabled)
	}
	pub fn get_display_name(&self) -> CatapultResult<Option<String>>{
		lazy_load!(self, display_name)
	}
	pub fn get_preferred_id(&self) -> CatapultResult<Option<String>>{
		lazy_load!(self, preferred_id)
	}
	pub fn get_withhold_caller_name(&self) -> CatapultResult<Option<bool>>{
		lazy_load!(self, withhold_caller_name)
	}
	pub fn get_withhold_caller_number(&self) -> CatapultResult<Option<bool>>{
		lazy_load!(self, withhold_caller_number)
	}
	
	pub fn get_bridge(&self) -> CatapultResult<Option<Bridge>>{
		Ok(match try!(self.get_bridge_id()){
			Some(id) => Some(Bridge::get_by_id(&self.client, &id)),
			None => None
		})
	}
	pub fn get_events(&self) -> CatapultResult<Vec<Event>>{
		let path = "users/".to_string() + &self.client.get_user_id() + "/calls/" + &self.id + "/events";
		let res:JsonResponse<Vec<Event>> = try!(self.client.raw_get_request(&path, (), ()));
		Ok(res.body)
	}
	pub fn get_event(&self, id: &str) -> CatapultResult<Event>{
		let path = "users/".to_string() + &self.client.get_user_id() + "/calls/" + &self.id + "/events/" + id;
		let res:JsonResponse<Event> = try!(self.client.raw_get_request(&path, (), ()));
		Ok(res.body)
	}
	pub fn get_end_time(&self) -> CatapultResult<Option<String>>{
		lazy_load!(self, end_time)
	}
}
