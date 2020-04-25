mod card;
mod io;
mod skill;
mod tabletop;
use skill::{Profession, Skill};

fn main() {
	io::create_directories();

	for profession in Profession::iter() {
		io::build_data_cache(profession);
	}

	let skills: Vec<Skill> = Profession::iter()
		.flat_map(|profession| io::load_skill_cache(profession))
		.filter(|s: &Skill| !s.is_pvp_variant())
		.collect();
	io::build_image_cache(&skills);
	for skill in &skills {
		card::generate_card(&skill);
	}
	tabletop::create_tabletop_simulator_decks(&skills);
}
