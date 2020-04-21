use scraper::{element_ref::ElementRef, Selector};
use std::convert::TryFrom;

#[derive(Debug, Clone)]
pub struct Skill {
	// icon_url: String,
	pub name: String,
	// profession: Option<Profession>,
	attribute: Option<String>,
	skill_type: String,
	description: String,
	// cost_adrenaline: Option<u8>,
	cost_energy: Option<u8>,
	cast_time: Option<f32>,
	recharge_time: Option<u8>,
	is_quest_reward: bool,
	campaign: String,
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

		let _ = cols.next(); // TODO adrenaline, blood sacrifice, perhaps others, or None
		let cost_energy: Option<u8> = cols.next().map(numerical_row_value).unwrap();
		let cast_time: Option<f32> = cols.next().map(cast_time_value).unwrap();
		let recharge_time: Option<u8> = cols.next().map(numerical_row_value).unwrap();
		let is_quest_reward = cols.next().unwrap().inner_html().len() > 0;
		let attribute = cols.next().map(attribute_value).unwrap();
		let campaign: String = cols.next().map(innerText).unwrap();

		Ok(Self {
			name,
			description,
			skill_type,
			is_quest_reward,
			cost_energy,
			cast_time,
			recharge_time,
			attribute,
			campaign,
		})
	}
}

fn attribute_value(el: ElementRef) -> Option<String> {
	let node = el.children().next().unwrap().value();
	if node.is_text() {
		Some(node.as_text().unwrap().to_string())
	} else {
		None
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

fn numerical_row_value(el: ElementRef) -> Option<u8> {
	let select_span = Selector::parse("span").unwrap();
	let text: String = el.select(&select_span).next().map(innerText).unwrap();
	match text.parse::<u8>().unwrap() {
		n if n == 0 => None,
		n => Some(n),
	}
}

fn cast_time_value(el: ElementRef) -> Option<f32> {
	let select_span = Selector::parse("span").unwrap();
	let text: String = el.select(&select_span).next().map(innerText).unwrap();
	match text.parse::<f32>().unwrap() {
		n if !n.is_normal() => None,
		n => Some(n),
	}
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
