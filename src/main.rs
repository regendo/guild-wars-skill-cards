use reqwest;
use scraper::{Html, Selector};
use std::convert::TryFrom;
use std::error::Error;
use std::fs;

mod skill;

fn load_page() -> Result<String, Box<dyn Error>> {
	let url = "https://wiki.guildwars.com/wiki/List_of_paragon_skills";
	Ok(reqwest::blocking::get(url)?.text()?)
}

fn load_cached_page() -> Result<String, std::io::Error> {
	fs::read_to_string("body.html")
}

fn main() {
	// if let Ok(raw_html) = load_page() {
	if let Ok(raw_html) = load_cached_page() {
		let page = Html::parse_fragment(&raw_html);
		let select_table_rows = Selector::parse("table.sortable tbody tr[data-name]").unwrap();
		let raw_skills = page.select(&select_table_rows);
		let skills = raw_skills.map(skill::Skill::try_from);
		for skill in skills {
			println!("{:?}", skill.unwrap());
		}
	}
}
