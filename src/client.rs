use std::io::Read;
use BResult;
use error::BError;
use hyper;
use hyper::header::{Authorization, Basic, ContentType, Headers};
use hyper::client::response::Response;
use rustc_serialize::{Encodable, Decodable, json};
use hyper::Url;
use util;
use hyper::client::RequestBuilder;
use call_event::CallEvent;

#[derive(Clone)]
pub struct Client{
	user_id: String,
	api_token: String,
	api_secret: String,
	base_url: String,
	api_version: String
}

pub trait ApiResponse<T>{
	fn new(response: &mut Response) -> BResult<T>;
}
#[derive(Debug)]
pub struct JsonResponse<T>{
	pub headers: Headers,
	pub body: T
}
impl<T> ApiResponse<JsonResponse<T>> for JsonResponse<T>
where T: Decodable{
	fn new(res: &mut Response) -> BResult<JsonResponse<T>>{
		let mut data = String::new();
		try!(res.read_to_string(&mut data));
		Ok(JsonResponse{
			headers: res.headers.clone(),
			body: try!(json::decode(&data))
		})
	}
}


#[derive(Debug)]
pub struct EmptyResponse{
	pub headers: Headers
}
impl ApiResponse<EmptyResponse> for EmptyResponse{
	fn new(response: &mut Response) -> BResult<EmptyResponse>{
		Ok(EmptyResponse{
			headers: response.headers.clone()
		})
	}
}



impl Client{
	pub fn new(user_id: &str, api_token: &str, api_secret: &str) -> Client{
		Client{
			user_id: user_id.to_string(),
			api_token: api_token.to_string(),
			api_secret: api_secret.to_string(),
			base_url: "https://api.catapult.inetwork.com".to_string(),
			api_version: "v1".to_string()
		}
	}
	pub fn create_url(&self, path: &str) -> String{
		self.base_url.clone() + "/" + &self.api_version + "/" + path 
	}
	
	pub fn raw_post_request<Input, Params, Output>(&self, path: &str, params: Params, body: Input) -> BResult<Output>
	where Input: Encodable, Params: json::ToJson, Output: ApiResponse<Output>{
		self.raw_request(path, params, body, |client, url|{
			client.post(url)
		})
	}
	
	pub fn raw_get_request<Input, Params, Output>(&self, path: &str, params: Params, body: Input) -> BResult<Output>
	where Input: Encodable, Params: json::ToJson, Output: ApiResponse<Output>{
		self.raw_request(path, params, body, |client, url|{
			client.get(url)
		})
	}
	
	fn raw_request<Input, Params, Output, Type>(&self, path: &str, params: Params, body: Input, req_type: Type) -> BResult<Output>
	where 
	 	Input: Encodable,
		Output: ApiResponse<Output>,
		Params: json::ToJson,
		Type: for<'a> FnOnce(&'a hyper::Client, Url) -> RequestBuilder<'a, Url>
	{			
		let mut url = try!(Url::parse(&self.create_url(path)));
		util::set_query_params_from_json(&mut url, &params.to_json());
		
		let client = hyper::Client::new();
		let json_body = try!(json::encode(&body));
		
		let req = 
			req_type(&client, url)
			.header(Authorization(Basic{
				username: self.get_api_token(),
				password: Some(self.get_api_secret())
			}))
			.header(ContentType::json())
			.body(&json_body);
		
		let mut res = try!(req.send());
		
		let status = res.status_raw().0;
		
		if status >= 200 && status < 400{
			Output::new(&mut res)
		}else{
			let mut data = String::new();
			try!(res.read_to_string(&mut data));
			Err(BError::api_error(&data))
		}
	}
	
	pub fn get_user_id(&self) -> String{
		self.user_id.clone()
	}
	pub fn get_api_token(&self) -> String{
		self.api_token.clone()
	}
	pub fn get_api_secret(&self) -> String{
		self.api_secret.clone()
	}
	
	/* Object Helpers */
	pub fn parse_call_event(&self, data: &str) -> BResult<CallEvent>{
		CallEvent::parse(self, data)
	}
}