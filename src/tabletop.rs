use crate::skill;
use raster::{editor, BlendMode, PositionMode};

pub fn create_tabletop_simulator_decks(skills: &[skill::Skill]) {
	// Tabletop Simulator wants our cards in batches 10 cards wide, 7 cards high.
	// Except the bottom right card is a placeholder.
	// TODO create placeholder card, card background
	let mut skills = skills.iter().peekable();
	let mut batch_num = 1;
	let base = raster::Image {
		width: 300 * 10,
		height: 432 * 7,
		bytes: [0xFF_u8, 0xFF_u8, 0xFF_u8, 0xFF_u8].repeat(300 * 10 * 432 * 7),
	};

	while skills.peek().is_some() {
		let mut deck = base.clone();
		let batch = skills.by_ref().take(69);
		for (idx, skill) in batch.enumerate() {
			let card = raster::open(&skill.card_path()).unwrap();
			let offset_x = idx % 10 * 300;
			let offset_y = idx / 10 * 432;

			deck = editor::blend(
				&deck,
				&card,
				BlendMode::Normal,
				1.0,
				PositionMode::TopLeft,
				offset_x as i32,
				offset_y as i32,
			)
			.unwrap();
		}
		raster::save(&deck, &format!("cards/decks/Deck {}.jpg", batch_num)).unwrap();
		batch_num += 1;
	}
}
