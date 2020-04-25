use image::{DynamicImage, ImageBuffer, Rgba};
use imageproc::drawing::draw_text_mut;
use raster::{editor, BlendMode, PositionMode, ResizeMode};
use rusttype::{Font, FontCollection, Scale};

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

	let mut card = editor::blend(
		&card,
		&profession,
		BlendMode::Normal,
		0.2,
		PositionMode::BottomCenter,
		0,
		-9,
	)
	.unwrap();

	let font_data = Vec::from(include_bytes!("../Roboto-Regular.ttf") as &[u8]);
	let font: Font = FontCollection::from_bytes(&font_data)
		.unwrap()
		.into_font()
		.unwrap();

	let card_name = "Shadow Form";
	let mut text_image =
		ImageBuffer::from_raw(card.width as u32, card.height as u32, card.bytes).unwrap();
	let type_line = "Elite Assassin Enchantment Spell (Shadow Arts)";

	let mut layout = font.layout(
		description,
		Scale::uniform(font_scale_description),
		Point {
			x: description_x_start as f32,
			y: description_y as f32,
		},
	);
	let line_len = layout
		.filter(|g| g.position().x <= (300 - description_x_start) as f32)
		.count();
	println!("{}", &description[0..line_len]);

	text_image.save("Shadow_Form.png").unwrap();
}
