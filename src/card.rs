use raster::{editor, BlendMode, PositionMode, ResizeMode};

pub fn generate_example_card() {
	let black_background = raster::open("design/exports/Background.png").unwrap();

	let elite_background = raster::open("design/exports/Elite Background.png").unwrap();
	let mut profession = raster::open("cache/images/Assassin-tango-icon-200.png").unwrap();
	let textboxes = raster::open("design/exports/Textboxes.png").unwrap();
	let mut skill_icon = raster::open("cache/images/Shadow_Form.jpg").unwrap();

	editor::resize(&mut skill_icon, 300, 300, ResizeMode::Exact);
	editor::resize(&mut profession, 110, 110, ResizeMode::Exact);
	let card = editor::blend(
		&elite_background,
		&skill_icon,
		BlendMode::Normal,
		1.0,
		PositionMode::TopLeft,
		0,
		0,
	)
	.unwrap();

	let card = editor::blend(
		&card,
		&textboxes,
		BlendMode::Normal,
		1.0,
		PositionMode::TopLeft,
		0,
		0,
	)
	.unwrap();

	let card = editor::blend(
		&card,
		&profession,
		BlendMode::Normal,
		0.2,
		PositionMode::BottomCenter,
		0,
		-9,
	)
	.unwrap();

	raster::save(&card, "card.png");
}
