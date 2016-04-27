use {Client, CatapultResult};
use client::{JsonResponse};
use self::info::AccountInfo;
use std::sync::{Mutex, Arc};
use lazy::Lazy;
use lazy::Lazy::*;

struct Data{
	balance: Lazy<String>,
	account_type: Lazy<String>
}

pub struct Account{
	client: Client,
	data: Arc<Mutex<Data>>
}
impl Account{
	pub fn load(&self) -> CatapultResult<()>{
		let path = "users/".to_string() + &self.client.get_user_id() + "/account";
		let res:JsonResponse<AccountInfo> = try!(self.client.raw_get_request(&path, (), ()));
		let mut data = self.data.lock().unwrap();
		data.balance = Available(res.body.balance);
		data.account_type = Available(res.body.accountType);
		Ok(())
	}
	pub fn get(client: &Client) -> Account{
		Account{
			client: client.clone(),
			data: Arc::new(Mutex::new(Data{
				balance: NotLoaded,
				account_type: NotLoaded
			}))
		}
	}
	
	/* Getters */
	pub fn get_balance(&self) -> CatapultResult<String>{
		lazy_load!(self, balance)
	}
	
	pub fn get_type(&self) -> CatapultResult<String>{
		lazy_load!(self, account_type)
	}
}

mod info{
	#![allow(non_snake_case)]
	#[derive(RustcDecodable)]
	pub struct AccountInfo{
		pub balance: String,
		pub accountType: String
	}
}