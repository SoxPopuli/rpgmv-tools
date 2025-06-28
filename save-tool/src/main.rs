mod error;
mod global;
mod save_entry;

use crate::{error::Error, global::GlobalEntry, save_entry::SaveEntry};
use iced::{
    Length, Theme,
    widget::{button, column, row, text, text_input},
};
use lib::save::Json as SaveJson;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Message {
    DirectoryChanged(String),
    OpenDirectoryDialog,
}

pub type ImageBuffer = image::ImageBuffer<image::Rgba<u8>, Vec<u8>>;
pub type Element<'a> = iced::Element<'a, Message>;

#[derive(Debug, Default)]
pub enum SavesState {
    #[default]
    NotLoaded,
    Loaded(Vec<Option<SaveEntry>>),
    Error(Error),
}
impl SavesState {
    pub fn load_from_global(save_dir: &Path, global_path: &Path) -> SavesState {
        let save_json = {
            let file = std::fs::read_to_string(global_path);
            file.ok().and_then(|f| SaveJson::decompress(&f).ok())
        };

        let mut file_cache = HashMap::default();

        let save_json = match save_json {
            None => return SavesState::NotLoaded,
            Some(x) => x,
        };

        let mut from_values = |values: Vec<serde_json::Value>| -> SavesState {
            let values = values
                .into_iter()
                .map(serde_json::from_value::<Option<GlobalEntry>>)
                .map(|entry| match entry {
                    Ok(Some(entry)) => SaveEntry::new(entry, save_dir, &mut file_cache).map(Some),
                    Ok(None) => Ok(None),
                    Err(e) => Err(Error::Io(e.to_string())),
                })
                .collect::<Result<Vec<_>, _>>();

            match values {
                Ok(v) => SavesState::Loaded(v),
                Err(e) => SavesState::Error(Error::global_error(e.to_string())),
            }
        };

        match save_json.inner() {
            serde_json::Value::Array(values) => from_values(values),
            _ => SavesState::Error(Error::global_error("Invalid global file")),
        }
    }
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
            self.saves = SavesState::load_from_global(
                &self.save_directory,
                &self.save_directory.join("global.rpgsave"),
            );
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

        let content: Option<Element> = match &self.saves {
            SavesState::NotLoaded => None,
            SavesState::Error(e) => Some(text(e.to_string()).into()),
            SavesState::Loaded(saves) => {
                let elems = saves.iter().map(|save| match save {
                    Some(s) => row![
                        row(s
                            .face_image_handles()
                            .map(|x| iced::widget::image(x).into())),
                        row(s
                            .character_image_handles()
                            .map(|x| iced::widget::image(x).into()))
                    ]
                    .into(),
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

fn main() {
    App::default().run().expect("Failed to run app");
}
