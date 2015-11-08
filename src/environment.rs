pub enum Environment{
	Production,
	Stage,
	Development
}
impl Environment{
	pub fn get_base_url(&self) -> String{
		match *self{
			Environment::Production => "https://api.catapult.inetwork.com",
			Environment::Stage => "https://api.stage.catapult.inetwork.com",
			Environment::Development => "https://api.dev.catapult.inetwork.com"
		}.to_owned()
	}
}