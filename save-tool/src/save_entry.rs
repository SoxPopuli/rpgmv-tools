use crate::{
    error::Error,
    global::{GlobalEntry, SpriteInfo},
};
use iced::advanced::image::Handle;
use image::DynamicImage;
use lib::image::{Spritesheet, SpritesheetKind};
use std::{
    collections::{HashMap, hash_map::Entry},
    fs::File,
    io::{BufReader, Cursor},
    path::{Path, PathBuf},
};

pub type ImageBuffer = image::ImageBuffer<image::Rgba<u8>, Vec<u8>>;

#[derive(Debug, PartialEq, Eq)]
pub struct SaveEntry {
    pub global: GlobalEntry,
    pub face_images: Vec<ImageBuffer>,
    pub character_images: Vec<ImageBuffer>,
}
impl SaveEntry {
    pub fn new(
        entry: GlobalEntry,
        save_dir: &Path,
        file_cache: &mut HashMap<PathBuf, Spritesheet<DynamicImage>>,
    ) -> Result<Self, Error> {
        let img_dir = save_dir
            .parent()
            .ok_or(Error::io_error(format!(
                "no parent of {}",
                save_dir.display()
            )))?
            .join("img");

        let characters_dir = img_dir.join("characters");
        let faces_dir = img_dir.join("faces");

        let load_image_file = |file_path: &Path| -> Result<DynamicImage, Error> {
            let file =
                BufReader::new(File::open(file_path).map_err(|_| {
                    Error::io_error(format!("Missing file: {}", file_path.display()))
                })?);
            let decrypted = {
                let buf = lib::image::decrypt_derive_key(file)?;
                BufReader::new(Cursor::new(buf))
            };

            let mut image_reader = image::ImageReader::new(decrypted);
            image_reader.set_format(image::ImageFormat::Png);

            let decoded = image_reader.decode()?;

            Ok(decoded)
        };

        let mut get_image_buffer =
            |kind: SpritesheetKind, spritesheet_dir: &Path, sprite: &SpriteInfo| {
                let spritesheet_file_path =
                    spritesheet_dir.join(format!("{}.rpgmvp", &sprite.file_name));

                let sheet = match file_cache.entry(spritesheet_file_path.clone()) {
                    Entry::Vacant(vacant) => {
                        // Load image file
                        let file = load_image_file(&spritesheet_file_path);
                        file.map(|f| Spritesheet::new(kind, f))
                            .map(|sheet| vacant.insert(sheet))
                    }
                    Entry::Occupied(occupied) => Ok(occupied.into_mut()),
                };

                let subimage = sheet.and_then(|s| {
                    s.get_subimage(sprite.sprite_index)
                        .ok_or(Error::Image(format!(
                            "Invalid sprite index {} for file {}",
                            sprite.sprite_index, sprite.file_name
                        )))
                });

                subimage.map(|s| s.to_image())
            };

        let face_images = entry
            .faces
            .iter()
            .map(|face| get_image_buffer(SpritesheetKind::Face, &faces_dir, face))
            .collect::<Result<Vec<_>, _>>()?;

        let character_images = entry
            .characters
            .iter()
            .map(|character| {
                get_image_buffer(SpritesheetKind::Character, &characters_dir, character)
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            global: entry,
            face_images,
            character_images,
        })
    }

    pub fn face_image_handles(&self) -> impl Iterator<Item = Handle> {
        self.face_images
            .iter()
            .map(|x| Handle::from_rgba(x.width(), x.height(), x.to_vec()))
    }

    pub fn character_image_handles(&self) -> impl Iterator<Item = Handle> {
        self.character_images
            .iter()
            .map(|x| Handle::from_rgba(x.width(), x.height(), x.to_vec()))
    }
}
