use std::{fs::File, io::{Write, self}, path::{PathBuf, Path}};

use crunch::Rect;
use path_slash::PathBufExt;

const ATLAS_RESOURCE_TEMPLATE:&str =
r#"[gd_resource type="AtlasTexture" load_steps=2 format=3]

[ext_resource type="Texture2D" path="{RES_PATH}" id="1"]

[resource]
atlas = ExtResource("1")
region = Rect2({x}, {y}, {w}, {h})

"#;

pub struct AtlasResourceWriter {
	godot_relative_path: PathBuf,
	output_path: PathBuf,
}

impl AtlasResourceWriter {
	pub fn new(output_path: PathBuf) -> io::Result<Self> {
		Ok(AtlasResourceWriter {
			godot_relative_path: match Self::get_path_relative_to_gd_proj(&output_path) {
				Ok(path) => path,
				Err(err) => return Err(err),
			},
			output_path,
		})
	}

	fn get_path_relative_to_gd_proj(path:&Path) -> io::Result<PathBuf> {
		let mut parent_path:Option<&Path> = Some(path);
		while parent_path.is_some() {
			if let Some(path_to_test) = parent_path {
				let mut test_path = path_to_test.to_path_buf();
				test_path.push("project.godot");
				match test_path.try_exists() {
					Ok(found) => {
						if found {
							if let Ok(result) = path.strip_prefix(path_to_test) {
								return Ok(result.to_path_buf());
							} else {
								return Err(io::Error::new(io::ErrorKind::NotFound, "Couldn't create Godot-relative path from output path."));
							};
						} else {
							parent_path = path_to_test.parent();
							continue;
						}
					},
					Err(err) => return Err(err),
				}
			} else {
				return Err(io::Error::new(io::ErrorKind::NotFound, "Couldn't find Godot-relative path in output path."))
			}
		}
		// If there's no parent, or there's an error, it's probably fine to just use the original directory provided.
		// The user may not have intended to export straight into their Godot project.
		Ok(path.to_path_buf())
	}

	pub fn write(&self, file_name:&str, region:&Rect) -> io::Result<()> {
		let mut file_content = String::from(ATLAS_RESOURCE_TEMPLATE);
		// Godot doesn't like backslashes. Convert them to slashes here.
		file_content = file_content.replace("{RES_PATH}", &("res://".to_owned() + &self.godot_relative_path.to_slash_lossy()));
		file_content = file_content.replace("{x}", &region.x.to_string());
		file_content = file_content.replace("{y}", &region.y.to_string());
		file_content = file_content.replace("{w}", &region.w.to_string());
		file_content = file_content.replace("{h}", &region.h.to_string());

		let mut resource_path = self.output_path.with_file_name(file_name);
		resource_path.set_extension("tres");
		let mut file:File = match File::options().create(true).truncate(true).write(true).open(resource_path) {
			Ok(f) => f,
			Err(err) => return Err(err),
		};
		// `file` is automatically closed when dropped
		file.write_all(file_content.as_bytes())
	}
}