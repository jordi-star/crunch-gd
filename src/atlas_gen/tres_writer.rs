use std::{fs::File, io::{Write, self}};

const ATLAS_RESOURCE_TEMPLATE:&str = r#"[gd_resource type="AtlasTexture" load_steps=2 format=3]

[ext_resource type="Texture2D" path="{RES_PATH}" id="1"]

[resource]
atlas = ExtResource("1")
region = Rect2({x}, {y}, {w}, {h})

"#;

pub fn write_gd_atlas_resource(output_path:&str, img_path:&str, x:usize, y:usize, w:usize, h:usize) -> io::Result<()> {
	let mut file_content = String::from(ATLAS_RESOURCE_TEMPLATE);
	// Godot doesn't like backslashes. Convert them to slashes here.
	file_content = file_content.replace("{RES_PATH}", &("res://".to_owned() + &output_path.replace(r"\", "/")));
	file_content = file_content.replace("{x}", &x.to_string());
	file_content = file_content.replace("{y}", &y.to_string());
	file_content = file_content.replace("{w}", &w.to_string());
	file_content = file_content.replace("{h}", &h.to_string());

	let mut file:File = match File::options().create(true).truncate(true).write(true).open(img_path.replace(".png", ".tres")) {
		Ok(f) => f,
		Err(err) => return Err(err),
	};
	match file.write_all(file_content.as_bytes()) {
		Ok(_) => {
			Ok(())
		},
		Err(err) => Err(err),
	}
	// `file` is automatically closed when dropped
}