use scraper::{element_ref::ElementRef, Selector};
use std::convert::TryFrom;

#[derive(Debug, Clone)]
pub struct Skill {
	// icon_url: String,
	pub name: String,
	// profession: Option<Profession>,
	// attribute: Option<String>,
	skill_type: String,
	description: String,
	// cost_adrenaline: Option<u8>,
	// cost_energy: Option<u8>,
	// cast_time: Option<u8>,
	// recharge_time: Option<u8>,
	// is_quest_reward: bool,
}

impl TryFrom<ElementRef<'_>> for Skill {
	type Error = &'static str;

	fn try_from(value: ElementRef) -> Result<Self, Self::Error> {
		if value.value().name() != "tr" {
			return Err("Not a table row!");
		}
		let select_cols = Selector::parse("th, td").unwrap();
		if value.select(&select_cols).count() != 10 {
			return Err("Wrong number of cols!");
		}
		let mut cols = value.select(&select_cols);

		let _ = cols.next(); // TODO image
		let name = cols.next().map(innerText).unwrap().into();

		let full_description: String = cols.next().map(innerText).unwrap();
		let mut split_description = full_description.splitn(2, ". ");
		let skill_type = split_description.next().unwrap().to_string();
		let description = split_description.next().unwrap().to_string();

		let _ = cols.next(); // TODO adrenaline
		let _ = cols.next(); // TODO energy
		let _ = cols.next(); // TODO cast time
		let _ = cols.next(); // TODO recharge time
		let _ = cols.next(); // TODO quest reward
		let _ = cols.next(); // TODO attribute
		let _ = cols.next(); // TODO campaign

		Ok(Self {
			name,
			description,
			skill_type,
		})
	}
}

#[allow(non_snake_case)] // This mimics the element.innerText method found in browsers.
fn innerText(el: ElementRef) -> String {
	el.text()
		.map(|t| t.to_string())
		.collect::<String>()
		.trim()
		.to_string()
}

#[derive(Copy, Clone)]
enum Profession {
	Warrior,
	Ranger,
	Monk,
	Mesmer,
	Necromancer,
	Elementalist,
	Assassin,
	Ritualist,
	Dervish,
	Paragon,
}
