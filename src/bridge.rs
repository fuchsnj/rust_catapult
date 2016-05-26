use client::{EmptyResponse, JsonResponse, Client};
use CatapultResult;
use lazy::Lazy;
use lazy::Lazy::*;
use std::sync::{Arc, Mutex};
use util;
use call::Call;
use self::info::BridgeInfo;

pub struct Bridge{
	id: String,
	client: Client,
	data: Arc<Mutex<Data>>
}

struct Data{
	state: Lazy<String>,
	bridge_audio: Lazy<bool>,
	created_time: Lazy<String>,
	activated_time: Lazy<Option<String>>,
	completed_time: Lazy<Option<String>>
}
impl Data{
	fn from_info(info: &BridgeInfo) -> CatapultResult<Data>{
		Ok(Data{
			state: Available(info.state.clone()),
			bridge_audio: Available(info.bridgeAudio),
			created_time: Available(info.createdTime.clone()),
			activated_time: Available(info.activatedTime.clone()),
			completed_time: Available(info.completedTime.clone())
		})
	}
}

mod info{
	#![allow(non_snake_case)]
	#[derive(RustcDecodable)]
	pub struct BridgeInfo{
		pub id: String,
		pub state: String,
		pub bridgeAudio: bool,
		pub calls: String,
		pub createdTime: String,
		pub activatedTime: Option<String>,
		pub completedTime: Option<String>
	}
}

impl Bridge{
	pub fn load(&self) -> CatapultResult<()>{
		let path = "users/".to_string() + &self.client.get_user_id() + "/bridges/" + &self.id;
		let res:JsonResponse<BridgeInfo> = try!(self.client.raw_get_request(&path, (), ()));
		let mut data = self.data.lock().unwrap();
		*data = try!(Data::from_info(&res.body));
		Ok(())
	}
	pub fn get_by_id(client: &Client, id: &str) -> Bridge{
		Bridge{
			id: id.to_string(),
			client: client.clone(),
			data: Arc::new(Mutex::new(Data{
				state: NotLoaded,
				bridge_audio: NotLoaded,
				created_time: NotLoaded,
				activated_time: NotLoaded,
				completed_time: NotLoaded
			}))
		}
	}
	pub fn create(client: &Client, bridge_audio: bool, call_ids: &Vec<String>) -> CatapultResult<Bridge>{
		let path = "users/".to_string() + &client.get_user_id() + "/bridges";
		
		let json = json!({
			"bridgeAudio" => (bridge_audio),
			"callIds" => (call_ids)
		});
		
		let res:EmptyResponse = try!(client.raw_post_request(&path, (), &json));
		let id = try!(util::get_id_from_location_header(&res.headers));
		Ok(Bridge{
			id: id,
			client: client.clone(),
			data: Arc::new(Mutex::new(Data{
				state: NotLoaded,
				bridge_audio: Available(bridge_audio),
				created_time: NotLoaded,
				activated_time: NotLoaded,
				completed_time: NotLoaded
			}))
		})
	}
	
	/* Actions */
	pub fn update(&self, bridge_audio: bool, call_ids: &Vec<String>) -> CatapultResult<()>{
		let path = "users/".to_string() + &self.client.get_user_id() + "/bridges/" + &self.id;
		let json = json!({
			"bridgeAudio" => (bridge_audio),
			"callIds" => (call_ids)
		});
		let _:EmptyResponse = try!(self.client.raw_post_request(&path, (), &json));
		Ok(())
	}
	
	pub fn remove_all_calls(&self) -> CatapultResult<()>{
		let path = "users/".to_string() + &self.client.get_user_id() + "/bridges/" + &self.id;
		let json = json!({
			"callIds" => (Vec::<String>::new())
		});
		let _:EmptyResponse = try!(self.client.raw_post_request(&path, (), &json));
		Ok(())
	}
	pub fn enable_audio(&self, enable: bool) -> CatapultResult<()>{
		let path = "users/".to_string() + &self.client.get_user_id() + "/bridges/" + &self.id;
		let json = json!({
			"bridgeAudio" => (enable)
		});
		let _:EmptyResponse = try!(self.client.raw_post_request(&path, (), &json));
		Ok(())
	}
	pub fn play_audio_file(&self, url: &str, loop_audio: bool) -> CatapultResult<()>{
		let path = "users/".to_string() + &self.client.get_user_id() + "/bridges/" + &self.id + "/audio";
		let json = json!({
			"fileUrl" => (url),
			"loopEnabled" => (loop_audio)
		});
		let _:EmptyResponse = try!(self.client.raw_post_request(&path, (), &json));
		Ok(())
	}
	
	/* Getters */
	pub fn get_id(&self) -> String{
		self.id.clone()
	}
	pub fn get_client(&self) -> Client{
		self.client.clone()
	}
	pub fn get_calls(&self) -> CatapultResult<Vec<Call>>{
		Call::get_calls_from_bridge(self)
	}
	
	pub fn get_state(&self) -> CatapultResult<String>{
		lazy_load!(self, state)
	}
	pub fn get_bridge_audio(&self) -> CatapultResult<bool>{
		lazy_load!(self, bridge_audio)
	}
	pub fn get_created_time(&self) -> CatapultResult<String>{
		lazy_load!(self, created_time)
	}
	pub fn get_activated_time(&self) -> CatapultResult<Option<String>>{
		lazy_load!(self, activated_time)
	}
	pub fn get_completed_time(&self) -> CatapultResult<Option<String>>{
		lazy_load!(self, completed_time)
	}
}