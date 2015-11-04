#[derive(Debug)]
pub enum Lazy<T>{
	Available(T),
	NotLoaded
}

pub struct LazyError;

impl<T> Lazy<T>{
	pub fn available(&self) -> bool{
		match *self{
			Lazy::Available(_) => true,
			Lazy::NotLoaded => false
		}
	}
	pub fn peek(&self) -> Option<&T>{
		match *self{
			Lazy::Available(ref data) => Some(data),
			Lazy::NotLoaded => None
		}
	}
	pub fn get(&self) -> Result<&T, LazyError>{
		match *self{
			Lazy::Available(ref data) => Ok(data),
			Lazy::NotLoaded => Err(LazyError)
		}
	}
	pub fn load_if_available(data: Option<T>) -> Lazy<T>{
		match data{
			Some(data) => Lazy::Available(data),
			None => Lazy::NotLoaded
		}
	}
}