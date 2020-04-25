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
	// TODO Make profession-specific
	// TODO Cache profession icon
	let mut profession_icon = raster::open("cache/images/Assassin-tango-icon-200.png").unwrap();
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
	// TODO cache textbox image
	let textboxes = raster::open("design/exports/Textboxes.png").unwrap();

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
	// TODO scale font for extra long titles
	let font_scale_title = 25.0;
	let title_x_start = 35;
	let title_y = 275;

	draw_text_mut(
		image,
		Rgba([0x0_u8, 0x0_u8, 0x0_u8, 0xFF_u8]),
		title_x_start,
		title_y,
		Scale::uniform(font_scale_title),
		font,
		text,
	);
}

fn draw_type_line(image: &mut ImageBuffer<Rgba<u8>, Vec<u8>>, text: &str, font: &Font) {
	// TODO scale font for extra long type lines
	let font_scale_description = 13.0;
	let type_line_x_start = 25;
	let type_line_y = 315;

	draw_text_mut(
		image,
		Rgba([0x0_u8, 0x0_u8, 0x0_u8, 0xFF_u8]),
		type_line_x_start,
		type_line_y,
		Scale::uniform(font_scale_description),
		font,
		text,
	);
}

fn draw_description(image: &mut ImageBuffer<Rgba<u8>, Vec<u8>>, text: &str, font: &Font) {
	let description_x_start = 25;
	let font_scale_description = 13.0;
	let description_y = 340;
	let padding_right = 10;

	let description_lines = split_into_lines(
		text,
		font,
		(300 - description_x_start * 2 - padding_right) as f32,
		Scale::uniform(font_scale_description),
	);

	let line_height = 12;
	for (idx, line) in description_lines.iter().enumerate() {
		draw_text_mut(
			image,
			Rgba([0x0_u8, 0x0_u8, 0x0_u8, 0xFF_u8]),
			description_x_start,
			(description_y + idx * line_height) as u32,
			Scale::uniform(font_scale_description),
			font,
			line,
		);
	}
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

fn add_skill_image(background: &raster::Image, _skill: &skill::Skill) -> raster::Image {
	// TODO: use the actual skill's icon
	// no need to cache this one, only few skills re-use icons
	let mut skill_image = if _skill.is_elite {
		raster::open("cache/images/Shadow_Form.jpg").unwrap()
	} else {
		raster::open("cache/images/_Go_for_the_Eyes!_.jpg").unwrap()
	};
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
