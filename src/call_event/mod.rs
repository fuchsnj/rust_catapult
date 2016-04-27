use {BError, CatapultResult, Client};
use rustc_serialize::json;
use call::Call;
use self::info::CallEventInfo;

mod incoming_event;
mod answer_event;
mod hangup_event;
mod dtmf_event;
mod playback_event;
mod timeout_event;
mod gather_event;
mod recording_event;
mod speak_event;

pub use self::incoming_event::IncomingEvent;
pub use self::answer_event::AnswerEvent;
pub use self::hangup_event::HangupEvent;
pub use self::dtmf_event::DtmfEvent;
pub use self::playback_event::PlaybackEvent;
pub use self::timeout_event::TimeoutEvent;
pub use self::gather_event::GatherEvent;
pub use self::recording_event::RecordingEvent;
pub use self::speak_event::SpeakEvent;

pub struct CallEvent{
	client: Client,
	event_type: EventType,
	call_id: String,
	withhold_caller_number: Option<bool>,
	withhold_caller_name: Option<bool>,
	display_name: Option<String>,
	preferred_id: Option<String>
}

pub enum EventType{
	Incoming(IncomingEvent),
	Answer(AnswerEvent),
	Hangup(HangupEvent),
	Dtmf(DtmfEvent),
	Playback(PlaybackEvent),
	Timeout(TimeoutEvent),
	Gather(GatherEvent),
	Recording(RecordingEvent),
	Speak(SpeakEvent)
}

mod info{
	#![allow(non_snake_case)]
	#[derive(RustcDecodable)]
	pub struct CallEventInfo{
		pub callState: Option<String>,
		pub to: Option<String>,
		pub withholdCallerNumber: Option<bool>,
		pub time: Option<String>,
		pub applicationId: Option<String>,
		pub from: Option<String>,
		pub eventType: String,
		pub withholdCallerName: Option<bool>,
		pub displayName: Option<String>,
		pub callId: String,
		pub callUri: String,
		pub preferredId: Option<String>,
		pub cause: Option<String>,
		pub tag: Option<String>,
		pub bridge: Option<String>,
		pub digit: Option<String>,
		pub status: Option<String>,
		pub reason: Option<String>,
		pub digits: Option<String>,
		pub gatherId: Option<String>,
		pub recordingId: Option<String>,
		pub startTime: Option<String>,
		pub endTime: Option<String>
	}
}

impl CallEvent{
	pub fn parse(client: &Client, data: &str) -> CatapultResult<CallEvent>{
		let info: CallEventInfo = try!(json::decode(data));
		let event_type = match info.eventType.as_ref(){
			"incomingcall" => EventType::Incoming(try!(IncomingEvent::new(&info))),
			"answer" => EventType::Answer(try!(AnswerEvent::new(&info))),
			"hangup" => EventType::Hangup(try!(HangupEvent::new(&info))),
			"dtmf" => EventType::Dtmf(try!(DtmfEvent::new(&info))),
			"playback" => EventType::Playback(try!(PlaybackEvent::new(&info))),
			"timeout" => EventType::Timeout(try!(TimeoutEvent::new(&info))),
			"gather" => EventType::Gather(try!(GatherEvent::new(&info))),
			"recording" => EventType::Recording(try!(RecordingEvent::new(&info))),
			"speak" => EventType::Speak(try!(SpeakEvent::new(&info))),
			event @ _ => return Err(BError::unexpected(&format!("unknown call event: {}", event)))
		};
		Ok(CallEvent{
			client: client.clone(),
			event_type: event_type,
			call_id: info.callId.clone(),
			withhold_caller_number: info.withholdCallerNumber,
			withhold_caller_name: info.withholdCallerName,
			display_name: info.displayName,
			preferred_id: info.preferredId
		})
	}
	
	pub fn get_client(&self) -> Client{
		self.client.clone()
	}
	pub fn get_event_type(&self) -> &EventType{
		&self.event_type
	}
	pub fn get_call_id(&self) -> String{
		self.call_id.clone()
	}
	pub fn get_withhold_caller_number(&self) -> Option<bool>{
		self.withhold_caller_number
	}
	pub fn get_withhold_caller_name(&self) -> Option<bool>{
		self.withhold_caller_name
	}
	pub fn get_display_name(&self) -> Option<String>{
		self.display_name.clone()
	}
	pub fn get_preferred_id(&self) -> Option<String>{
		self.preferred_id.clone()
	}
	pub fn get_call(&self) -> Call{
		Call::from_call_event(self)
	}
	pub fn get_to(&self) -> Option<String>{
		match self.event_type{
			EventType::Incoming(ref data) => Some(data.get_to()),
			EventType::Answer(ref data) => Some(data.get_to()),
			EventType::Hangup(ref data) => Some(data.get_to()),
			EventType::Timeout(ref data) => Some(data.get_to()),
			_ => None
		}
	} 
	pub fn get_from(&self) -> Option<String>{
		match self.event_type{
			EventType::Incoming(ref data) => Some(data.get_from()),
			EventType::Answer(ref data) => Some(data.get_from()),
			EventType::Hangup(ref data) => Some(data.get_from()),
			_ => None
		}
	}
}


