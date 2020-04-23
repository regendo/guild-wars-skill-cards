use reqwest;
use scraper::{Html, Selector};
use serde_json;
use std::convert::TryFrom;
use std::fs;

mod skill;
use skill::{Profession, Skill};

fn build_skill_cache(profession: Option<skill::Profession>) {
	if let Some(profession) = profession {
		let url = format!(
			"https://wiki.guildwars.com/wiki/List_of_{}_skills",
			profession.to_string().to_lowercase()
		);
		let raw_html = reqwest::blocking::get(&url)
			.expect(&format!("Unable to get response from {}", url))
			.text()
			.expect(&format!("Failed to get response text from {}", url));
		let skills = parse_skills(&raw_html);
		let path = &format!("cache/{}.json", profession);

		fs::write(path, serde_json::to_string(&skills).unwrap())
			.expect(&format!("Couldn't write to file {}", path));
	}
	// TODO else case for non-profession skills
}

#[allow(dead_code)]
fn load_skill_cache(profession: Option<skill::Profession>) -> Vec<Skill> {
	if let Some(profession) = profession {
		let path = format!("cache/{}.json", profession);
		let raw_skills = fs::read_to_string(path).expect("Couldn't read from file!");
		let skills: Vec<Skill> = serde_json::from_str(&raw_skills).unwrap();
		return skills;
	}
	// TODO else case for non-profession skills
	vec![]
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
	// for profession in Profession::iter() {
	// 	build_skill_cache(Some(profession));
	// }
	build_skill_cache(Some(Profession::Elementalist));
	let skills = load_skill_cache(Some(Profession::Elementalist));
	println!("{} Elementalist skills loaded.", skills.len());
}
