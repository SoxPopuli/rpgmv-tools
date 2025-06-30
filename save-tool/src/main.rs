mod error;
mod global;
mod save_entry;
mod saves_state;
mod widgets;

use crate::saves_state::SavesState;
use iced::{
    Length, Theme,
    widget::{button, column, row, scrollable, text, text_input, vertical_space},
};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub enum Message {
    DirectoryChanged(String),
    OpenDirectoryDialog,
    SaveSelectionChanged(String),
    SaveWidgetMessage(usize, widgets::save_widget::SaveWidgetMessage),
    FontLoaded,
}

pub type Element<'a> = iced::Element<'a, Message>;

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
            SaveSelectionChanged(s) => {}
            SaveWidgetMessage(widget_index, msg) => {
                if let SavesState::Loaded { entries, .. } = &mut self.saves
                    && let Some(widget) = entries.get_mut(widget_index)
                {
                    widget.update(msg);
                }
            }
            FontLoaded => {}
        }
    }

    fn view_saves(&self) -> Element {
        match &self.saves {
            SavesState::NotLoaded => vertical_space().into(),
            SavesState::Error(e) => text(e.to_string())
                .style(|theme: &Theme| text::Style {
                    color: Some(theme.palette().danger),
                })
                .into(),
            SavesState::Loaded {
                entries: saves,
                names: _,
            } => {
                let views = saves.iter().enumerate().skip(1).map(|(i, widget)| {
                    widget
                        .view()
                        .map(move |msg| Message::SaveWidgetMessage(i, msg))
                });

                scrollable(column(views)).into()
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

        let content = self.view_saves();

        // let content =
        //     iced::widget::scrollable(row![text(format!("{:#?}", self.saves)),]).width(Length::Fill);

        column![folder_box, iced::widget::horizontal_rule(24), content]
            .padding(8)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn theme(&self) -> Theme {
        Theme::Dark
    }

    fn run(self, task: iced::Task<Message>) -> iced::Result {
        iced::application(env!("CARGO_BIN_NAME"), Self::update, Self::view)
            .theme(Self::theme)
            .run_with(|| (self, task))
    }
}

fn main() {
    let fa_font = include_bytes!("../Font Awesome 6 Free-Solid-900.otf");
    let task = iced::font::load(fa_font).map(|e| {
        if let Err(e) = e {
            panic!("Error loading font: {e:?}")
        }
        Message::FontLoaded
    });

    App::default().run(task).expect("Failed to run app");
}
