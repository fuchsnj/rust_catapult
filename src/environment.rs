pub enum Environment{
	Production,
	Stage,
	Custom(String)
}
impl Environment{
	pub fn get_base_url(&self) -> String{
		match *self{
			Environment::Production => "https://api.catapult.inetwork.com".to_owned(),
			Environment::Stage => "https://api.stage.catapult.inetwork.com".to_owned(),
			Environment::Custom(ref url) => url.clone()
		}
	}
}