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
			.display_order(0)
			.default_value("./")
			.hide_default_value(true)
			.help("Location of all sprites to be packed")
			.takes_value(true))
		.arg(Arg::with_name("output_path")
			.short("o")
			.long("output_path")
			.display_order(1)
			.default_value("./atlas.png")
			.help("Path for resulting spritesheet to be exported. Value must be relative to project root, as all resources generated will point to it.")
			.takes_value(true))
		.arg(Arg::with_name("width")
			.short("w")
			.long("width")
			.display_order(2)
			.value_name("Spritesheet Width")
			.default_value("512")
			.help("Final Width of Spritesheet")
			.takes_value(true))
		.arg(Arg::with_name("height")
			.short("h")
			.long("height")
			.display_order(3)
			.value_name("Spritesheet Height")
			.default_value("512")
			.help("Final Height of Spritesheet")
			.takes_value(true))
		.arg(Arg::with_name("padding")
			.short("p")
			.long("padding")
			.value_name("Padding")
			.display_order(4)
			.help("Amount of empty space to put between each sprite")
			.default_value("0")
			.takes_value(true))
		.get_matches();

	let mut packer = SpritePacker::new(
		(
			config.value_of("width").unwrap_or("512").parse().expect("Invalid width provided."),
			config.value_of("height").unwrap_or("512").parse().expect("Invalid height provided.")
		),
		config.value_of("padding").unwrap_or("0").parse().expect("Invalid padding provided."),
	);

	let mut output_image_path:PathBuf = PathBuf::from(config.value_of("output_path").unwrap_or("./atlas.png"));
	if output_image_path.extension().is_none() {
		println!("Invalid output path provided. Path must be a file path. Ex: \"project/atlas.png\"");
		return;
	}
	output_image_path.set_extension("png");
	let input_folder = PathBuf::from(config.value_of("input_path").unwrap_or("./"));
	if input_folder.extension().is_some() {
		println!("Invalid input folder provided. Path must be a path to a folder. Ex: \"sprites_to_pack/\"");
		return;
	}
	for path in glob(&(input_folder.to_string_lossy() + "*.png")).expect("Invalid glob pattern").flatten() {
		if path == output_image_path {
			continue;
		}
		let image = image::open(&path).unwrap();
		if let Err(err) = packer.add_image(image, path.clone()) {
			println!("Failed to pack {:?}. Error: {:?}", path, err);
		}
	}

	match packer.pack_sprites(&output_image_path) {
		Ok(_) => {
			println!("Sprites successfully packed. Saved atlas at: {}", output_image_path.to_string_lossy());
		},
		Err(err) => {
			println!("An error occured during sprite packing. {:?}", err);
		}
	}
}
