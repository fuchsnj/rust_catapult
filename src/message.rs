use {BResult, BError};
use client::{EmptyResponse, JsonResponse, Client};
use std::sync::{Arc, Mutex};
use lazy::Lazy;
use lazy::Lazy::*;
use rustc_serialize::json::ToJson;
use util;
use message_event::MessageEvent;
use self::info::MessageInfo;

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
		//pub media: Vec<String>,
		pub time: String
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
	pub fn parse(state: &str) -> BResult<State>{
		Ok(match state.as_ref(){
			"received" => State::Received,
			"queued" => State::Queued,
			"sending" => State::Sending,
			"sent" => State::Sent,
			"error" => State::Error,
			err @ _ => return Err(BError::unexpected(
				&format!("unknown Message state: {}", err)
			))
		})
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
	pub fn create(self) -> BResult<Message>{
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
		let res:EmptyResponse = try!(self.client.raw_post_request(&path, (), json));
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
				time: NotLoaded
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
	time: Lazy<String>
}
impl Data{
	fn from_info(info: &MessageInfo) -> BResult<Data>{
		Ok(Data{
			inbound: Available(match info.direction.as_ref(){
				"in" => true,
				"out" => false,
				direction @ _ => return Err(BError::unexpected(
					&format!("unknown Message direction: {}", direction)
				))
			}),
			from: Available(info.from.clone()),
			to: Available(info.to.clone()),
			state: Available(try!(State::parse(&info.state))),
			text: Available(info.text.clone()),
			time: Available(info.time.clone())
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
				time: NotLoaded
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
				time: Available(event.get_time())
			}))
		}
	}
	pub fn load(&self) -> BResult<()>{
		let path = "users/".to_string() + &self.client.get_user_id() + "/messages/" + &self.id;
		let res:JsonResponse<MessageInfo> = try!(self.client.raw_get_request(&path, (), ()));
		let mut data = self.data.lock().unwrap();
		*data = try!(Data::from_info(&res.body));
		Ok(())
	}
	
	/* Getters */
	pub fn get_id(&self) -> String{
		self.id.clone()
	}
	pub fn get_client(&self) -> Client{
		self.client.clone()
	}
	pub fn is_inbound(&self) -> BResult<bool>{
		if !self.data.lock().unwrap().inbound.available(){
			try!(self.load());
		}
		Ok(try!(self.data.lock().unwrap().inbound.get()).clone())
	}
	pub fn is_outbound(&self) -> BResult<bool>{
		Ok(! try!(self.is_inbound()))
	}
	pub fn get_from(&self) -> BResult<String>{
		if !self.data.lock().unwrap().from.available(){
			try!(self.load());
		}
		Ok(try!(self.data.lock().unwrap().from.get()).clone())
	}
	pub fn get_to(&self) -> BResult<String>{
		if !self.data.lock().unwrap().to.available(){
			try!(self.load());
		}
		Ok(try!(self.data.lock().unwrap().to.get()).clone())
	}
	pub fn get_state(&self) -> BResult<State>{
		if !self.data.lock().unwrap().state.available(){
			try!(self.load());
		}
		Ok(try!(self.data.lock().unwrap().state.get()).clone())
	}
	pub fn get_text(&self) -> BResult<String>{
		if !self.data.lock().unwrap().text.available(){
			try!(self.load());
		}
		Ok(try!(self.data.lock().unwrap().text.get()).clone())
	}
	pub fn get_time(&self) -> BResult<String>{
		if !self.data.lock().unwrap().time.available(){
			try!(self.load());
		}
		Ok(try!(self.data.lock().unwrap().time.get()).clone())
	}
}
