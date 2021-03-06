use crate::skill;
use image::{ImageBuffer, Rgba};
use imageproc::drawing::draw_text_mut;
use raster::{editor, BlendMode, PositionMode, ResizeMode};
use rusttype::{Font, FontCollection, Point, Scale};
use std::{cmp, fs};

pub fn generate_card(skill: &skill::Skill) {
	let path = skill.card_path();
	if fs::metadata(&path).is_ok() {
		// Already exist
		return;
	}

	let background = gen_background(skill);

	let card = add_skill_image(&background, skill);
	let card = add_textboxes(&card);
	let card = add_profession_icon(&card, skill.profession);
	let card = add_resource_icons(&card, &skill.resources);

	let mut writable_card =
		ImageBuffer::from_raw(card.width as u32, card.height as u32, card.bytes).unwrap();

	let font = load_font();
	draw_title(&mut writable_card, &*skill.name, &font);
	draw_type_line(&mut writable_card, &*skill.type_line(), &font);
	draw_description(&mut writable_card, &*skill.description, &font);
	draw_resources(&mut writable_card, &skill.resources, &font);

	writable_card.save(&path).unwrap();
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
	let max_line_width = 300 - 2 * x_off;

	let line_data = fit_line(text, font, scale, max_line_width);
	y_off += line_data.y_off;
	scale = line_data.scale;
	let centered_x = 300 / 2 - line_data.len / 2;

	draw_text_mut(
		image,
		Rgba([0x0_u8, 0x0_u8, 0x0_u8, 0xFF_u8]),
		centered_x as u32,
		y_off.trunc() as u32,
		Scale::uniform(scale),
		font,
		text,
	);
}

fn draw_type_line(image: &mut ImageBuffer<Rgba<u8>, Vec<u8>>, text: &str, font: &Font) {
	let x_off = 27;
	let mut y_off: f32 = 324.0;
	let mut scale = 13.0;
	let max_line_width = 300 - 2 * x_off;

	let line_data = fit_line(text, font, scale, max_line_width);
	y_off += line_data.y_off;
	scale = line_data.scale;

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

struct LineData {
	scale: f32,
	y_off: f32,
	len: i32,
}

fn fit_line(text: &str, font: &Font, scale: f32, line_width: i32) -> LineData {
	let mut scale = scale;
	let mut y_off = 0.0;
	let len = loop {
		let x = calc_line_width(text, font, Scale::uniform(scale));
		if x <= line_width {
			break x;
		}
		scale -= 0.5;
		y_off += 0.25;
	};
	LineData { scale, y_off, len }
}

fn calc_line_width(text: &str, font: &Font, scale: Scale) -> i32 {
	font
		.layout(text, scale, Point { x: 0.0, y: 0.0 })
		.last()
		.unwrap()
		.pixel_bounding_box()
		.unwrap()
		.max
		.x
}

fn draw_description(image: &mut ImageBuffer<Rgba<u8>, Vec<u8>>, text: &str, font: &Font) {
	let x_off = 27;
	let y_off = 343;
	let scale = 13.0;
	let max_line_width = 300 - 2 * x_off;

	let description_lines = split_into_lines(text, font, max_line_width, Scale::uniform(scale));

	let line_height = match description_lines.len() {
		1..=4 => 15,
		5 => 14,
		6 | _ => 12,
	};
	for (idx, line) in description_lines.iter().enumerate() {
		draw_text_mut(
			image,
			Rgba([0x0_u8, 0x0_u8, 0x0_u8, 0xFF_u8]),
			x_off as u32,
			(y_off + idx * line_height) as u32,
			Scale::uniform(scale),
			font,
			line,
		);
	}
}

fn add_resource_icons(card: &raster::Image, resources: &[skill::Resource]) -> raster::Image {
	if resources.is_empty() {
		return card.to_owned();
	}
	let icon_width = 20;
	let text_max_width = 12;
	let padding_inside = 4;
	let total_resource_width = icon_width + text_max_width + padding_inside;
	let padding_right = 4;
	let total_space_needed =
		total_resource_width * resources.len() + (resources.len() - 1) * padding_right;
	let x_start = 300 / 2 - total_space_needed / 2;
	let y_off = 301;

	let mut card = card.to_owned();

	for (idx, res) in resources.iter().enumerate() {
		let x_off =
			x_start + idx * (total_resource_width + padding_right) + text_max_width + padding_inside;
		let icon = raster::open(&res.icon_path()).unwrap();
		card = editor::blend(
			&card,
			&icon,
			BlendMode::Normal,
			1.0,
			PositionMode::TopLeft,
			x_off as i32,
			y_off,
		)
		.unwrap();
	}

	card
}

fn draw_resources(
	image: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
	resources: &[skill::Resource],
	font: &Font,
) {
	if resources.is_empty() {
		return;
	}
	let icon_width = 20;
	let text_assumed_width = 12;
	let padding_inside = 4;
	let total_resource_width = icon_width + text_assumed_width + padding_inside;
	let padding_right = 4;
	let total_space_needed =
		total_resource_width * resources.len() + (resources.len() - 1) * padding_right;
	let x_start = 300 / 2 - total_space_needed / 2;

	for (idx, res) in resources.iter().enumerate() {
		let x_off = x_start + idx * (total_resource_width + padding_right);
		draw_resource_text(image, font, res, x_off as u32, text_assumed_width as u32);
	}
}

fn draw_resource_text(
	image: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
	font: &Font,
	resource: &skill::Resource,
	x_left_start: u32,
	max_width: u32,
) {
	let scale = Scale::uniform(13.0);
	let text = &resource.text_value();
	let width = calc_line_width(text, font, scale);
	let x_off = cmp::max(0, width) as u32;
	let x_pos = x_left_start + max_width - x_off;

	draw_text_mut(
		image,
		Rgba([0x0_u8, 0x0_u8, 0x0_u8, 0xFF_u8]),
		x_pos,
		305,
		scale,
		font,
		text,
	);
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
	let font_data = include_bytes!("../assets/fonts/Roboto-Regular.ttf") as &'static [u8];
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
