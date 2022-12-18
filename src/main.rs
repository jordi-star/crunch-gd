use clap::Arg;

use glob::glob;

use std::{path::{PathBuf}};

mod atlas_gen;
use atlas_gen::packer::{SpritePacker};

// TODO: Replace usize casts to try_into for better error handling. Image sizes might get large enough to overflow!
fn main() {
	let config = clap::App::new("crunch-gd")
		.about("Generates spritesheets & associated AtlasTexture resources for Godot 4. Uses ChevyRay's crunch-rs for sprite packing.")
		.author("jordi")
		.arg(Arg::with_name("input_path")
			.short("i")
			.long("input_path")
			.value_name("(Folder path)")
			.help("Location of all sprites to be packed")
			.takes_value(true))
		.arg(Arg::with_name("width")
			.short("w")
			.long("width")
			.value_name("Spritesheet Width")
			.help("Final Width of Spritesheet")
			.takes_value(true))
		.arg(Arg::with_name("height")
			.short("h")
			.long("height")
			.value_name("Spritesheet Height")
			.help("Final Height of Spritesheet")
			.takes_value(true))
		.arg(Arg::with_name("output_path")
			.short("o")
			.long("output_path")
			.value_name("(.png path, relative to project root)")
			.help("Path for resulting spritesheet to be exported. Value must be relative to project root, as all resources generated will point to it.")
			.takes_value(true))
		.get_matches();

	let mut packer = SpritePacker::new(
		(
			config.value_of("width").unwrap_or("512").parse().expect("Invalid width provided."),
			config.value_of("height").unwrap_or("512").parse().expect("Invalid height provided.")
		)
	);

	let output_image_path:&str = config.value_of("output_path").unwrap_or("atlas.png");
	let mut input_folder = String::from(config.value_of("input_path").unwrap_or("./"));
	if !input_folder.ends_with('/') {
		input_folder += "/";
	}
	for path in glob(&(input_folder + "*.png")).expect("Invalid glob pattern").flatten() {
		if path == PathBuf::from(output_image_path) {
			continue;
		}
		let image = image::open(&path).unwrap();
		packer.add_image(image, path);
	}

	match packer.pack_sprites(output_image_path) {
		Ok(_) => {
			println!("Sprites successfully packed. Saved atlas at: {}", output_image_path);
		},
		Err(err) => {
			println!("An error occured during sprite packing. {:?}", err);
		}
	}
}
