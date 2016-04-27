use call_event::info::CallEventInfo;
use util;
use CatapultResult;

pub struct DtmfEvent{
	digit: String,
	time: String
}
impl DtmfEvent{
	pub fn new(info: &CallEventInfo) -> CatapultResult<DtmfEvent>{
		Ok(DtmfEvent{
			digit: try!(util::expect(info.digit.clone(), "DtmfEvent::digit")),
			time: try!(util::expect(info.time.clone(), "DtmfEvent::time"))
		})
	}
	pub fn get_digit(&self) -> String{
		self.digit.clone()
	}
	pub fn get_time(&self) -> String{
		self.time.clone()
	}
}