use {CatapultResult, Endpoint};
use client::JsonResponse;

#[derive(RustcDecodable)]
pub struct AuthToken{
	token: String,
	expires: u64
}

impl AuthToken{
	pub fn create(endpoint: &Endpoint) -> CatapultResult<AuthToken>{
		let client = endpoint.get_client();
		let domain = endpoint.get_domain();
		let path = "users/".to_string() + &client.get_user_id() + "/domains/" + &domain.get_id()
		+ "/endpoints/" + &endpoint.get_id() + "/tokens";
		let res:JsonResponse<AuthToken> = try!(client.raw_post_request(&path, (), ()));
		Ok(res.body)
	} 
	pub fn get_token(&self) -> String{
		self.token.clone()
	}
	pub fn get_expires(&self) -> u64{
		self.expires
	}
}