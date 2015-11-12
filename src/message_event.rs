use {BResult, BError};
use client::Client;
use self::info::MessageEventInfo;
use rustc_serialize::json;
use message::{Message, State};
use application::Application;

pub struct MessageEvent{
	client: Client,
	message_id: String,
	to: String,
	from: String,
	time: String,
	text: String,
	inbound: bool,
	state: State,
	application_id: Option<String>
}
impl MessageEvent{
	pub fn parse(client: &Client, data: &str) -> BResult<MessageEvent>{
		let info: MessageEventInfo = try!(json::decode(data));
		Ok(MessageEvent{
			client: client.clone(),
			message_id: info.messageId.clone(),
			to: info.to.clone(),
			from: info.from.clone(),
			time: info.time.clone(),
			text: info.text.clone(),
			inbound: match info.direction.as_ref(){
				"in" => true,
				"out" => false,
				direction @ _ => return Err(BError::unexpected(
					&format!("unknown MessageEvent direction: {}", direction)
				))
			},
			state: try!(State::parse(&info.state)),
			application_id: info.applicationId.clone()
		})
	}
}
impl MessageEvent{
	pub fn get_client(&self) -> Client{
		self.client.clone()
	}
	pub fn get_message_id(&self) -> String{
		self.message_id.clone()
	}
	pub fn get_message(&self) -> Message{
		Message::from_event(self)
	} 
	pub fn get_to(&self) -> String{
		self.to.clone()
	}
	pub fn get_from(&self) -> String{
		self.from.clone()
	}
	pub fn get_time(&self) -> String{
		self.time.clone()
	}
	pub fn get_text(&self) -> String{
		self.text.clone()
	}
	pub fn is_inbound(&self) -> bool{
		self.inbound
	}
	pub fn is_outbound(&self) -> bool{
		!self.is_inbound()
	}
	pub fn get_state(&self) -> State{
		self.state.clone()
	}
	pub fn get_application(&self) -> Option<Application>{
		self.application_id.clone().map(|id|{
			self.client.get_application(&id)
		})
	}
}
mod info{
	#![allow(non_snake_case)]
	#[derive(RustcDecodable)]
	pub struct MessageEventInfo{
		pub to: String,
		pub from: String,
		pub time: String,
		pub text: String,
		pub direction: String,
		pub applicationId: Option<String>,
		pub state: String,
		pub eventType: String,
		pub messageId: String,
		pub media: Option<Vec<String>>
	}
}