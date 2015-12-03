use std::io::Read;
use BResult;
use error::BError;
use hyper;
use hyper::header::{Authorization, Basic, ContentType, Headers};
use hyper::client::response::Response;
use rustc_serialize::{Decodable, json};
use rustc_serialize::json::Json;
use hyper::Url;
use hyper::client::RequestBuilder;
use std::sync::{Mutex, Arc};
use {util, application, message};
use environment::Environment;
use domain::Domain;
use call_event::CallEvent;
use application::Application;
use message::Message;
use message_event::MessageEvent;
use media::{Media, ToBytes};

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

pub struct ByteResponse{
	pub headers: Headers,
	pub body: Vec<u8>
}
impl ApiResponse<ByteResponse> for ByteResponse{
	fn new(res: &mut Response) -> BResult<ByteResponse>{
		let mut data = vec!();
		try!(res.read_to_end(&mut data));
		Ok(ByteResponse{
			headers: res.headers.clone(),
			body: data
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

pub trait ToBody{
	fn to_body(self) -> Vec<u8>;
}
impl ToBody for Vec<u8>{
	fn to_body(self) -> Vec<u8>{
		self
	}
}
impl ToBody for String{
	fn to_body(self) -> Vec<u8>{
		self.into_bytes()
	}
}

impl<'a> ToBody for &'a Json{
	fn to_body(self) -> Vec<u8>{
		self.to_string().to_body()
	}
}
impl<'a> ToBody for (){
	fn to_body(self) -> Vec<u8>{
		vec!()
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
	pub fn raw_put_request<Input, Params, Output>(&self, path: &str, params: Params, body: Input) -> BResult<Output>
	where Input: ToBody, Params: json::ToJson, Output: ApiResponse<Output>{
		self.raw_request(path, params, body, |client, url|{
			client.put(url)
		})
	}
	pub fn raw_post_request<Input, Params, Output>(&self, path: &str, params: Params, body: Input) -> BResult<Output>
	where Input: ToBody, Params: json::ToJson, Output: ApiResponse<Output>{
		self.raw_request(path, params, body, |client, url|{
			client.post(url)
		})
	}
	
	pub fn raw_get_request<Input, Params, Output>(&self, path: &str, params: Params, body: Input) -> BResult<Output>
	where Input: ToBody, Params: json::ToJson, Output: ApiResponse<Output>{
		self.raw_request(path, params, body, |client, url|{
			client.get(url)
		})
	}
	
	pub fn raw_head_request<Input, Params, Output>(&self, path: &str, params: Params, body: Input) -> BResult<Output>
	where Input: ToBody, Params: json::ToJson, Output: ApiResponse<Output>{
		self.raw_request(path, params, body, |client, url|{
			client.head(url)
		})
	}
	
	fn raw_request<Input, Params, Output, Type>(&self, path: &str, params: Params, body: Input, req_type: Type) -> BResult<Output>
	where 
	 	Input: ToBody,
		Output: ApiResponse<Output>,
		Params: json::ToJson,
		Type: FnOnce(& hyper::Client, Url) -> RequestBuilder
	{
		let mut url = try!(Url::parse(&self.create_url(path)));
		util::set_query_params_from_json(&mut url, &params.to_json());
		
		let client = hyper::Client::new();
		let vec_body: Vec<u8> = body.to_body();
		let req = 
			req_type(&client, url)
			.header(Authorization(Basic{
				username: self.get_api_token(),
				password: Some(self.get_api_secret())
			}))
			.header(ContentType::json())
			.body(&vec_body as &[u8]);
		
		let mut res = try!(req.send());
		
		let status = res.status_raw().0;
		if status >= 200 && status < 400{
			let output = Output::new(&mut res);
			output
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
	pub fn get_application(&self, id: &str) -> Application{
		Application::get(self, id)
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
	//Media
	pub fn create_media<T>(&self, filename: &str, data: T) -> BResult<Media>
	where T: ToBytes{
		Media::create(self, filename, data)
	}
	pub fn get_media(&self, filename: &str) -> Media{
		Media::get(self, filename)
	}
	// Message
	pub fn build_message(&self, from: &str, to: &str, text: &str) -> message::MessageBuilder{
		Message::build(self, from, to, text)
	}
	pub fn get_message(&self, id: &str) -> Message{
		Message::get(self, id)
	}
	
	// MessageEvent
	pub fn parse_message_event(&self, event: &str) -> BResult<MessageEvent>{
		MessageEvent::parse(self, event)
	}
}