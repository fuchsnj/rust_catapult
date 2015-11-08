use {Client, BResult, Endpoint};
use client::{JsonResponse, EmptyResponse};
use std::sync::{Mutex, Arc};
use lazy::Lazy;
use util;
use error::BError;

pub struct Domain{
	id: String,
	client: Client,
	data: Arc<Mutex<Data>>
}

#[derive(Debug)]
struct Data{
	name: Lazy<String>
}

#[derive(RustcDecodable)]
struct DomainInfo{
	id: String,
	name: String
}

impl Domain{
	fn load(&self) -> BResult<()>{
		//if id = empty string, this will return all domains
		if self.get_id().len() == 0{
			return Err(BError::bad_input("invalid app id"))
		}
		
		let path = "users/".to_string() + &self.client.get_user_id() + "/domains/" + &self.id;
		let res:JsonResponse<DomainInfo> = try!(self.client.raw_get_request(&path, (), ()));
		let mut data = self.data.lock().unwrap();
		data.name = Lazy::Available(res.body.name);
		Ok(())
	}
	pub fn create(client: &Client, name: &str) -> BResult<Domain>{
		let path = "users/".to_string() + &client.get_user_id() + "/domains";
		let json = json!({
			"name" => (name)
		});
		let res:EmptyResponse = try!(client.raw_post_request(&path, (), json));
		Ok(Domain{
			id: try!(util::get_id_from_location_header(&res.headers)),
			client: client.clone(),
			data: Arc::new(Mutex::new(Data{
				name: Lazy::Available(name.to_string())
			})) 
		})
	}
	pub fn get(client: &Client, id: &str) -> Domain{
		Domain{
			id: id.to_string(),
			client: client.clone(),
			data: Arc::new(Mutex::new(Data{
				name: Lazy::NotLoaded
			})) 
		}
	}
	
	pub fn get_by_name(client: &Client, name: &str) -> BResult<Option<Domain>>{
		let domains = try!(Self::list(client));
		for domain in domains{
			if try!(domain.get_name()) == name{
				return Ok(Some(domain))
			}
		}
		Ok(None)
	}
	pub fn list(client: &Client) -> BResult<Vec<Domain>>{
		let path = "users/".to_string() + &client.get_user_id() + "/domains";
		let res:JsonResponse<Vec<DomainInfo>> = try!(client.raw_get_request(&path, (), ()));
		
		let mut output = vec!();
		for info in res.body{
			output.push(Domain{
				id: info.id,
				client: client.clone(),
				data: Arc::new(Mutex::new(Data{
					name: Lazy::Available(info.name)
				})) 
			});
		}
		Ok(output)
	}
	pub fn get_id(&self) -> String{
		self.id.clone()
	}
	pub fn get_name(&self) -> BResult<String>{
		if !self.data.lock().unwrap().name.available(){
			try!(self.load());
		}
		Ok(try!(self.data.lock().unwrap().name.get()).clone())
	}
	pub fn get_endpoint_by_id(&self, id: &str) -> Endpoint{
		Endpoint::get_by_id(&self.client, id, &self.id)
	}
}