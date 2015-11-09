use std::io::Read;
use BResult;
use error::BError;
use hyper;
use hyper::header::{Authorization, Basic, ContentType, Headers};
use hyper::client::response::Response;
use rustc_serialize::{Encodable, Decodable, json};
use hyper::Url;
use hyper::client::RequestBuilder;

use std::sync::{Mutex, Arc};
use {util, application};

use environment::Environment;
use domain::Domain;
use call_event::CallEvent;
use application::Application;

#[derive(Clone)]
pub struct Client{
	data: Arc<Mutex<Data>>
}

struct Data{
	user_id: String,
	api_token: String,
	api_secret: String,
	api_version: String,
	environment: Environment
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
			data: Arc::new(Mutex::new(Data{
				user_id: user_id.to_string(),
				api_token: api_token.to_string(),
				api_secret: api_secret.to_string(),
				api_version: "v1".to_string(),
				environment: Environment::Production
			}))
		}
	}
	pub fn create_url(&self, path: &str) -> String{
		let data = self.data.lock().unwrap();
		data.environment.get_base_url() + "/" + &data.api_version + "/" + path 
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
	/* Setters */
	pub fn set_environment(&self, env: Environment){
		let mut data = self.data.lock().unwrap();
		data.environment = env;
	}
	
	/* Getters */
	pub fn get_user_id(&self) -> String{
		let data = self.data.lock().unwrap();
		data.user_id.clone()
	}
	pub fn get_api_token(&self) -> String{
		let data = self.data.lock().unwrap();
		data.api_token.clone()
	}
	pub fn get_api_secret(&self) -> String{
		let data = self.data.lock().unwrap();
		data.api_secret.clone()
	}
	
	/* Object Helpers */
	
	pub fn parse_call_event(&self, data: &str) -> BResult<CallEvent>{
		CallEvent::parse(self, data)
	}
	// Application
	pub fn build_application(&self, name: &str, call_url: &str, msg_url: &str) -> application::ApplicationBuilder{
		Application::build(self, name, call_url, msg_url)
	}
	// Domain
	pub fn create_domain(&self, name: &str) -> BResult<Domain>{
		Domain::create(self, name)
	}
	pub fn get_domain(&self, id: &str) -> Domain{
		Domain::get(self, id)
	}
	pub fn get_domain_by_name(&self, name: &str) -> BResult<Option<Domain>>{
		Domain::get_by_name(self, name)
	}
	pub fn list_domains(&self) -> BResult<Vec<Domain>>{
		Domain::list(self)
	}
}