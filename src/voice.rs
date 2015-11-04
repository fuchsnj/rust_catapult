use self::Voice::*;

#[derive(Clone)]
pub enum Voice{
	///English US, Female
	Kate,
	///English US, Female
	Susan,
	///English US, Female
	Julie,
	///English US, Male
	Dave,
	///English US, Male
	Paul,
	///English UK, Female
	Bridget,
	///Spanish, Female
	Esperanza,
	///Spanish, Female
	Violeta,
	///Spanish, Male
	Jorge,
	///French, Female
	Jolie,
	///French, Male
	Bernard,
	///German, Female
	Katrin,
	///German, Male
	Stefan,
	///Italian, Female
	Paola,
	///Italian, Male
	Luca
}

impl Voice{
	pub fn get_name(&self) -> String{
		match *self{
			Kate => "Kate",
			Susan => "Susan",
			Julie => "Julie",
			Dave => "Dave",
			Paul => "Paul",
			Bridget => "Bridget",
			Esperanza => "Esperanza",
			Violeta => "Violeta",
			Jorge => "Jorge",
			Jolie => "Jolie",
			Bernard => "Bernard",
			Katrin => "Katrin",
			Stefan => "Stefan",
			Paola => "Paola",
			Luca => "Luca",
		}.to_string()
	}
}