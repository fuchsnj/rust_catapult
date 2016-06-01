use {CatapultResult, CatapultError};
use client::{EmptyResponse, JsonResponse, Client};
use std::sync::{Arc, Mutex};
use lazy::Lazy;
use lazy::Lazy::*;
use util;
use message_event::MessageEvent;
use media::Media;
use self::info::MessageInfo;
use std::collections::BTreeMap;
use rustc_serialize::json::{ToJson, Json};
use rustc_serialize::json;

pub struct QueryResult{
	client: Client,
	data: Vec<Message>,
	next_url: Option<String>
}
impl QueryResult{
	pub fn get_messages(&self) -> &Vec<Message>{
		&self.data
	}
	pub fn has_next(&self) -> bool{
		self.next_url.is_some()
	}
	pub fn next(&self) -> Option<CatapultResult<QueryResult>>{
		self.next_url.as_ref().map(|ref url|{
			get_message_list(&self.client, &url, ())
		})
	}
}

mod info{
	#![allow(non_snake_case)]
	#[derive(RustcDecodable)]
	pub struct MessageInfo{
		pub id: String,
		pub direction: String,
		pub from: String,
		pub to: String,
		pub state: String,
		pub text: String,
		pub time: String,
		pub media: Option<Vec<String>>
	}
}

