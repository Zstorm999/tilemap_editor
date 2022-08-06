use iced::{
    pure::{
        widget::{Button, Column, Row, Text},
        Sandbox,
    },
    Settings,
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
    OpenFile,
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
            .push(Button::new(Text::new("Open")).on_press(Message::OpenFile))
            .push(Text::new(match &self.tiles_file {
                Some(path) => path.to_str().unwrap().to_string(),
                None => "No file selected".to_string(),
            }))
            .into()
    }

    fn update(&mut self, message: Self::Message) {
        match message {
            Message::OpenFile => {
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
