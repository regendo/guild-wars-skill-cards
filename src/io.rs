use crate::skill::{GameMode, Profession, Skill};
use image;
use reqwest;
use scraper::{Html, Selector};
use serde_json;
use std::convert::TryFrom;
use std::{fs, thread, time};

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

pub fn build_data_cache(profession: Profession) {
	let path = &format!("cache/data/{}.json", profession);
	if fs::metadata(path).is_ok() {
		// already exists
		return;
	}
	let url = format!(
		"https://wiki.guildwars.com/wiki/List_of_{}_skills",
		profession.to_string().to_lowercase()
	);
	let raw_html = reqwest::blocking::get(&url)
		.expect(&format!("Unable to get response from {}", url))
		.text()
		.expect(&format!("Failed to get response text from {}", url));
	let skills = touch_up_skills(parse_skills(&raw_html));

	fs::write(path, serde_json::to_string(&skills).unwrap())
		.expect(&format!("Couldn't write to file {}", path));
}

pub fn load_skill_cache(profession: Profession) -> Vec<Skill> {
	let path = format!("cache/data/{}.json", profession);
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

pub fn create_directories() {
	let mut dir_builder = fs::DirBuilder::new();
	dir_builder.recursive(true);
	dir_builder
		.create("cache/data")
		.expect("Couldn't create image directory!");
	dir_builder
		.create("cache/images")
		.expect("Couldn't create data directory!");
	dir_builder
		.create("cards")
		.expect("Couldn't create cards directory!");
	dir_builder
		.create("cards/decks")
		.expect("Couldn't create decks directory!");
}

pub fn build_image_cache(skills: &[Skill]) {
	let client = reqwest::blocking::Client::new();
	let delay = time::Duration::from_secs(1);
	for skill in skills {
		if fs::metadata(skill.icon_path()).is_ok() {
			// Already exists
			// Manually instead of filter because a few skills will have the same icon
			continue;
		}
		let url = &format!("https://wiki.guildwars.com{}", skill.icon_url);
		let response = client
			.get(url)
			.send()
			.expect(&format!("Couldn't send request to {}.", url));
		let path = skill.icon_path();
		let data = response.bytes().unwrap();
		if path.ends_with("-Kurzick.jpg") || path.ends_with("-Luxon.jpg") {
			let image = image::load_from_memory(&*data).unwrap();
			let kurzick = image.crop_imm(0, 0, 64, 64);
			let luxon = image.crop_imm(0, 64, 64, 64);
			let path = path
				.trim_end_matches("-Kurzick.jpg")
				.trim_end_matches("-Luxon.jpg");
			kurzick
				.save(format!("{}-Kurzick.jpg", path))
				.expect(&format!("Couldn't write to {}-Kurzick.jpg.", &path));
			luxon
				.save(format!("{}-Luxon.jpg", path))
				.expect(&format!("Couldn't write to {}-Luxon.jpg.", &path));
		} else {
			fs::write(&path, data).expect(&format!("Couldn't write to {}.", &path));
		}
		thread::sleep(delay);
	}
}
