use iced::{
    executor,
    pure::{
        horizontal_rule, scrollable, vertical_rule,
        widget::{Button, Column, Row, Text},
        Application,
    },
    Alignment, Command, Length, Settings,
};

use rfd::{AsyncFileDialog, AsyncMessageDialog};
use std::path::PathBuf;

fn main() -> iced::Result {
    TilemapEditor::run(Settings::default())
}

#[derive(Default)]
struct TilemapEditor {
    tiles_file: Option<PathBuf>,
    loading_tiles: bool,
}

#[derive(Debug, Clone)]
enum Message {
    OpenTiles,
    TilesOpened(Option<PathBuf>),
}

impl Application for TilemapEditor {
    type Message = Message;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        (
            TilemapEditor {
                tiles_file: None,
                loading_tiles: false,
            },
            Command::none(),
        )
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

    fn update(&mut self, message: Self::Message) -> Command<Message> {
        // don’t process any event if a file is loading

        match message {
            Message::OpenTiles => {
                if self.loading_tiles {
                    return Command::none();
                }
                self.loading_tiles = true;

                return Command::perform(
                    Self::open_tiles(self.tiles_file.is_some()),
                    Message::TilesOpened,
                );
            }
            Message::TilesOpened(new_tiles) => {
                if new_tiles.is_some() {
                    self.tiles_file = new_tiles;
                }

                self.loading_tiles = false;
            }
        }

        Command::none()
    }
}

impl TilemapEditor {
    async fn open_tiles(has_a_file: bool) -> Option<PathBuf> {
        if has_a_file {
            match AsyncMessageDialog::new().set_level(rfd::MessageLevel::Warning).set_buttons(rfd::MessageButtons::OkCancel).set_title("Tilesheet already loaded").set_description("A tilesheet is already loaded. Loading a new one will overwrite the previous one, and may break your map").show().await {
                false => return None,
                true => {},
            }
        }

        return AsyncFileDialog::new()
            .add_filter("aseprite", &["ase", "aseprite"])
            .pick_file()
            .await
            .map(|h| h.path().into());
    }
}
