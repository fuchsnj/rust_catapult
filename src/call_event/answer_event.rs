use call_event::info::CallEventInfo;
use util;
use BResult;

pub struct AnswerEvent{
	from: String,
	to: String,
	tag: Option<String>,
	time: String
}
impl AnswerEvent{
	pub fn new(info: &CallEventInfo) -> BResult<AnswerEvent>{
		Ok(AnswerEvent{
			from: try!(util::expect(info.from.clone(), "Answer::from")),
			to: try!(util::expect(info.to.clone(), "Answer::to")),
			tag: info.tag.clone(),
			time: try!(util::expect(info.time.clone(), "Answer::time"))
		})
	}
	pub fn get_to(&self) -> String{
		self.to.clone()
	} 
	pub fn get_from(&self) -> String{
		self.from.clone()
	}
	pub fn get_tag(&self) -> Option<String>{
		self.tag.clone()
	}
	pub fn get_time(&self) -> String{
		self.time.clone()
	}
}