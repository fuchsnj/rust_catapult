use call_event::info::CallEventInfo;
use util;
use BResult;
use error::BError;

#[derive(Clone)]
pub enum Reason{
	MaxDigits,
	TerminatingDigit,
	InterDigitTimeout,
	HungUp
}

pub struct GatherEvent{
	digits: String,
	reason: Reason,
	time: String,
	id: String
}
impl GatherEvent{
	pub fn new(info: &CallEventInfo) -> BResult<GatherEvent>{
		let reason_string = try!(util::expect(info.reason.clone(), "GatherEvent::reason"));
		
		Ok(GatherEvent{
			digits: try!(util::expect(info.digits.clone(), "GatherEvent::digits")),
			reason: match reason_string.as_ref(){
				"max-digits" => Reason::MaxDigits,
				"terminating-digit" => Reason::TerminatingDigit,
				"inter-digit-timeout" => Reason::InterDigitTimeout,
				"hung-up" => Reason::HungUp,
				reason @ _ => return Err(BError::unexpected(
					&format!("unknown GatherEvent reason: {}", reason)
				))
			},
			time: try!(util::expect(info.time.clone(), "GatherEvent::time")),
			id: try!(util::expect(info.gatherId.clone(), "GatherEvent::id")),
		})
	}
	pub fn get_digits(&self) -> String{
		self.digits.clone()
	}
	pub fn get_reason(&self) -> Reason{
		self.reason.clone()
	}
	pub fn get_time(&self) -> String{
		self.time.clone()
	}
	pub fn get_id(&self) -> String{
		self.id.clone()
	}
}