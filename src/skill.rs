use regex::Regex;
use scraper::{element_ref::ElementRef, Selector};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::fmt;

mod helpers {
	use super::*;

	pub fn determine_profession(row: ElementRef) -> Profession {
		let style = row.value().attr("style").unwrap();
		let re = Regex::new(r"background: (#[0-9A-F]{6})").unwrap();
		let color = re
			.captures_iter(style)
			.next()
			.unwrap()
			.get(1)
			.unwrap()
			.as_str();
		Profession::from_table_background_color(color).expect("Unknown profession!")
	}

	pub fn determine_icon_url(col: ElementRef) -> String {
		let icon_selection = Selector::parse("a img").unwrap();
		col.select(&icon_selection)
			.next()
			.unwrap()
			.value()
			.attr("src")
			.unwrap()
			.to_string()
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
	icon_url: String,
	name: String,
	profession: Profession,
	attribute: Option<String>,
	skill_type: String,
	description: String,
	cost_energy: Option<u8>,
	cost_adrenaline: Option<u8>,
	cost_overcast: Option<u8>,
	cost_upkeep: Option<i8>,
	cost_sacrifice: Option<u8>,
	cast_time: Option<f32>,
	recharge_time: Option<u8>,
	is_quest_reward: bool,
	campaign: String,
	split_by_game_mode: Option<GameMode>,
	is_pve_only: bool,
	is_elite: bool,
}

impl TryFrom<ElementRef<'_>> for Skill {
	type Error = &'static str;

	fn try_from(row: ElementRef) -> Result<Self, Self::Error> {
		if row.value().name() != "tr" {
			return Err("Not a table row!");
		}
		let select_cols = Selector::parse("th, td").unwrap();
		if row.select(&select_cols).count() != 10 {
			return Err("Wrong number of cols!");
		}
		let mut cols = row.select(&select_cols);

		let profession = helpers::determine_profession(row);
		let icon_url = helpers::determine_icon_url(cols.next().unwrap());

		let mut name: String = cols.next().map(innerText).unwrap();
		let mut split_by_game_mode = None;
		if name.ends_with(" (PvP)") {
			split_by_game_mode = Some(GameMode::PvP);
			name = name.trim_end_matches(" (PvP)").to_owned();
			// TODO: cannot detect PvE versions of game mode split skills
		}

		let full_description: String = cols.next().map(innerText).unwrap();
		let mut split_description = full_description.splitn(2, ". ");
		let mut skill_type = split_description.next().unwrap().to_string();
		let mut is_elite = false;
		if skill_type.starts_with("Elite ") {
			is_elite = true;
			skill_type = skill_type.trim_start_matches("Elite ").to_owned();
		}
		let description = split_description.next().unwrap().to_string();

		let mut cost_adrenaline = None;
		let mut cost_overcast = None;
		let mut cost_sacrifice = None;
		let mut cost_upkeep = None;
		let specific_col = cols.next();
		match profession {
			Profession::Necromancer | Profession::Ritualist => {
				cost_sacrifice = specific_col.map(sacrifice_value).unwrap();
			}
			Profession::Warrior | Profession::Paragon | Profession::Dervish | Profession::Common => {
				// Norn and Deldrimor Common skills have adrenaline costs
				cost_adrenaline = specific_col.map(adrenaline_value).unwrap();
			}
			Profession::Elementalist => {
				if name == "Over the Limit" {
					cost_upkeep = specific_col.map(upkeep_value).unwrap();
				} else {
					cost_overcast = specific_col.map(overcast_value).unwrap();
				}
			}
			Profession::Monk | Profession::Assassin => {
				cost_upkeep = specific_col.map(upkeep_value).unwrap();
			}
			Profession::Mesmer | Profession::Ranger => (),
		}

		let cost_energy: Option<u8> = cols.next().map(numerical_row_value).unwrap();
		let cast_time: Option<f32> = cols.next().map(cast_time_value).unwrap();
		let recharge_time: Option<u8> = cols.next().map(numerical_row_value).unwrap();
		let is_quest_reward = !cols.next().unwrap().inner_html().is_empty();
		let attribute: Option<String> = cols.next().map(attribute_value).unwrap();
		let campaign: String = cols.next().map(innerText).unwrap();

		let is_pve_only = (attribute.is_some() && attribute.clone().unwrap().ends_with(" rank"))
			|| [
				"Signet of Capture",
				"\"Together as One!\"",
				"Heroic Refrain",
				"Judgment Strike",
				"Over the Limit",
				"Seven Weapons Stance",
				"Shadow Theft",
				"Soul Taker",
				"Time Ward",
				"Vow of Revolution",
				"Weapons of Three Forges",
			]
			.contains(&&*name);

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
			cost_adrenaline,
			cost_overcast,
			cost_upkeep,
			cost_sacrifice,
			profession,
			icon_url,
			is_elite,
			split_by_game_mode,
			is_pve_only,
		})
	}
}

