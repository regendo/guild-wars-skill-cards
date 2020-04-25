use crate::skill;
use image::{ImageBuffer, Rgba};
use imageproc::drawing::draw_text_mut;
use raster::{editor, BlendMode, PositionMode, ResizeMode};
use rusttype::{Font, FontCollection, Point, Scale};

pub fn generate_card(skill: &skill::Skill) {
	let background = gen_background(skill);

	let card = add_skill_image(&background, skill);
	let card = add_textboxes(&card);
	let card = add_profession_icon(&card, skill.profession);

	let mut writable_card =
		ImageBuffer::from_raw(card.width as u32, card.height as u32, card.bytes).unwrap();

	// TODO Draw energy cost, cast time, etc.

	let font = load_font();
	draw_title(&mut writable_card, &*skill.name, &font);
	draw_type_line(&mut writable_card, &*skill.type_line(), &font);
	draw_description(&mut writable_card, &*skill.description, &font);

	writable_card
		.save(format!("cards/{}.png", skill.name))
		.unwrap();
}

fn add_profession_icon(
	background: &raster::Image,
	_profession: skill::Profession,
) -> raster::Image {
	// TODO Cache profession icon in memory, we'll use it a lot
	let path = format!(
		"assets/icons/{}-tango-icon-200.png",
		if _profession == skill::Profession::Common {
			"Any".to_owned()
		} else {
			_profession.to_string()
		}
	);
	let mut profession_icon = raster::open(&path).unwrap();
	editor::resize(&mut profession_icon, 110, 110, ResizeMode::Exact).unwrap();

	editor::blend(
		background,
		&profession_icon,
		BlendMode::Normal,
		0.2,
		PositionMode::BottomCenter,
		0,
		-9,
	)
	.unwrap()
}

fn add_textboxes(card: &raster::Image) -> raster::Image {
	// TODO cache textbox image in memory, we'll use it a lot
	let textboxes = raster::open("assets/card_frames/Textboxes.png").unwrap();

	editor::blend(
		card,
		&textboxes,
		BlendMode::Normal,
		1.0,
		PositionMode::TopLeft,
		0,
		0,
	)
	.unwrap()
}

fn draw_title(image: &mut ImageBuffer<Rgba<u8>, Vec<u8>>, text: &str, font: &Font) {
	let x_off = 35;
	let mut y_off: f32 = 275.0;
	let mut scale = 25.0;
	let line_width = 300 - 2 * x_off;
	while !fits_into_line(text, font, Scale::uniform(scale), line_width) {
		scale -= 0.5;
		y_off += 0.25;
	}

	draw_text_mut(
		image,
		Rgba([0x0_u8, 0x0_u8, 0x0_u8, 0xFF_u8]),
		x_off as u32,
		y_off.trunc() as u32,
		Scale::uniform(scale),
		font,
		text,
	);
}

fn draw_type_line(image: &mut ImageBuffer<Rgba<u8>, Vec<u8>>, text: &str, font: &Font) {
	let x_off = 25;
	let padding_right = 10;
	let mut y_off: f32 = 315.0;
	let mut scale = 13.0;
	let line_width = 300 - 2 * x_off - padding_right;
	while !fits_into_line(text, font, Scale::uniform(scale), line_width) {
		scale -= 0.5;
		y_off += 0.25;
	}

	draw_text_mut(
		image,
		Rgba([0x0_u8, 0x0_u8, 0x0_u8, 0xFF_u8]),
		x_off as u32,
		y_off.trunc() as u32,
		Scale::uniform(scale),
		font,
		text,
	);
}

fn fits_into_line(text: &str, font: &Font, scale: Scale, line_width: i32) -> bool {
	font
		.layout(text, scale, Point { x: 0.0, y: 0.0 })
		.last()
		.unwrap()
		.pixel_bounding_box()
		.unwrap()
		.max
		.x <= line_width
}

fn draw_description(image: &mut ImageBuffer<Rgba<u8>, Vec<u8>>, text: &str, font: &Font) {
	let description_x_start = 25;
	let font_scale_description = 13.0;
	let description_y = 340;
	let padding_right = 10;

	let description_lines = split_into_lines(
		text,
		font,
		300 - description_x_start * 2 - padding_right,
		Scale::uniform(font_scale_description),
	);

	let line_height = match description_lines.len() {
		1..=4 => 15,
		5 => 14,
		6 | _ => 13,
	};
	for (idx, line) in description_lines.iter().enumerate() {
		draw_text_mut(
			image,
			Rgba([0x0_u8, 0x0_u8, 0x0_u8, 0xFF_u8]),
			description_x_start as u32,
			(description_y + idx * line_height) as u32,
			Scale::uniform(font_scale_description),
			font,
			line,
		);
	}
}

fn split_into_lines<'a>(text: &'a str, font: &Font, line_width: i32, scale: Scale) -> Vec<&'a str> {
	let mut lines = vec![];

	let mut subslice_start = 0;
	while subslice_start < text.len() {
		let chars_in_line = font
			.layout(&text[subslice_start..], scale, Point { x: 0.0, y: 0.0 })
			.filter(|g| {
				if let Some(rect) = g.pixel_bounding_box() {
					// Does the rightmost glyph pixel still fit into the line?
					rect.max.x <= line_width
				} else {
					// Whitespace doesn't have a bounding box.
					// This is the theoretical leftmost pixel but for whitespace it doesn't matter.
					g.position().x <= line_width as f32
				}
			})
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

fn load_font() -> Font<'static> {
	let font_data = include_bytes!("../Roboto-Regular.ttf") as &'static [u8];
	let font: Font<'static> = FontCollection::from_bytes(font_data)
		.unwrap()
		.into_font()
		.unwrap();

	font
}

fn gen_background(skill: &skill::Skill) -> raster::Image {
	let black = [0x0_u8, 0x0_u8, 0x0_u8, 0xFF_u8];
	let elite = [0xFC_u8, 0xDF_u8, 0x02_u8, 0xFF_u8];
	let color = if skill.is_elite { elite } else { black };

	raster::Image {
		width: 6 * 50,
		height: 6 * 72,
		bytes: color.repeat(6 * 6 * 50 * 72),
	}
}

fn add_skill_image(background: &raster::Image, skill: &skill::Skill) -> raster::Image {
	// no need to cache this one, only few skills re-use icons
	let mut skill_image = raster::open(&skill.icon_path()).unwrap();
	editor::resize(&mut skill_image, 300, 300, ResizeMode::Exact).unwrap();

	editor::blend(
		&background,
		&skill_image,
		BlendMode::Normal,
		1.0,
		PositionMode::TopLeft,
		0,
		0,
	)
	.unwrap()
}
