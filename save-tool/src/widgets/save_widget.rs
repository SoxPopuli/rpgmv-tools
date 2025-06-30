use iced::{
    Alignment, Length, Theme,
    widget::{
        button, column, container, horizontal_space, image, mouse_area, row, text, vertical_rule,
    },
};

use crate::save_entry::SaveEntry;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SaveWidgetMessage {
    HoverEnter,
    HoverExit,
}

#[derive(Debug, PartialEq, Eq)]
pub struct SaveWidget {
    pub file_index: usize,
    pub save_entry: Option<SaveEntry>,
    pub is_hovered: bool,
}
impl SaveWidget {
    pub fn empty(file_index: usize) -> Self {
        Self {
            file_index,
            save_entry: None,
            is_hovered: false,
        }
    }

    pub fn new(file_index: usize, entry: SaveEntry) -> Self {
        Self {
            file_index,
            save_entry: Some(entry),
            is_hovered: false,
        }
    }

    pub fn update(&mut self, msg: SaveWidgetMessage) {
        match msg {
            SaveWidgetMessage::HoverEnter => {
                self.is_hovered = true;
            }
            SaveWidgetMessage::HoverExit => {
                self.is_hovered = false;
            }
        }
    }

    pub fn view(&self) -> iced::Element<SaveWidgetMessage> {
        let background_color = |theme: &iced::Theme| {
            let palette = theme.extended_palette();
            let color = if self.is_hovered {
                palette.background.weak.color
            } else {
                palette.background.base.color
            };
            iced::Background::Color(color)
        };

        const IMAGE_HEIGHT: f32 = 60.0;

        let inner = {
            let faces = self
                .save_entry
                .as_ref()
                .map(|s| {
                    s.face_image_handles()
                        .map(|h| image(h).height(IMAGE_HEIGHT).into())
                })
                .map(row);

            let characters = self
                .save_entry
                .as_ref()
                .map(|s| {
                    s.character_image_handles()
                        .map(|h| image(h).height(IMAGE_HEIGHT).into())
                })
                .map(row);

            let image_col = {
                if faces.is_none() && characters.is_none() {
                    None
                } else {
                    let items = [faces, characters].into_iter().flatten().map(|x| x.into());
                    Some(column(items).align_x(Alignment::Center))
                }
            };

            let content = text(format!("File {}", self.file_index));
            let content = if let Some(images) = image_col {
                row![images, vertical_rule(24.0), content]
            } else {
                row![content]
            }
            .height(Length::Fixed(IMAGE_HEIGHT * 2.0))
            .align_y(Alignment::Center);

            if self.is_hovered && self.save_entry.is_some() {
                let font = iced::font::Font {
                    family: iced::font::Family::Name("Font Awesome 6 Free"),
                    weight: iced::font::Weight::Black,
                    ..Default::default()
                };

                let copy = text("\u{f0c5}").font(font);
                let delete = text("\u{f2ed}").font(font);

                let buttons = container(row![button(copy), button(delete)].spacing(8)).padding(16);
                content.extend([horizontal_space().into(), buttons.into()])
            } else {
                content
            }
        };

        let inner = container(inner)
            .width(Length::Fill)
            .style(move |theme: &Theme| container::Style {
                background: Some(background_color(theme)),
                text_color: Some(if self.save_entry.is_some() {
                    theme.palette().text
                } else {
                    theme.palette().text.scale_alpha(0.5)
                }),
                ..Default::default()
            });

        mouse_area(inner)
            .on_enter(SaveWidgetMessage::HoverEnter)
            .on_exit(SaveWidgetMessage::HoverExit)
            .into()
    }
}