fn attribute_value(el: ElementRef) -> Option<String> {
	let node = el.children().next().unwrap().value();
	if node.is_text() {
		Some(node.as_text().unwrap().trim().to_string())
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
	if text == "morale boost" {
		return None;
	}
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

fn upkeep_value(el: ElementRef) -> Option<i8> {
	match attribute_value(el) {
		Some(val) => Some(val.parse::<i8>().unwrap()),
		None => None,
	}
}

fn adrenaline_value(el: ElementRef) -> Option<u8> {
	match upkeep_value(el) {
		Some(n) if n < 0 => panic!("Unexpected negative value {}", n),
		Some(n) => Some(n as u8),
		None => None,
	}
}

fn overcast_value(el: ElementRef) -> Option<u8> {
	adrenaline_value(el)
}

fn sacrifice_value(el: ElementRef) -> Option<u8> {
	let node = el.children().skip(1).next()?.value();
	if node.is_text() {
		Some(
			node
				.as_text()
				.unwrap()
				.to_string()
				.split('%')
				.next()
				.unwrap()
				.parse::<u8>()
				.unwrap(),
		)
	} else {
		None
	}
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
enum GameMode {
	PvE,
	PvP,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub enum Profession {
	Warrior,
	Ranger,
	Monk,
	Necromancer,
	Mesmer,
	Elementalist,
	Assassin,
	Ritualist,
	Paragon,
	Dervish,
	// no real profession, but we'll allow it to make things easier
	Common,
}
impl Profession {
	fn from_table_background_color(color: &str) -> Option<Self> {
		match color {
			"#FFFFCC" => Some(Self::Warrior),
			"#EEFFCC" => Some(Self::Ranger),
			"#CCEEFF" => Some(Self::Monk),
			"#CCFFCC" => Some(Self::Necromancer),
			"#EEDDFF" => Some(Self::Mesmer),
			"#FFDDDD" => Some(Self::Elementalist),
			"#FFEEFF" => Some(Self::Assassin),
			"#DDFFFF" => Some(Self::Ritualist),
			"#FFEECC" => Some(Self::Paragon),
			"#EEEEFF" => Some(Self::Dervish),
			"#EEEEEE" => Some(Self::Common),
			_ => None,
		}
	}
	pub fn iter() -> impl Iterator<Item = Self> {
		[
			Self::Warrior,
			Self::Ranger,
			Self::Monk,
			Self::Necromancer,
			Self::Mesmer,
			Self::Elementalist,
			Self::Assassin,
			Self::Ritualist,
			Self::Paragon,
			Self::Dervish,
			Self::Common,
		]
		.iter()
		.copied()
	}
}
impl fmt::Display for Profession {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let name = match self {
			Self::Warrior => "Warrior",
			Self::Ranger => "Ranger",
			Self::Monk => "Monk",
			Self::Necromancer => "Necromancer",
			Self::Mesmer => "Mesmer",
			Self::Elementalist => "Elementalist",
			Self::Assassin => "Assassin",
			Self::Ritualist => "Ritualist",
			Self::Paragon => "Paragon",
			Self::Dervish => "Dervish",
			Self::Common => "Common",
		};
		write!(f, "{}", name)
	}
}
