use regex::{Captures, Regex};
use scraper::{element_ref::ElementRef, Selector};
use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use std::{fmt, iter};
use string_builder;

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

	pub fn determine_resources<'a, I>(
		cols: &mut I,
		profession: Profession,
		skill_name: &str,
	) -> Vec<Resource>
	where
		I: iter::Iterator<Item = ElementRef<'a>>,
	{
		let mut res = vec![];

		let specific_col = cols.next();
		match profession {
			Profession::Necromancer | Profession::Ritualist => {
				if let Some(cost) = specific_col.map(sacrifice_value).unwrap() {
					res.push(Resource::Sacrifice(cost));
				}
			}
			Profession::Warrior | Profession::Paragon | Profession::Dervish | Profession::Common => {
				// Norn and Deldrimor Common skills have adrenaline costs
				if let Some(cost) = specific_col.map(adrenaline_value).unwrap() {
					res.push(Resource::Adrenaline(cost));
				}
			}
			Profession::Elementalist => {
				if skill_name == "Over the Limit" {
					if let Some(drain) = specific_col.map(upkeep_value).unwrap() {
						res.push(Resource::Upkeep(drain));
					}
				} else {
					if let Some(cost) = specific_col.map(overcast_value).unwrap() {
						res.push(Resource::Overcast(cost));
					}
				}
			}
			Profession::Monk | Profession::Assassin => {
				if let Some(drain) = specific_col.map(upkeep_value).unwrap() {
					res.push(Resource::Upkeep(drain));
				}
			}
			Profession::Mesmer | Profession::Ranger => (),
		}

		if let Some(cost) = cols.next().map(numerical_row_value).unwrap() {
			res.push(Resource::Energy(cost));
		}
		if let Some(time) = cols.next().map(cast_time_value).unwrap() {
			res.push(Resource::Cast(time));
		}
		if let Some(time) = cols.next().map(numerical_row_value).unwrap() {
			res.push(Resource::Recharge(time));
		}

		res
	}
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Resource {
	Recharge(u8),
	Cast(f32),
	Energy(u8),
	Adrenaline(u8),
	Overcast(u8),
	Upkeep(i8),
	Sacrifice(u8),
}

impl Resource {
	pub fn text_value(&self) -> String {
		match self {
			Resource::Cast(time) => {
				// TODO
				time.to_string()
			}
			Resource::Upkeep(_) => "-1".to_owned(),
			Resource::Recharge(value)
			| Resource::Energy(value)
			| Resource::Adrenaline(value)
			| Resource::Overcast(value) => value.to_string(),
			Resource::Sacrifice(value) => format!("{}%", value),
		}
	}

	pub fn icon_path(&self) -> String {
		let name = match self {
			Resource::Adrenaline(_) => "adrenaline",
			Resource::Energy(_) => "energy",
			Resource::Cast(_) => "activation-darker",
			Resource::Recharge(_) => "recharge-darker",
			Resource::Sacrifice(_) => "sacrifice",
			Resource::Upkeep(_) => "upkeep",
			Resource::Overcast(_) => "overcast",
		};
		format!("assets/icons/Tango-{}.png", name)
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Skill {
	pub icon_url: String,
	pub name: String,
	pub profession: Profession,
	pub attribute: Option<String>,
	skill_type: String,
	pub description: String,
	pub resources: Vec<Resource>,
	is_quest_reward: bool,
	campaign: String,
	pub split_by_game_mode: Option<GameMode>,
	is_pve_only: bool,
	pub is_elite: bool,
}

impl Skill {
	pub fn type_line(&self) -> String {
		let mut line = string_builder::Builder::default();
		if self.is_elite {
			line.append("Elite ");
		}
		if self.profession != Profession::Common {
			line.append(self.profession.to_string());
			line.append(" ");
		}
		if self.is_pve_only {
			line.append("PvE ");
		}
		line.append(&*self.skill_type);
		if let Some(attribute) = &self.attribute {
			line.append(" (");
			if attribute.ends_with(" rank") {
				line.append(attribute.trim_end_matches(" rank"));
			} else {
				line.append(&**attribute);
			}
			line.append(")")
		}

		line.string().unwrap()
	}

	pub fn is_pvp_variant(&self) -> bool {
		match self.split_by_game_mode {
			Some(GameMode::PvP) => true,
			Some(GameMode::Codex) => true,
			_ => false,
		}
	}

	pub fn icon_path(&self) -> String {
		// I don't think we need to treat PvE/PvP split skills any differently here.
		let allegiance = match &self.attribute {
			Some(k) if k.starts_with("Kurzick") => "-Kurzick",
			Some(l) if l.starts_with("Luxon") => "-Luxon",
			_ => "",
		};
		format!("cache/images/{}{}.jpg", self.name, allegiance)
	}

	pub fn card_path(&self) -> String {
		// I don't think we need to treat PvE/PvP split skills any differently here.
		let allegiance = match &self.attribute {
			Some(k) if k.starts_with("Kurzick") => "-Kurzick",
			Some(l) if l.starts_with("Luxon") => "-Luxon",
			_ => "",
		};
		format!("cards/{}{}.jpg", self.name, allegiance)
	}
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
		}
		if name.ends_with(" (Codex)") {
			split_by_game_mode = Some(GameMode::Codex);
			name = name.trim_end_matches(" (Codex)").to_owned();
		}

		let full_description: String = cols.next().map(innerText).unwrap();
		let full_description = full_description.replace(" [sic]", "");
		let mut split_description = full_description.splitn(2, ". ");
		let mut skill_type = split_description.next().unwrap().to_string();
		let mut is_elite = false;
		if skill_type.starts_with("Elite ") {
			is_elite = true;
			skill_type = skill_type.trim_start_matches("Elite ").to_owned();
		}
		let description = split_description.next().unwrap().to_string();

		let resources = helpers::determine_resources(&mut cols.by_ref().take(4), profession, &*name);

		let is_quest_reward = !cols.next().unwrap().inner_html().is_empty();
		let attribute: Option<String> = cols.next().map(attribute_value).unwrap();
		let campaign: String = cols.next().map(innerText).unwrap();

		let is_pve_only = (attribute.is_some() && attribute.clone().unwrap().ends_with(" rank"))
			|| [
				"Signet of Capture",
				"\"Together as one!\"",
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
			resources,
			attribute,
			campaign,
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

fn parse_described_float(capture: Captures) -> f32 {
	let dividend = capture // upper
		.get(1)
		.unwrap()
		.as_str()
		.trim()
		.parse::<f32>()
		.unwrap();
	let divisor = capture // lower
		.get(2)
		.unwrap()
		.as_str()
		.trim()
		.parse::<f32>()
		.unwrap();
	dividend / divisor
}

fn cast_time_value(el: ElementRef) -> Option<f32> {
	let select_span = Selector::parse("span").unwrap();
	let text: String = el.select(&select_span).next().map(innerText).unwrap();
	let division_pattern = Regex::new("([0-9]+)/([0-9]+)").unwrap();
	let cast_time = if division_pattern.is_match(&*text) {
		parse_described_float(division_pattern.captures_iter(&text).next().unwrap())
	} else {
		text.parse::<f32>().unwrap()
	};

	match cast_time {
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

#[derive(Copy, Clone, Debug, Deserialize, Serialize, Eq, PartialEq)]
pub enum GameMode {
	PvE,
	PvP,
	Codex,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
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
