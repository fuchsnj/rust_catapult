use call_event::info::CallEventInfo;
use util;
use CatapultResult;
use error::BError;

#[derive(Clone)]
pub enum Status{
	Started,
	Done
}

pub struct PlaybackEvent{
	status: Status,
	tag: Option<String>,
	time: String
}
impl PlaybackEvent{
	pub fn new(info: &CallEventInfo) -> CatapultResult<PlaybackEvent>{
		let status_string:String = try!(util::expect(info.status.clone(), "PlaybackEvent::status"));
		
		Ok(PlaybackEvent{
			status: match status_string.as_ref(){
				"started" => Status::Started,
				"done" => Status::Done,
				status @ _ => return Err(BError::unexpected(
					&format!("unknown PlaybackEvent status: {}", status)
				))
			},
			time: try!(util::expect(info.time.clone(), "PlaybackEvent::time")),
			tag: info.tag.clone()
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