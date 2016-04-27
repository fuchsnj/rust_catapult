use call_event::info::CallEventInfo;
use util;
use CatapultResult;
use error::CatapultError;

#[derive(Clone)]
pub enum Cause{
	Rejected,
	NormalClearing,
	Busy
}

pub struct HangupEvent{
	from: String,
	to: String,
	cause: Cause,
	time: String
}
impl HangupEvent{
	pub fn new(info: &CallEventInfo) -> CatapultResult<HangupEvent>{
		let cause_string = try!(util::expect(info.cause.clone(), "HangupEvent::cause"));
		Ok(HangupEvent{
			from: try!(util::expect(info.from.clone(), "HangupEvent::from")),
			to: try!(util::expect(info.to.clone(), "HangupEvent::to")),
			cause: match cause_string.as_ref(){
				"NORMAL_CLEARING" => Cause::NormalClearing,
				"CALL_REJECTED" => Cause::Rejected,
				"USER_BUSY" => Cause::Busy,
				status @ _ => return Err(CatapultError::unexpected(
					&format!("unknown HangupEvent status: {}", status)
				))
			},
			time: try!(util::expect(info.time.clone(), "HangupEvent::time")),
		})
	}
	pub fn get_to(&self) -> String{
		self.to.clone()
	} 
	pub fn get_from(&self) -> String{
		self.from.clone()
	}
	pub fn get_cause(&self) -> Cause{
		self.cause.clone()
	}
	pub fn get_time(&self) -> String{
		self.time.clone()
	}
}


