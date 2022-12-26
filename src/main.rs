use clap::Arg;

use glob::glob;
use notify::{RecommendedWatcher, RecursiveMode, Watcher, EventKind};

use std::{path::{PathBuf}};

mod atlas_gen;
use atlas_gen::{packer::{SpritePacker}, tres_writer::ResourceFormat};

// TODO: !Write Tests!
fn main() {
	let config = clap::App::new("crunch-gd")
		.about("Generates spritesheets & associated AtlasTexture resources for Godot 4. Uses ChevyRay's crunch-rs for sprite packing.")
		.author("jordi")
		.version("1.11")
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
		.arg(Arg::with_name("gd3")
			.long("gd3")
			.takes_value(false)
			.display_order(5)
			.help("Generate AtlasTexture resources for Godot 3.x intead of 4.0"))
		.arg(Arg::with_name("watch")
			.long("watch")
			.takes_value(false)
			.display_order(6)
			.help("While running, regenerate atlas when input files changed."))
		.get_matches();

	// Initialize packer instance
	let mut packer = SpritePacker::new(
		(
			config.value_of("width").unwrap_or("512").parse().expect("Invalid width provided."),
			config.value_of("height").unwrap_or("512").parse().expect("Invalid height provided.")
		),
		config.value_of("padding").unwrap_or("0").parse().expect("Invalid padding provided."),
	);

	// Get output path
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

	let format:ResourceFormat = if !config.is_present("gd3") {
		ResourceFormat::Gd4
	} else {
		println!("Resources will be written for Godot 3.x");
		ResourceFormat::Gd3
	};

	// Pack sprites
	packer.find_input_files(&input_folder, &output_image_path);
	match packer.pack_sprites(&output_image_path, format) {
		Ok(_) => {
			println!("Sprites successfully packed. Saved atlas at: {}", output_image_path.to_string_lossy());
		},
		Err(err) => {
			println!("An error occured during sprite packing. {:?}", err);
		}
	}

	// Watch for file changes and re-pack sprites when needed
	if !config.is_present("watch") {
		return;
	}
    let (tx, rx) = std::sync::mpsc::channel();

	let mut watcher = RecommendedWatcher::new(tx, notify::Config::default()).expect("An error occured while enabling file watching.");

	watcher.watch(input_folder.as_path(), RecursiveMode::NonRecursive).expect("An error occured while enabling file watching.");
	println!("Watching for changes in input folder...");
    'event_loop: for e in rx {
		match e {
			Ok(event) => {
				if !event.kind.is_create() && !event.kind.is_modify() {
					continue;
				}
				for path in event.paths {
					if path == output_image_path {
						continue 'event_loop;
					}
					if let Some(extension) = path.extension() {
						if extension != "png" {
							continue 'event_loop;
						}
					}
				}
				// Disable file watching while atlas is being generated.
				watcher.unwatch(input_folder.as_path()).expect("An error occured while disabling file watch.");
				packer.find_input_files(&input_folder, &output_image_path);
				match packer.pack_sprites(&output_image_path, format) {
					Ok(_) => {
						println!("Sprites atlas regenerated.");
					},
					Err(err) => {
						println!("An error occured during sprite packing. {:?}", err);
					}
				}
				// Re-enable since files are now finalized.
				watcher.watch(input_folder.as_path(), RecursiveMode::NonRecursive).expect("An error occured while enabling file watching.");
			},
			Err(err) => println!("An error occured while watching files... {:?}", err),
		}
	}
}
