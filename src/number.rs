use {Client, BResult};
use client::{JsonResponse, EmptyResponse};
use util;
use std::sync::{Mutex, Arc};
use lazy::Lazy;
use lazy::Lazy::*;
use rustc_serialize::json::{Json, ToJson};
use std::collections::BTreeMap;
use self::info::{NumberInfo, AllocatedNumber};

pub enum Search{
	ByCity{
		city: String,
		state: String	
	},
	ByState(String),
	ByZip(String),
	ByAreaCode{
		area_code: String,
		local_number: String,
		in_local_calling_area: bool
	}
}



pub struct Number<'a>{
	id: String,
	client: &'a Client,
	data: Arc<Mutex<Data>>
}

#[derive(Debug)]
struct Data{
	name: Lazy<Option<String>>,
	number: Lazy<String>,
	national_number: Lazy<String>,
	created_time: Lazy<String>,
	city: Lazy<String>,
	state: Lazy<String>,
	price: Lazy<String>,
	number_state: Lazy<String>,
	application_id: Lazy<Option<String>>,
	fallback_number: Lazy<Option<String>>
}

mod info{
	#![allow(non_snake_case)]
	#[derive(RustcDecodable)]
	pub struct NumberInfo{
		pub name: Option<String>,
		pub number: String,
		pub nationalNumber: String,
		pub createdTime: String,
		pub city: String,
		pub state: String,
		pub price: String,
		pub numberState: String,
		pub application: Option<String>,//application URL
		pub fallbackNumber: Option<String>
	}
	
	#[derive(RustcDecodable, Debug)]
	pub struct AllocatedNumber{
		pub number: String,
		pub nationalNumber: String,
		pub price: String,
		pub location: String
	}
}


