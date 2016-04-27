use call_event::info::CallEventInfo;
use util;
use CatapultResult;

pub struct IncomingEvent{
	from: String,
	to: String,
	application_id: Option<String>,
	time: String,
	tag: Option<String>
}

impl IncomingEvent{
	pub fn new(info: &CallEventInfo) -> CatapultResult<IncomingEvent>{
		Ok(IncomingEvent{
			from: try!(util::expect(info.from.clone(), "IncomingCall::from")),
			to: try!(util::expect(info.to.clone(), "IncomingCall::to")),
			application_id: info.applicationId.clone(),
			time: try!(util::expect(info.time.clone(), "IncomingCall::time")),
			tag: info.tag.clone()
		})
	}
	pub fn get_to(&self) -> String{
		self.to.clone()
	} 
	pub fn get_from(&self) -> String{
		self.from.clone()
	}
	pub fn get_aplication_id(&self) -> Option<String>{
		self.application_id.clone()
	}
	pub fn get_tag(&self) -> Option<String>{
		self.tag.clone()
	}
	pub fn get_time(&self) -> String{
		self.time.clone()
	}
}