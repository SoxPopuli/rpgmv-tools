mod error;
mod global;

use crate::{error::Error, global::SpriteInfo};
use iced::{
    Length, Task, Theme,
    advanced::image::Handle,
    widget::{button, column, row, text, text_editor, text_input},
};
use image::{DynamicImage, EncodableLayout};
use lib::{
    image::{Spritesheet, SpritesheetKind},
    save::Json as SaveJson,
};
use std::{
    collections::{HashMap, hash_map::Entry},
    convert::identity,
    fs::File,
    io::{BufReader, Cursor},
    path::{Path, PathBuf},
};

use crate::global::GlobalEntry;

#[derive(Debug, PartialEq, Eq, Clone)]
enum Message {
    DirectoryChanged(String),
    OpenDirectoryDialog,
}

type ImageBuffer = image::ImageBuffer<image::Rgba<u8>, Vec<u8>>;

#[derive(Debug)]
pub struct SaveEntry {
    global: GlobalEntry,
    face_images: Vec<ImageBuffer>,
    character_images: Vec<ImageBuffer>,
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
}

type Element<'a> = iced::Element<'a, Message>;

#[derive(Debug, Default)]
enum SavesState {
    #[default]
    NotLoaded,
    Loaded(Vec<Option<SaveEntry>>),
    Error(error::Error),
}

#[derive(Debug, Default)]
struct App {
    save_directory: PathBuf,
    saves: SavesState,
}
impl App {
    fn update(&mut self, msg: Message) {
        use Message::*;

        let mut on_new_dir = |dir: &Path| {
            self.save_directory = dir.into();
            self.saves = load_from_global(&self.save_directory.join("global.rpgsave"));
        };

        match msg {
            DirectoryChanged(new_dir) => {
                on_new_dir(Path::new(&new_dir));
            }
            OpenDirectoryDialog => {
                let dir = rfd::FileDialog::default()
                    .set_title("Open Save Directory")
                    .pick_folder();

                if let Some(new_dir) = dir {
                    on_new_dir(&new_dir);
                }
            }
        }
    }

    fn view(&self) -> iced::Element<Message> {
        let folder_box = row![
            text_input("Save directory", &self.save_directory.to_string_lossy())
                .on_input(Message::DirectoryChanged),
            button("...").on_press(Message::OpenDirectoryDialog),
        ]
        .width(Length::Fill)
        .spacing(8);

        let face_images = |s: &SaveEntry| -> Element {
            let faces = s.face_images.iter().map(|f| {
                let handle = Handle::from_rgba(f.width(), f.height(), f.to_vec());
                iced::widget::image(handle).into()
            });

            row(faces).into()
        };

        let character_images = |s: &SaveEntry| -> Element {
            let characters = s.character_images.iter().map(|c| {
                let handle = Handle::from_rgba(c.width(), c.height(), c.to_vec());
                iced::widget::image(handle).into()
            });

            row(characters).into()
        };

        let content: Option<Element> = match &self.saves {
            SavesState::NotLoaded => None,
            SavesState::Error(e) => Some(text(e.to_string()).into()),
            SavesState::Loaded(saves) => {
                let elems = saves.iter().map(|save| match save {
                    Some(s) =>  {
                        row![
                            face_images(s),
                            character_images(s),
                        ].into()
                    }
                    None => text("None").into(),
                });

                Some(column(elems).into())
            }
        };

        let content = content.unwrap_or(iced::widget::vertical_space().into());

        // let content =
        //     iced::widget::scrollable(row![text(format!("{:#?}", self.saves)),]).width(Length::Fill);

        column![folder_box, content]
            .padding(8)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }

    fn run(self) -> iced::Result {
        iced::application(env!("CARGO_BIN_NAME"), Self::update, Self::view)
            .theme(Self::theme)
            .run()
    }
}

fn load_from_global(path: &Path) -> SavesState {
    let save_json = {
        let file = std::fs::read_to_string(path);
        file.ok().and_then(|f| SaveJson::decompress(&f).ok())
    };

    let save_dir = match path.parent() {
        Some(p) => p,
        None => {
            return SavesState::Error(Error::io_error(format!(
                "Invalid directory: {}",
                path.display()
            )));
        }
    };

    let mut file_cache = HashMap::default();

    save_json
        .map(|x| match x.inner() {
            serde_json::Value::Array(values) => {
                let values = values
                    .into_iter()
                    .map(serde_json::from_value::<Option<GlobalEntry>>)
                    .map(|entry| match entry {
                        Ok(Some(entry)) => {
                            SaveEntry::new(entry, save_dir, &mut file_cache).map(Some)
                        }
                        Ok(None) => Ok(None),
                        Err(e) => Err(Error::Io(e.to_string())),
                    })
                    .collect::<Result<Vec<_>, _>>();

                match values {
                    Ok(v) => SavesState::Loaded(v),
                    Err(e) => SavesState::Error(Error::global_error(e.to_string())),
                }
            }
            _ => SavesState::Error(Error::global_error("Invalid global file")),
        })
        .unwrap_or(SavesState::NotLoaded)
}

fn main() {
    App::default().run().expect("Failed to run app");
}
