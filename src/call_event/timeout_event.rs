use call_event::info::CallEventInfo;
use util;
use CatapultResult;

pub struct TimeoutEvent{
	time: String,
	from: String,
	to: String
}
impl TimeoutEvent{
	pub fn new(info: &CallEventInfo) -> CatapultResult<TimeoutEvent>{
		Ok(TimeoutEvent{
			time: try!(util::expect(info.time.clone(), "TimeoutEvent::time")),
			from: try!(util::expect(info.from.clone(), "TimeoutEvent::from")),
			to: try!(util::expect(info.to.clone(), "TimeoutEvent::to"))
		})
	}
	pub fn get_time(&self) -> String{
		self.time.clone()
	}
	pub fn get_from(&self) -> String{
		self.from.clone()
	}
	pub fn get_to(&self) -> String{
		self.to.clone()
	}
}