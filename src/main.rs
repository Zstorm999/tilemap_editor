use iced::{
    executor,
    pure::{
        horizontal_rule, scrollable, vertical_rule,
        widget::{Button, Column, Row, Text},
        Application, Element,
    },
    Alignment, Command, Length, Settings,
};

use rfd::{AsyncFileDialog, AsyncMessageDialog};
use std::{cell::RefCell, path::PathBuf, rc::Rc};

use asefile::{AsepriteFile, AsepriteParseError};

mod mapviewer;
mod tilemap;
mod tileselector;

use mapviewer::MapViewer;
use tileselector::TileSelector;

fn main() -> iced::Result {
    TilemapEditor::run(Settings::default())
}

pub type Tiles = Rc<RefCell<Option<AsepriteFile>>>;

struct TilemapEditor {
    tiles_file: Option<PathBuf>,
    loading_tiles: bool,
    error_message: bool,
    tile_selector: TileSelector,
    tiles: Tiles,
    map_viewer: MapViewer,
}

#[derive(Debug, Clone)]
pub enum Message {
    OpenTiles,
    TilesOpened(Option<PathBuf>),
    ErrorClosed(()), // unit type needed for command
    TileSelected(u32),
    TileUnSelected,
    PaintTile(u16, u16),
    ClearTile(u16, u16),
}

impl Application for TilemapEditor {
    type Message = Message;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Message>) {
        let tiles = Rc::new(RefCell::new(None));
        (
            TilemapEditor {
                tiles_file: None,
                loading_tiles: false,
                error_message: false,
                tile_selector: TileSelector::new(tiles.clone()),
                map_viewer: MapViewer::new(tiles.clone()),
                tiles,
            },
            Command::none(),
        )
    }

    fn title(&self) -> String {
        "Tilemap editor".to_string()
    }

    fn view(&self) -> Element<'_, Self::Message> {
        Column::new()
            // menu bar
            .push(
                Row::new()
                    .push(Button::new(Text::new("New")))
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
                            .push(scrollable(self.tile_selector.view()).height(Length::Fill))
                            .push(Button::new("Open tiles").on_press(Message::OpenTiles)),
                    )
                    .push(vertical_rule(2))
                    .push(Column::new().push(self.map_viewer.view())),
            )
            .into()
    }

    fn update(&mut self, message: Self::Message) -> Command<Message> {
        if self.error_message {
            match message {
                Message::ErrorClosed(_) => self.error_message = false,
                _ => {}
            }
            return Command::none();
        }

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
                self.loading_tiles = false;

                if new_tiles.is_some() {
                    let file = AsepriteFile::read_file(&new_tiles.as_ref().unwrap());
                    match file {
                        Ok(f) => {
                            self.tiles_file = new_tiles;
                            *self.tiles.borrow_mut() = Some(f);

                            self.tile_selector.reset();
                            self.map_viewer.reset();
                        }
                        Err(err) => {
                            return Command::perform(
                                Self::error_with_tiles(new_tiles.clone().unwrap(), err),
                                Message::ErrorClosed,
                            )
                        }
                    }
                }
            }
            Message::ErrorClosed(_) => {
                self.error_message = false;
            }
            Message::TileSelected(i) => self.tile_selector.select(i),
            Message::TileUnSelected => self.tile_selector.unselect(),
            Message::PaintTile(x, y) => {
                self.map_viewer
                    .set_tile(x, y, self.tile_selector.get_selected())
            }
            Message::ClearTile(x, y) => self.map_viewer.set_tile(x, y, None),
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

    async fn error_with_tiles(file: PathBuf, err: AsepriteParseError) {
        AsyncMessageDialog::new()
            .set_level(rfd::MessageLevel::Error)
            .set_buttons(rfd::MessageButtons::Ok)
            .set_title("Error opening file")
            .set_description(&format!(
                "There was an error opening the file {:?}:\n{}",
                file, err
            ))
            .show()
            .await;
    }
}
