use call_event::info::CallEventInfo;
use util;
use BResult;
use error::BError;

#[derive(Clone)]
pub enum Status{
	Started,
	Done
}

pub struct SpeakEvent{
	status: Status,
	tag: Option<String>,
	time: String
}

impl SpeakEvent{
	pub fn new(info: &CallEventInfo) -> BResult<SpeakEvent>{
		let status_string:String = try!(util::expect(info.status.clone(), "SpeakEvent::status"));
		Ok(SpeakEvent{
			status:  match status_string.as_ref(){
				"started" => Status::Started,
				"done" => Status::Done,
				status @ _ => return Err(BError::unexpected(
					&format!("unknown SpeakEvent status: {}", status)
				))
			},
			tag: info.tag.clone(),
			time: try!(util::expect(info.time.clone(), "SpeakEvent::time"))

		})
	}
	pub fn get_status(&self) -> Status{
		self.status.clone()
	}
	pub fn get_tag(&self) -> Option<String>{
		self.tag.clone()
	}
	pub fn get_time(&self) -> String{
		self.time.clone()
	}
}