use hyper::header;
use std::path::Path;
use {BError, BResult};
use rustc_serialize::json::Json;
use hyper::Url;

pub fn get_id_from_location_header(headers: &header::Headers) -> BResult<String>{
	match headers.get::<header::Location>(){
		Some(location) => {
			get_id_from_location_url(&location.0)
		},
		None => Err(BError::unexpected("Location header not found"))
	}
}
pub fn get_id_from_location_url(url: &str) -> BResult<String>{
	let id = try!(
		Path::new(url).file_name()
		.ok_or(BError::unexpected("failed to parse id from Location header"))
	);
	//OsStr -> String
	Ok(try!(
		id.to_str().ok_or(BError::unexpected("failed to parse id from Location header: UTF-8 problem"))
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
		let vec:Vec<(String, String)> = json.iter().map(
			|(x,y)|{
				( x.to_string(), get_unquoted_value(y) )
			}
		).collect();
		url.set_query_from_pairs(vec.iter().map(|&(ref x, ref y)| (&x[..], &y[..])));
	}
}
pub fn expect<T>(data: Option<T>, name: &str) -> BResult<T>{
	match data{
		Some(data) => Ok(data),
		None => Err(BError::unexpected(&format!("required field not found: {}", name)))
	}
}

