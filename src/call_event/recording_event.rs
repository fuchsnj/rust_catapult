use call_event::info::CallEventInfo;
use util;
use CatapultResult;
use error::CatapultError;

#[derive(Clone)]
pub enum Status{
	Complete,
	Error
}

pub struct RecordingEvent{
	status: Status,
	id: String,
	start_time: String,
	end_time: String 
}

impl RecordingEvent{
	pub fn new(info: &CallEventInfo) -> CatapultResult<RecordingEvent>{
		let status_string:String = try!(util::expect(info.status.clone(), "RecordingEvent::status"));
		Ok(RecordingEvent{
			status:  match status_string.as_ref(){
				"complete" => Status::Complete,
				"error" => Status::Error,
				status @ _ => return Err(CatapultError::unexpected(
					&format!("unknown RecordingEvent status: {}", status)
				))
			},
			id: try!(util::expect(info.recordingId.clone(), "RecordingEvent::id")),
			start_time: try!(util::expect(info.startTime.clone(), "RecordingEvent::start_time")),
			end_time: try!(util::expect(info.endTime.clone(), "RecordingEvent::end_time"))
		})
	}
	pub fn get_status(&self) -> Status{
		self.status.clone()
	}
	pub fn get_id(&self) -> String{
		self.id.clone()
	}
	pub fn get_start_time(&self) -> String{
		self.start_time.clone()
	}
	pub fn get_end_time(&self) -> String{
		self.end_time.clone()
	}
}