#[derive(Clone, Debug)]
pub enum State{
	Received,
	Queued,
	Sending,
	Sent,
	Error
}
impl State{
	pub fn parse(state: &str) -> CatapultResult<State>{
		Ok(match state.as_ref(){
			"received" => State::Received,
			"queued" => State::Queued,
			"sending" => State::Sending,
			"sent" => State::Sent,
			"error" => State::Error,
			err @ _ => return Err(CatapultError::unexpected(
				&format!("unknown Message state: {}", err)
			))
		})
	}
	pub fn to_string(&self) -> &str{
		use self::State::*;
		match *self{
			Received => "received",
			Queued => "queued",
			Sending => "sending",
			Sent => "sent",
			Error => "error"
		}
	}
}
fn get_message_list<P: json::ToJson>(client: &Client, path: &str, params: P) -> CatapultResult<QueryResult>{
	let res:JsonResponse<Vec<MessageInfo>> = try!(client.raw_get_request(&path, params, ()));
	let mut output = vec!();
	for info in res.body{
		output.push(Message{
			id: info.id.clone(),
			client: client.clone(),
			data: Arc::new(Mutex::new(try!(Data::from_info(&client, &info))))
		});
	}
	let next_url = try!(util::get_next_link_from_headers(&res.headers));
	Ok(QueryResult{
		client: client.clone(),
		data: output,
		next_url: next_url
	})
}
pub struct Query{
	client: Client,
	from_number: Option<String>,
	to_number: Option<String>,
	from_time: Option<String>,
	to_time: Option<String>,
	size: Option<u32>,
	direction: Option<String>,
	state: Option<State>,
	sort_order: Option<String>
}
impl Query{
	pub fn from_time(mut self, time: &str) -> Query{
		self.from_time = Some(time.to_owned()); self
	}
	pub fn to_time(mut self, time: &str) -> Query{
		self.to_time = Some(time.to_owned()); self
	}
	pub fn from_number(mut self, number: &str) -> Query{
		self.from_number = Some(number.to_owned()); self
	}
	pub fn to_number(mut self, number: &str) -> Query{
		self.to_number = Some(number.to_owned()); self
	}
	pub fn size(mut self, size: u32) -> Query{
		self.size = Some(size); self
	}
	pub fn only_inbound(mut self) -> Query{
		self.direction = Some("in".to_owned()); self
	}
	pub fn only_outbound(mut self) -> Query{
		self.direction = Some("out".to_owned()); self
	}
	pub fn state(mut self, state: State) -> Query{
		self.state = Some(state); self
	}
	pub fn desc(mut self) -> Query{
		self.sort_order = Some("desc".to_owned()); self
	}
	pub fn submit(&self) -> CatapultResult<QueryResult>{
		let path = "users/".to_string() + &self.client.get_user_id() + "/messages";
		
		let mut map = BTreeMap::new();
		
		if let Some(ref from) = self.from_number{
			map.insert("from".to_string(), from.to_json());
		}
		if let Some(ref to) = self.to_number{
			map.insert("to".to_string(), to.to_json());
		}
		if let Some(ref from) = self.from_time{
			map.insert("fromDateTime".to_string(), from.to_json());
		}
		if let Some(ref to) = self.to_time{
			map.insert("fromDateTime".to_string(), to.to_json());
		}
		if let Some(ref direction) = self.direction{
			map.insert("direction".to_owned(), direction.to_json());
		}
		if let Some(ref state) = self.state{
			map.insert("state".to_owned(), state.to_string().to_json());
		}
		if let Some(ref sort_order) = self.sort_order{
			map.insert("sortOrder".to_owned(), sort_order.to_json());
		}
		if let Some(size) = self.size{
			map.insert("size".to_owned(), size.to_json());
		}
		let json = Json::Object(map);
		get_message_list(&self.client, &path, json)
	}
}
pub struct MessageBuilder{
	client: Client,
	from: String,
	to: String,
	text: String,
	media: Vec<String>,
	receipt_requested: bool,
	callback_url: Option<String>,
	callback_http_method: String,
	callback_timeout: Option<u64>,
	fallback_url: Option<String>,
	tag: Option<String>
}
impl MessageBuilder{
	pub fn media(mut self, url: &str) -> Self{
		self.media.push(url.to_owned()); self
	}
	pub fn request_receipt(mut self) -> Self{
		self.receipt_requested = true; self
	}
	pub fn callback_url(mut self, url: &str) -> Self{
		self.callback_url = Some(url.to_owned()); self
	}
	pub fn use_get_http_method(mut self) -> Self{
		self.callback_http_method = "GET".to_owned(); self
	}
	pub fn callback_timeout(mut self, millis: u64) -> Self{
		self.callback_timeout = Some(millis); self
	}
	pub fn fallback_url(mut self, url: &str) -> Self{
		self.fallback_url = Some(url.to_owned()); self
	}
	pub fn tag(mut self, tag: &str) -> Self{
		self.tag = Some(tag.to_owned()); self
	}
	pub fn create(self) -> CatapultResult<Message>{
		let path = "users/".to_string() + &self.client.get_user_id() + "/messages";
		let json = json!({
			"from" => (self.from),
			"to" => (self.to),
			"text" => (self.text),
			"media" => (self.media),
			"receiptRequested" => (self.receipt_requested),
			"callbackUrl" => (self.callback_url),
			"callbackHttpMethod" => (self.callback_http_method),
			"callbackTimeout" => (self.callback_timeout),
			"fallbackUrl" => (self.fallback_url),
			"tag" => (self.tag)
		});
		let res:EmptyResponse = try!(self.client.raw_post_request(&path, (), &json));
		let id = try!(util::get_id_from_location_header(&res.headers));
		Ok(Message{
			id: id,
			client: self.client,
			data: Arc::new(Mutex::new(Data{
				inbound: Available(false),
				from: Available(self.from.clone()),
				to: Available(self.to.clone()),
				state: NotLoaded,
				text: Available(self.text.clone()),
				time: NotLoaded,
				media: NotLoaded
			}))
		})
	}
}

