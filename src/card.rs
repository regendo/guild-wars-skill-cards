use image::{DynamicImage, ImageBuffer, Rgba};
use imageproc::drawing::draw_text_mut;
use raster::{editor, BlendMode, PositionMode, ResizeMode};
use rusttype::{Font, FontCollection, Point, Scale};

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
	let description = "(5...18...21 seconds.) Enemy spells cannot target you. Gain 5 damage reduction for each Assassin enchantment on you. You cannot deal more than 5...21...25 damage with a single skill or attack.";

	let font_scale_title = 25.0;
	let font_scale_description = 13.0;

	let title_x_start = 35;
	let title_y = 275;
	let type_line_x_start = 25;
	let type_line_y = 315;
	let description_x_start = type_line_x_start;
	let description_y = 340;

	draw_text_mut(
		&mut text_image,
		Rgba([0x0_u8, 0x0_u8, 0x0_u8, 0xFF_u8]),
		title_x_start,
		title_y,
		Scale::uniform(font_scale_title),
		&font,
		card_name,
	);
	draw_text_mut(
		&mut text_image,
		Rgba([0x0_u8, 0x0_u8, 0x0_u8, 0xFF_u8]),
		type_line_x_start,
		type_line_y,
		Scale::uniform(font_scale_description),
		&font,
		type_line,
	);

	let description_lines = split_into_lines(
		description,
		&font,
		(300 - description_x_start * 2) as f32,
		Scale::uniform(font_scale_description),
	);

	let line_height = 12;
	for (idx, line) in description_lines.iter().enumerate() {
		draw_text_mut(
			&mut text_image,
			Rgba([0x0_u8, 0x0_u8, 0x0_u8, 0xFF_u8]),
			description_x_start,
			(description_y + idx * line_height) as u32,
			Scale::uniform(font_scale_description),
			&font,
			line,
		);
	}

	text_image.save("Shadow_Form.png").unwrap();
}

fn split_into_lines<'a>(text: &'a str, font: &Font, line_width: f32, scale: Scale) -> Vec<&'a str> {
	let mut lines = vec![];

	let mut subslice_start = 0;
	while subslice_start < text.len() {
		let chars_in_line = font
			.layout(&text[subslice_start..], scale, Point { x: 0.0, y: 0.0 })
			.filter(|g| g.position().x <= line_width)
			.count();
		if subslice_start + chars_in_line == text.len() {
			// All done!
			lines.push(&text[subslice_start..]);
			subslice_start += chars_in_line;
		} else {
			// Find a word or sentence break.
			let chars: Vec<char> = text.chars().collect();
			let mut off = 0;
			while chars[subslice_start + chars_in_line - off] != ' ' {
				off += 1;
				// fails silently and likely messes up things if line has no whitespace
			}
			lines.push(&text[subslice_start..(subslice_start + chars_in_line - off)]);
			// Skip that whitespace
			subslice_start += chars_in_line - off + 1;
		}
	}

	lines
}
