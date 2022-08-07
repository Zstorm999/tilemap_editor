use iced::{
    pure::{
        horizontal_rule, scrollable, vertical_rule,
        widget::{Button, Column, Row, Text},
        Sandbox,
    },
    Alignment, Length, Settings,
};

use rfd::{FileDialog, MessageDialog};
use std::path::PathBuf;

fn main() -> iced::Result {
    TilemapEditor::run(Settings::default())
}

#[derive(Default)]
struct TilemapEditor {
    tiles_file: Option<PathBuf>,
}

#[derive(Debug, Clone, Copy)]
enum Message {
    OpenTiles,
}

impl Sandbox for TilemapEditor {
    type Message = Message;

    fn new() -> Self {
        TilemapEditor { tiles_file: None }
    }

    fn title(&self) -> String {
        "Tilemap editor".to_string()
    }

    fn view(&self) -> iced::pure::Element<'_, Self::Message> {
        Column::new()
            // menu bar
            .push(
                Row::new()
                    .push(Button::new(Text::new("Open")))
                    .push(Button::new(Text::new("Save"))),
            )
            .push(horizontal_rule(2))
            // window content
            .push(
                Row::new()
                    .push(
                        // tiles column
                        Column::new()
                            .align_items(Alignment::Center)
                            .width(Length::Units(200))
                            .push(Text::new(match &self.tiles_file {
                                Some(path) => path.file_name().unwrap().to_str().unwrap(),
                                None => "No file selected",
                            }))
                            .push(scrollable(Text::new("Tiles will go there")).height(Length::Fill))
                            .push(Button::new("Open tiles").on_press(Message::OpenTiles)),
                    )
                    .push(vertical_rule(2))
                    .push(Column::new()),
            )
            .into()
    }

    fn update(&mut self, message: Self::Message) {
        match message {
            Message::OpenTiles => {
                if self.tiles_file.is_some() {
                    match MessageDialog::new().set_level(rfd::MessageLevel::Warning).set_buttons(rfd::MessageButtons::OkCancel).set_title("Tilesheet already loaded").set_description("A tilesheet is already loaded. Loading a new one will overwrite the previous one, and may break your map").show() {
                        false => return,
                        true => {},
                    }
                }

                let new_tiles = FileDialog::new()
                    .add_filter("aseprite", &["ase", "aseprite"])
                    .pick_file();

                if new_tiles.is_some() {
                    self.tiles_file = new_tiles;
                }
            }
        }
    }
}