struct Data{
	inbound: Lazy<bool>,
	from: Lazy<String>,
	to: Lazy<String>,
	state: Lazy<State>,
	text: Lazy<String>,
	time: Lazy<String>,
	media: Lazy<Vec<Media>>
}
impl Data{
	fn from_info(client: &Client, info: &MessageInfo) -> CatapultResult<Data>{
		Ok(Data{
			inbound: Available(match info.direction.as_ref(){
				"in" => true,
				"out" => false,
				direction @ _ => return Err(CatapultError::unexpected(
					&format!("unknown Message direction: {}", direction)
				))
			}),
			from: Available(info.from.clone()),
			to: Available(info.to.clone()),
			state: Available(try!(State::parse(&info.state))),
			text: Available(info.text.clone()),
			time: Available(info.time.clone()),
			media: Available(
				match info.media{
					Some(ref media) => {
						let mut output = vec!();
						for url in media{
							let filename = try!(util::get_id_from_location_url(&url));
							output.push(Media::get(client, &filename));
						}
						output
					},
					None => vec!()
				}
			)
		})
	}
}

pub struct Message{
	id: String,
	client: Client,
	data: Arc<Mutex<Data>>
}
impl Message{
	pub fn build(client: &Client, from: &str, to: &str, text: &str) -> MessageBuilder{
		MessageBuilder{
			client: client.clone(),
			from: from.to_owned(),
			to: to.to_owned(),
			text: text.to_owned(),
			media: vec!(),
			receipt_requested: false,
			callback_url: None,
			callback_http_method: "POST".to_owned(),
			callback_timeout: None,
			fallback_url: None,
			tag: None
		}
	}
	pub fn query(client: &Client) -> Query{
		Query{
			client: client.clone(),
			from_number: None,
			to_number: None,
			from_time: None,
			to_time: None,
			size: None,
			direction: None,
			state: None,
			sort_order: None
		}
	}
	pub fn get(client: &Client, id: &str) -> Message{
		Message{
			id: id.to_owned(),
			client: client.clone(),
			data: Arc::new(Mutex::new(Data{
				inbound: NotLoaded,
				from: NotLoaded,
				to: NotLoaded,
				state: NotLoaded,
				text: NotLoaded,
				time: NotLoaded,
				media: NotLoaded
			}))
		}
	}
	pub fn from_event(event: &MessageEvent) -> Message{
		Message{
			id: event.get_message_id(),
			client: event.get_client(),
			data: Arc::new(Mutex::new(Data{
				inbound: Available(event.is_inbound()),
				from: Available(event.get_from()),
				to: Available(event.get_to()),
				state: NotLoaded,
				text: Available(event.get_text()),
				time: Available(event.get_time()),
				media: Available(event.get_media())
			}))
		}
	}
	pub fn load(&self) -> CatapultResult<()>{
		let path = "users/".to_string() + &self.client.get_user_id() + "/messages/" + &self.id;
		let res:JsonResponse<MessageInfo> = try!(self.client.raw_get_request(&path, (), ()));
		let mut data = self.data.lock().unwrap();
		*data = try!(Data::from_info(&self.client, &res.body));
		Ok(())
	}
	
	/* Getters */
	pub fn get_id(&self) -> String{
		self.id.clone()
	}
	pub fn get_client(&self) -> Client{
		self.client.clone()
	}
	pub fn is_inbound(&self) -> CatapultResult<bool>{
		if !self.data.lock().unwrap().inbound.available(){
			try!(self.load());
		}
		Ok(try!(self.data.lock().unwrap().inbound.get()).clone())
	}
	pub fn is_outbound(&self) -> CatapultResult<bool>{
		Ok(! try!(self.is_inbound()))
	}
	pub fn get_from(&self) -> CatapultResult<String>{
		lazy_load!(self, from)
	}
	pub fn get_to(&self) -> CatapultResult<String>{
		lazy_load!(self, to)
	}
	pub fn get_state(&self) -> CatapultResult<State>{
		lazy_load!(self, state)
	}
	pub fn get_text(&self) -> CatapultResult<String>{
		lazy_load!(self, text)
	}
	pub fn get_time(&self) -> CatapultResult<String>{
		lazy_load!(self, time)
	}
	pub fn get_media(&self) -> CatapultResult<Vec<Media>>{
		lazy_load!(self, media)
	}
}
