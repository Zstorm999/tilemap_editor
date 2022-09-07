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
use tilemap::{Layer, TileMap};

use asefile::{AsepriteFile, AsepriteParseError};

mod mapviewer;
mod save;
mod tilemap;
mod tileselector;

use mapviewer::MapViewer;
use tilemap::Tile;
use tileselector::TileSelector;

fn main() -> iced::Result {
    TilemapEditor::run(Settings::default())
}

pub type Tiles = Rc<RefCell<Option<AsepriteFile>>>;

struct TilemapEditor {
    tiles_file: Option<PathBuf>,
    loading_state: LoadingState,
    tile_selector: TileSelector,
    tiles: Tiles,
    map_viewer: MapViewer,
}

enum LoadingState {
    Inactive,
    NewMap,
    OpeningMap,
    SavingMap,
    LoadingTiles,
    Error,
}

impl LoadingState {
    fn inactive(&self) -> bool {
        match self {
            LoadingState::Inactive => true,
            _ => false,
        }
    }

    fn active(&self) -> bool {
        !self.inactive()
    }

    fn is_error(&self) -> bool {
        match self {
            LoadingState::Error => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    ErrorClosed(()), // unit type needed for command

    // handling UI major buttons
    NewMap,
    CreateNewMap(bool),
    OpenMap,
    MapOpened(Option<PathBuf>),
    SaveMap,
    MapSaved(Option<String>),

    // tiles selector events
    OpenTiles,
    TilesOpened(Option<PathBuf>),
    TileSelected(u32),
    TileUnSelected,

    // map viewer events
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
                loading_state: LoadingState::Inactive,
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
                    .push(Button::new(Text::new("New")).on_press(Message::NewMap))
                    .push(Button::new(Text::new("Open")).on_press(Message::OpenMap))
                    .push(Button::new(Text::new("Save")).on_press(Message::SaveMap)),
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
        if self.loading_state.is_error() {
            match message {
                Message::ErrorClosed(_) => self.loading_state = LoadingState::Inactive,
                _ => {}
            }
            return Command::none();
        }

        match message {
            Message::ErrorClosed(_) => {
                self.loading_state = LoadingState::Inactive;
            }

            Message::NewMap => {
                self.loading_state = LoadingState::NewMap;

                return Command::perform(
                    Self::new_map(self.map_viewer.modified),
                    Message::CreateNewMap,
                );
            }

            Message::CreateNewMap(create) => {
                self.loading_state = LoadingState::Inactive;

                if create {
                    self.map_viewer.set_entire_map(TileMap::new(32, 32))
                }
            }

            Message::OpenMap => {
                if self.loading_state.active() {
                    return Command::none();
                }

                self.loading_state = LoadingState::OpeningMap;

                return Command::perform(
                    Self::open_map(self.map_viewer.modified),
                    Message::MapOpened,
                );
            }

            Message::MapOpened(new_map_file) => {
                self.loading_state = LoadingState::Inactive;
                match new_map_file {
                    Some(new_map_file) => {
                        let new_map = save::load_from_file(&new_map_file);

                        match new_map {
                            Ok(new_map) => self.map_viewer.set_entire_map(new_map),
                            Err(err) => {
                                self.loading_state = LoadingState::Error;
                                return Command::perform(
                                    Self::error_opening_map(new_map_file, err.to_string()),
                                    Message::ErrorClosed,
                                );
                            }
                        }
                    }
                    None => {}
                }
            }

            Message::SaveMap => {
                if self.loading_state.active() {
                    return Command::none();
                }

                self.loading_state = LoadingState::SavingMap;

                return Command::perform(
                    Self::save_map(self.map_viewer.get_map_instant()),
                    Message::MapSaved,
                );
            }
            Message::MapSaved(potential_error) => match potential_error {
                None => {
                    self.loading_state = LoadingState::Inactive;
                    self.map_viewer.modified = false;
                }
                Some(error_message) => {
                    self.loading_state = LoadingState::Error;
                    return Command::perform(
                        Self::error_with_save(error_message),
                        Message::ErrorClosed,
                    );
                }
            },

            Message::OpenTiles => {
                if self.loading_state.active() {
                    return Command::none();
                }
                self.loading_state = LoadingState::LoadingTiles;

                return Command::perform(
                    Self::open_tiles(self.tiles_file.is_some()),
                    Message::TilesOpened,
                );
            }

            Message::TilesOpened(new_tiles) => {
                self.loading_state = LoadingState::Inactive;

                if new_tiles.is_some() {
                    let file = AsepriteFile::read_file(&new_tiles.as_ref().unwrap());
                    match file {
                        Ok(f) => {
                            self.tiles_file = new_tiles;
                            *self.tiles.borrow_mut() = Some(f);

                            self.tile_selector.reset();
                            self.map_viewer.refresh();
                        }
                        Err(err) => {
                            self.loading_state = LoadingState::Error;
                            return Command::perform(
                                Self::error_with_tiles(new_tiles.clone().unwrap(), err),
                                Message::ErrorClosed,
                            );
                        }
                    }
                }
            }

            Message::TileSelected(i) => self.tile_selector.select(i),
            Message::TileUnSelected => self.tile_selector.unselect(),
            Message::PaintTile(x, y) => self.map_viewer.set_tile(
                x,
                y,
                self.tile_selector.get_selected().map_or_else(
                    || self.map_viewer.get_tile(x, y, Layer::Background), // if no selected tile preserves current tile
                    |tile| Some(Tile::new(tile, false, false)),           // otherwise overwrite it
                ),
            ),
            Message::ClearTile(x, y) => self.map_viewer.set_tile(x, y, None),
        }

        Command::none()
    }
}

impl TilemapEditor {
    async fn new_map(modified: bool) -> bool {
        // only case where we do not create a new map is modified and !keep, corresponding to a NAND
        !(modified && keep_modifications().await)
    }

    async fn open_map(modified: bool) -> Option<PathBuf> {
        if modified && keep_modifications().await {
            return None;
        }

        return AsyncFileDialog::new()
            .add_filter("RON", &["ron", "RON"])
            .pick_file()
            .await
            .map(|h| h.path().into());
    }

    async fn error_opening_map(file: PathBuf, err: String) {
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

    async fn save_map(map: TileMap) -> Option<String> {
        if let Some(file) = AsyncFileDialog::new()
            .add_filter("RON", &["ron", "RON"])
            .save_file()
            .await
            .map(|h| h.path().into())
        {
            return match save::save_in_file(map, file) {
                Ok(_) => None,
                Err(err) => Some(err.to_string()),
            };
        }

        None
    }

    async fn error_with_save(message: String) {
        AsyncMessageDialog::new()
            .set_level(rfd::MessageLevel::Error)
            .set_buttons(rfd::MessageButtons::Ok)
            .set_title("Error saving map")
            .set_description(&format!("There was an error saving the map :\n{}", message))
            .show()
            .await;
    }

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

/// Prompts the user if they want to keep overwrite their modifications.
///
/// Returns `true` if they should be kept, or `false` if they should be overwritten.
async fn keep_modifications() -> bool {
    if !AsyncMessageDialog::new().set_level(rfd::MessageLevel::Warning).set_buttons(rfd::MessageButtons::YesNo).set_title("Map modified").set_description("The current tilemap has been modified since last save. Do you still want to open a new one ? All changes will be lost").show().await {
        return true;
    }
    return false;
}