impl<'a> Number<'a>{
	fn load(&self) -> BResult<()>{
		let path = "users/".to_string() + &self.client.get_user_id() + "/phoneNumbers/" + &self.id;
		let res:JsonResponse<NumberInfo> = try!(self.client.raw_get_request(&path, (), ()));
		let mut data = self.data.lock().unwrap();
		data.name = Available(res.body.name);
		data.number = Available(res.body.number);
		data.national_number = Available(res.body.nationalNumber);
		data.created_time = Available(res.body.createdTime);
		data.city = Available(res.body.city);
		data.state = Available(res.body.state);
		data.price = Available(res.body.price);
		data.number_state = Available(res.body.numberState);
		data.application_id = Available(match res.body.application{
			Some(url) => Some(try!(util::get_id_from_location_url(&url))),
			None => None
		});
		data.fallback_number = Available(res.body.fallbackNumber);
		Ok(())
	}
	pub fn save(&self) -> BResult<()>{
		let path = "users/".to_string() + &self.client.get_user_id() + "/phoneNumbers/" + &self.id;
		let data = self.data.lock().unwrap();
		let mut map = BTreeMap::new();
		if let Some(app_id) = data.application_id.peek(){
			map.insert("applicationId".to_string(), app_id.to_json());
		}
		if let Some(name) = data.name.peek(){
			map.insert("name".to_string(), name.to_json());
		}
		if let Some(fallback_number) = data.fallback_number.peek(){
			map.insert("fallbackNumber".to_string(), fallback_number.to_json());
		}
		let json = Json::Object(map);
		let _:EmptyResponse = try!(self.client.raw_post_request(&path, (), json));
		Ok(())
	}
	pub fn by_id(client: &'a Client, id: &str) -> Number<'a>{
		Number{
			id: id.to_string(),
			client: client,
			data: Arc::new(Mutex::new(Data{
				created_time: NotLoaded,
				fallback_number: NotLoaded,
				name: NotLoaded,
				number: NotLoaded,
				national_number: NotLoaded,
				city: NotLoaded,
				state: NotLoaded,
				price: NotLoaded,
				number_state: NotLoaded,
				application_id: NotLoaded,
			}))
		}
	}

	pub fn search_and_allocate_local(client: &'a Client, quantity: u32, search: Search) -> BResult<Vec<Number<'a>>>{
		let json = match search{
			Search::ByCity{city, state} => json!({
				"city" => (city),
				"state" => (state),
				"quantity" => (quantity)
			}),
			Search::ByState(state) => json!({
				"state" => (state),
				"quantity" => (quantity)
			}),
			Search::ByZip(zip) => json!({
				"zip" => (zip),
				"quantity" => (quantity)
			}),
			Search::ByAreaCode{
				area_code, local_number, in_local_calling_area
			} => json!({
				"areaCode" => (area_code),
				"localNumber" => (local_number),
				"inLocalCallingArea" => (in_local_calling_area),
				"quantity" => (quantity)
			}),
		};
		let res:JsonResponse<Vec<AllocatedNumber>> = try!(client.raw_post_request("availableNumbers/local", json, ()));
		
		let mut output = vec!();
		for number in res.body{
			let id = try!(util::get_id_from_location_url(&number.location));
			output.push(Number{
				id: id.to_string(),
				client: client,
				data: Arc::new(Mutex::new(Data{
					created_time: NotLoaded,
					fallback_number: NotLoaded,
					name: NotLoaded,
					number: Available(number.number),
					national_number: Available(number.nationalNumber),
					city: NotLoaded,
					state: NotLoaded,
					price: Available(number.price),
					number_state: NotLoaded,
					application_id: NotLoaded,
				}))
			});
		}
		Ok(output)
	}
	
	/* Getters */
	pub fn get_id(&self) -> BResult<String>{
		Ok(self.id.clone())
	}
	pub fn get_client(&self) -> Client{
		self.client.clone()
	}
	pub fn get_name(&self) -> BResult<Option<String>>{
		if !self.data.lock().unwrap().name.available(){
			try!(self.load());
		}
		Ok(try!(self.data.lock().unwrap().name.get()).clone())
	}
	pub fn get_number(&self) -> BResult<String>{
		if !self.data.lock().unwrap().number.available(){
			try!(self.load());
		}
		Ok(try!(self.data.lock().unwrap().number.get()).clone())
	}
	pub fn get_national_number(&self) -> BResult<String>{
		if !self.data.lock().unwrap().national_number.available(){
			try!(self.load());
		}
		Ok(try!(self.data.lock().unwrap().national_number.get()).clone())
	}
	pub fn get_created_time(&self) -> BResult<String>{
		if !self.data.lock().unwrap().created_time.available(){
			try!(self.load());
		}
		Ok(try!(self.data.lock().unwrap().created_time.get()).clone())
	}
	pub fn get_city(&self) -> BResult<String>{
		if !self.data.lock().unwrap().city.available(){
			try!(self.load());
		}
		Ok(try!(self.data.lock().unwrap().city.get()).clone())
	}
	pub fn get_state(&self) -> BResult<String>{
		if !self.data.lock().unwrap().state.available(){
			try!(self.load());
		}
		Ok(try!(self.data.lock().unwrap().state.get()).clone())
	}
	pub fn get_price(&self) -> BResult<String>{
		if !self.data.lock().unwrap().price.available(){
			try!(self.load());
		}
		Ok(try!(self.data.lock().unwrap().price.get()).clone())
	}
	pub fn get_number_state(&self) -> BResult<String>{
		if !self.data.lock().unwrap().number_state.available(){
			try!(self.load());
		}
		Ok(try!(self.data.lock().unwrap().number_state.get()).clone())
	}
	pub fn get_application_id(&self) -> BResult<Option<String>>{
		if !self.data.lock().unwrap().application_id.available(){
			try!(self.load());
		}
		Ok(try!(self.data.lock().unwrap().application_id.get()).clone())
	}
	pub fn get_fallback_number(&self) -> BResult<Option<String>>{
		if !self.data.lock().unwrap().fallback_number.available(){
			try!(self.load());
		}
		Ok(try!(self.data.lock().unwrap().fallback_number.get()).clone())
	}
	
	/* Setters */
	pub fn set_application_id(&self, id: Option<&str>){
		self.data.lock().unwrap().application_id = Available(id.map(|a|a.to_string()));
	}
	
}
