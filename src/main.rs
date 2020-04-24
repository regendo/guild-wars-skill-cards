use reqwest;
use scraper::{Html, Selector};
use serde_json;
use std::convert::TryFrom;
use std::fs;

mod skill;
use skill::{GameMode, Profession, Skill};

fn is_allegiance_rank(skill: &Skill) -> bool {
	match skill.attribute {
		Some(ref attr) if attr.starts_with("Allegiance") => true,
		_ => false,
	}
}

fn touch_up_skills(mut skills: Vec<Skill>) -> Vec<Skill> {
	let pvp_skill_names: Vec<String> = skills
		.iter()
		.filter_map(|s| {
			if s.is_pvp_variant() && s.name != "Charm Animal" {
				// the regular Charm Animal is usable in both PvE and PvP, so it'd
				// be a mistake to mark it PvE-only just because of the Codex variant
				Some(s.name.clone())
			} else {
				None
			}
		})
		.collect();
	skills
		.iter_mut()
		.filter(|s| pvp_skill_names.contains(&s.name) && s.split_by_game_mode.is_none())
		.for_each(|s| s.split_by_game_mode = Some(GameMode::PvE));

	let allegiance: Vec<&mut Skill> = skills
		.iter_mut()
		.filter(|s| is_allegiance_rank(&s))
		.collect();

	let mut luxon_variants = vec![];

	for skill in allegiance {
		let mut luxon_variant = skill.clone();
		luxon_variant.attribute = Some("Luxon rank".to_owned());
		skill.attribute = Some("Kurzick rank".to_owned());
		luxon_variants.push(luxon_variant);
	}

	skills.extend(luxon_variants);
	skills
}

fn build_skill_cache(profession: Profession) {
	let url = format!(
		"https://wiki.guildwars.com/wiki/List_of_{}_skills",
		profession.to_string().to_lowercase()
	);
	let raw_html = reqwest::blocking::get(&url)
		.expect(&format!("Unable to get response from {}", url))
		.text()
		.expect(&format!("Failed to get response text from {}", url));
	let skills = touch_up_skills(parse_skills(&raw_html));
	let path = &format!("cache/{}.json", profession);

	fs::write(path, serde_json::to_string(&skills).unwrap())
		.expect(&format!("Couldn't write to file {}", path));
}

fn load_skill_cache(profession: Profession) -> Vec<Skill> {
	let path = format!("cache/{}.json", profession);
	let raw_skills = fs::read_to_string(path).expect("Couldn't read from file!");
	let skills: Vec<Skill> = serde_json::from_str(&raw_skills).unwrap();
	return skills;
}

fn parse_skills(raw_html: &str) -> Vec<Skill> {
	let page = Html::parse_fragment(&raw_html);
	let select_table_rows = Selector::parse("table.sortable tbody tr[data-name]").unwrap();
	let rows = page.select(&select_table_rows);
	let skills: Vec<Skill> = rows.map(|row| Skill::try_from(row).unwrap()).collect();
	skills
}

fn main() {
	fs::DirBuilder::new()
		.recursive(true)
		.create("cache")
		.expect("Couldn't create cache directory!");
	for profession in Profession::iter() {
		build_skill_cache(profession);
	}
	// let skills: Vec<Skill> = Profession::iter()
	// 	.flat_map(|profession| load_skill_cache(profession))
	// 	.collect();
	// println!("{} skills loaded.", skills.len());
	// for skill in skills.iter().filter(|s| !s.is_pvp_variant()) {
	// 	println!("{}: {}", skill.type_line(), skill.name);
	// }
}
