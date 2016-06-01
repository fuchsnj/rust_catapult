use hyper::header;
use std::path::Path;
use {CatapultError, CatapultResult};
use rustc_serialize::json::Json;
use hyper::Url;

pub fn get_id_from_location_header(headers: &header::Headers) -> CatapultResult<String>{
	match headers.get::<header::Location>(){
		Some(location) => {
			get_id_from_location_url(&location.0)
		},
		None => Err(CatapultError::unexpected("Location header not found"))
	}
}
pub fn get_next_link_from_headers(headers: &header::Headers) -> CatapultResult<Option<String>>{
	if let Some(raw_header) = headers.get_raw("link"){
		let header = String::from_utf8_lossy(&raw_header[0]);
		for link in header.split(','){
			if link.contains("rel=\"next\""){
				if let (Some(a), Some(b)) = (link.find('<'), link.find('>')){
					return Ok(Some(link[a+1..b].to_owned()))
				}
			}
		}
	}
	Ok(None)
}
pub fn get_id_from_location_url(url: &str) -> CatapultResult<String>{
	let id = try!(
		Path::new(url).file_name()
		.ok_or(CatapultError::unexpected("failed to parse id from Location header"))
	);
	//OsStr -> String
	Ok(try!(
		id.to_str().ok_or(CatapultError::unexpected("failed to parse id from Location header: UTF-8 problem"))
	).to_string())
}

fn get_unquoted_value(json: &Json) -> String{
	if let Some(val) = json.as_string(){
		val.to_string()//grab string without quotes
	}else{
		json.to_string()
	}
}

pub fn set_query_params_from_json(url: &mut Url, json: &Json){
	if let Some(json) = json.as_object(){
		for (key, value) in json.iter(){
			url.query_pairs_mut().append_pair(key, &get_unquoted_value(value));
		}
	}
}
pub fn expect<T>(data: Option<T>, name: &str) -> CatapultResult<T>{
	match data{
		Some(data) => Ok(data),
		None => Err(CatapultError::unexpected(&format!("required field not found: {}", name)))
	}
}

