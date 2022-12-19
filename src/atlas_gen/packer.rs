use std::{path::PathBuf};

use crunch::{Item, Rect, pack};
use image::{ImageBuffer, Rgba, RgbaImage, DynamicImage, GenericImageView};

use crate::atlas_gen::tres_writer::{AtlasResourceWriter};

pub fn trim_transparency(image:&DynamicImage) -> Result<Rect, SpritePackingError> {
	let mut rect:Rect = Rect::new(0, 0, image.width() as usize, image.height() as usize);
	'find_nontransparent_x: for x in 0..image.width() {
		for column_y in 0..image.height() {
			if image.get_pixel(x, column_y).0[3] != 0 {
				break 'find_nontransparent_x;
			}
		}
		rect.x = x.saturating_add(1) as usize;
	}

	'find_last_nontransparent_x: for x in image.width()..0 {
		for column_y in 0..image.height() {
			if image.get_pixel(x, column_y).0[3] != 0 {
				break 'find_last_nontransparent_x;
			}
		}
		rect.w = x.saturating_sub(1) as usize;
	}
	if rect.w == 0 {
		return Err(SpritePackingError::ImageEmpty);
	}
	rect.w -= rect.x;

	'find_nontransparent_y: for y in 0..image.height() {
		for row_x in 0..image.width() {
			if image.get_pixel(row_x, y).0[3] != 0 {
				break 'find_nontransparent_y;
			}
		}
		rect.y = y.saturating_add(1) as usize;
	}

	'find_last_nontransparent_y: for y in image.height()..0 {
		for row_x in 0..image.width() {
			if image.get_pixel(row_x, y).0[3] != 0 {
				break 'find_last_nontransparent_y;
			}
		}
		rect.h = y.saturating_sub(1) as usize;
	}
	if rect.h == 0 {
		return Err(SpritePackingError::ImageEmpty);
	}
	rect.h -= rect.y;

	Ok(rect)
}

type TrimmedImage = ImageBuffer<Rgba<u8>, Vec<u8>>;
#[derive(Clone)]
pub struct ImageInfo(pub TrimmedImage, pub PathBuf);

const MAX_FAIL_COUNT:u8 = 3;
#[derive(Debug)]
pub enum SpritePackingError {
	ImageError(image::ImageError),
	IoError(std::io::Error),
	InputSpriteTooLarge,
	ImageEmpty,
}

pub struct SpritePacker {
	pub sheet_size:(usize, usize),
	pub padding:usize,
	images:Vec<Item<ImageInfo>>,
	fail_count:u8,
}

impl SpritePacker {
	pub fn new(sheet_size:(usize, usize), padding:usize) -> Self {
		SpritePacker {
			sheet_size,
			padding,
			images: Vec::new(),
			fail_count: 0,
		}
	}

	pub fn add_image(&mut self, image:image::DynamicImage, path:PathBuf) -> Result<(), SpritePackingError> {
		let mut trimmed_rect = match trim_transparency(&image) {
			Ok(rect) => rect,
			Err(err) => return Err(err),
		};
		let sub = image.view(trimmed_rect.x as u32, trimmed_rect.y as u32, trimmed_rect.w as u32, trimmed_rect.h as u32).to_image();
		match trimmed_rect.w.checked_add(self.padding) {
			Some(val) => trimmed_rect.w = val,
			None => return Err(SpritePackingError::InputSpriteTooLarge)
		}
		match trimmed_rect.h.checked_add(self.padding) {
			Some(val) => trimmed_rect.h = val,
			None => return Err(SpritePackingError::InputSpriteTooLarge)
		}
		self.images.push(Item::new(ImageInfo(sub, path), trimmed_rect.w, trimmed_rect.h, crunch::Rotation::None));
		Ok(())
	}

	pub fn pack_sprites(mut self, output_path:&PathBuf) -> Result<(), SpritePackingError> {
		let container = Rect::of_size(self.sheet_size.0, self.sheet_size.1);
		let packed_items = match pack(container, self.images.clone()) {
			Ok(all_items_packed) => all_items_packed,
			Err(_) => {
				self.fail_count += 1;
				if self.fail_count > MAX_FAIL_COUNT {
					return Err(SpritePackingError::InputSpriteTooLarge)
				}
				self.sheet_size.0 =
					if let Some(val) = self.sheet_size.0.checked_mul(2) {
						val
					} else {
						return Err(SpritePackingError::InputSpriteTooLarge)
					};
				self.sheet_size.1 =
					if let Some(val) = self.sheet_size.1.checked_mul(2) {
						val
					} else {
						return Err(SpritePackingError::InputSpriteTooLarge)
					};
				println!("Some items could not be fit into the target spritesheet size. Trying {:?}...", self.sheet_size);
				return self.pack_sprites(output_path);
			},
		};

		// Get paths needed
		if let Err(err) = std::fs::create_dir_all(output_path.with_file_name("")) {
			return Err(SpritePackingError::IoError(err));
		}
		let atlas_writer = match AtlasResourceWriter::new(output_path.clone()) {
			Ok(writer) => writer,
			Err(err) => return Err(SpritePackingError::IoError(err)),
		};

		let mut buffer: RgbaImage = image::ImageBuffer::new(self.sheet_size.0 as u32, self.sheet_size.1 as u32);
		for (rect, image_info) in &packed_items {
			let image = &image_info.0;
			let image_path = &image_info.1;
			match atlas_writer.write(&image_path.file_name().unwrap().to_string_lossy(), rect) {
				Ok(_) => {},
				Err(err) => {
					println!("Error writting AtlasTexture resource for {}: {:?}", image_path.display(), err);
				}
			}

			let rect_x = rect.x as u32;
			let rect_y = rect.y as u32;
			for pixel in image.enumerate_pixels() {
				buffer.put_pixel(rect_x + pixel.0, rect_y + pixel.1, *pixel.2);
			}
		}
		match buffer.save(output_path) {
			Ok(_) => Ok(()),
			Err(err) => Err(SpritePackingError::ImageError(err))
		}
	}
}