use asefile::AsepriteFile;
use iced::{
    canvas::{event::Status, Event, Frame},
    mouse,
    pure::{
        widget::{canvas, Canvas},
        Element,
    },
    Color, Length, Point, Size,
};

use crate::{
    tilemap::{Layer, Tile, TileMap},
    Message, Tiles,
};

pub struct MapViewer {
    pub modified: bool,
    pub tool: Tool,
    pub tile: Option<Tile>,
    map: TileMap,
    cache: canvas::Cache,
    tiles: Tiles,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Tool {
    Pen,
    Rect,
    Selection,
}

impl MapViewer {
    pub fn new(tiles: Tiles) -> Self {
        MapViewer {
            modified: false,
            map: Default::default(),
            tile: None,
            cache: Default::default(),
            tiles,
            tool: Tool::Pen,
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        let (width, height) = self.map.get_dimensions();
        let tile_side = (8.0 * SCALE_FACTOR + BORDER_SIZE) as u16;

        Canvas::new(self)
            .width(Length::Units(width * tile_side - 1))
            .height(Length::Units(height * tile_side - 1))
            .into()
    }

    pub fn set_tile(&mut self, x: u16, y: u16, value: Option<Tile>) {
        self.modified = true;
        self.map.set_tile(x, y, value, Layer::Background);
        self.cache.clear();
    }

    pub fn get_tile(&self, x: u16, y: u16, layer: Layer) -> Option<Tile> {
        let tiles = self.map.get_tile(x, y);
        match layer {
            Layer::Background => tiles.0,
            Layer::Foreground => tiles.1,
        }
    }

    pub fn fill_rect(&mut self, x: u16, y: u16, width: i32, height: i32) {
        let min_x = i32::min(x as i32, x as i32 + width);
        let min_y = i32::min(y as i32, y as i32 + height);

        for x in min_x..(min_x + width.abs()) {
            for y in min_y..(min_y + height.abs()) {
                self.set_tile(x as u16, y as u16, self.tile);
            }
        }
    }

    /// Clear the cache and force redrawing
    pub fn refresh(&mut self) {
        self.cache.clear();
    }

    pub fn get_map_instant(&self) -> TileMap {
        self.map.clone()
    }

    pub fn set_entire_map(&mut self, map: TileMap) {
        self.map = map;
        self.modified = false;
        self.cache.clear();
    }
}

const SCALE_FACTOR: f32 = 2.0;
const BORDER_SIZE: f32 = 1.0;

#[derive(Default, Debug)]
pub struct ViewerState {
    interaction: Interaction,
    rect_dimensions: (i32, i32),
}

#[derive(Default, Debug)]
enum Interaction {
    #[default]
    None,
    Drawing,
    Rectangle(u16, u16),
    Erasing,
}

impl canvas::Program<Message> for MapViewer {
    type State = ViewerState;

    fn update(
        &self,
        state: &mut Self::State,
        event: iced::canvas::Event,
        bounds: iced::Rectangle,
        cursor: iced::canvas::Cursor,
    ) -> (iced::canvas::event::Status, Option<Message>) {
        let (x, y) = if let Some(position) = cursor.position_in(&bounds) {
            let tile_side = 8.0 * SCALE_FACTOR + BORDER_SIZE;
            (
                (position.x / tile_side).floor() as u16,
                (position.y / tile_side).floor() as u16,
            )
        } else {
            return (Status::Ignored, None);
        };

        match event {
            Event::Mouse(event) => match event {
                mouse::Event::ButtonReleased(_) => {
                    match state.interaction {
                        Interaction::Rectangle(x, y) => {
                            state.interaction = Interaction::None;
                            return (
                                Status::Captured,
                                Some(Message::PaintRect(
                                    x,
                                    y,
                                    state.rect_dimensions.0,
                                    state.rect_dimensions.1,
                                )),
                            );
                        }
                        _ => {}
                    }

                    state.interaction = Interaction::None;
                }
                mouse::Event::ButtonPressed(button) => match button {
                    mouse::Button::Left => match self.tool {
                        Tool::Pen => {
                            state.interaction = Interaction::Drawing;
                            return (Status::Captured, Some(Message::PaintTile(x, y)));
                        }
                        Tool::Rect => {
                            state.interaction = Interaction::Rectangle(x, y);
                            state.rect_dimensions = (1, 1);
                            return (Status::Captured, Some(Message::RectStarted));
                        }
                        _ => {}
                    },
                    mouse::Button::Right => {
                        state.interaction = Interaction::Erasing;
                        return (Status::Captured, Some(Message::ClearTile(x, y)));
                    }
                    _ => {}
                },
                mouse::Event::CursorMoved { .. } => match state.interaction {
                    Interaction::Drawing => {
                        return (Status::Captured, Some(Message::PaintTile(x, y)))
                    }
                    Interaction::Erasing => {
                        return (Status::Captured, Some(Message::ClearTile(x, y)))
                    }
                    Interaction::Rectangle(rect_x, rect_y) => {
                        let length = |a: u16, b: u16| {
                            let sub = a as i32 - b as i32;
                            sub + if sub >= 0 { 1 } else { 0 }
                        };

                        let new_width = length(x, rect_x);
                        let new_height = length(y, rect_y);

                        state.rect_dimensions = (new_width, new_height);
                        return (Status::Captured, Some(Message::Redraw));
                    }
                    _ => {}
                },

                _ => {}
            },
            Event::Keyboard(_) => {}
        }

        (Status::Ignored, None)
    }

    fn draw(
        &self,
        state: &Self::State,
        bounds: iced::Rectangle,
        _cursor: iced::canvas::Cursor,
    ) -> Vec<iced::canvas::Geometry> {
        let map_view = self.cache.draw(bounds.size(), |frame| {
            let (width, height) = self.map.get_dimensions();

            let default_colour = Color::new(
                0x35 as f32 / 255.0,
                0x79 as f32 / 255.0,
                0x60 as f32 / 255.0,
                1.0,
            );

            let border_colour = Color::new(0.7, 0.7, 0.7, 1.0);

            let tile_side = 8.0 * SCALE_FACTOR + BORDER_SIZE;

            // fill base colour
            frame.fill_rectangle(
                Point::new(0.0, 0.0),
                Size::new(width as f32 * tile_side, height as f32 * tile_side),
                default_colour,
            );

            // fill tiles
            if let Some(tiles) = &*self.tiles.borrow() {
                // draw tiles
                for y in 0..height {
                    for x in 0..width {
                        let (bg_tile, fg_tile) = self.map.get_tile(x, y);

                        // draw background first
                        if let Some(tile) = bg_tile {
                            draw_tile(tile, x, y, frame, tiles);
                        }

                        // then draw foreground above
                        if let Some(tile) = fg_tile {
                            draw_tile(tile, x, y, frame, tiles);
                        }
                    }
                }

                // draw preview rect ?
                if let Interaction::Rectangle(x_rect, y_rect) = state.interaction {
                    let (width, height) = state.rect_dimensions;

                    let min_x = i32::min(x_rect as i32, x_rect as i32 + width);
                    let min_y = i32::min(y_rect as i32, y_rect as i32 + height);

                    for x in min_x..(min_x + width.abs()) {
                        for y in min_y..(min_y + height.abs()) {
                            if let Some(tile) = self.tile {
                                draw_tile(tile, x as u16, y as u16, frame, tiles);
                            }
                        }
                    }
                }
            }

            // draw grid
            // vertical lines
            for line in 0..width {
                frame.fill_rectangle(
                    Point::new(line as f32 * tile_side, 0.0),
                    Size::new(BORDER_SIZE, height as f32 * tile_side),
                    border_colour,
                )
            }

            // horizontal rows
            for row in 0..height {
                frame.fill_rectangle(
                    Point::new(0.0, row as f32 * tile_side),
                    Size::new(width as f32 * tile_side, BORDER_SIZE),
                    border_colour,
                )
            }
        });
        vec![map_view]
    }
}

fn draw_tile(tile: Tile, x: u16, y: u16, frame: &mut Frame, tiles: &AsepriteFile) {
    if tile.value < tiles.num_frames() {
        // this is a valid index for the current tiles
        for (idx, pixel) in tiles
            .frame(tile.value)
            .image()
            .pixels()
            .take(64)
            .enumerate()
        {
            frame.fill_rectangle(
                Point::new(
                    x as f32 * (8.0 * SCALE_FACTOR + BORDER_SIZE) + (idx % 8) as f32 * SCALE_FACTOR,
                    y as f32 * (8.0 * SCALE_FACTOR + BORDER_SIZE) + (idx / 8) as f32 * SCALE_FACTOR,
                ),
                Size::new(SCALE_FACTOR, SCALE_FACTOR),
                Color::new(
                    pixel.0[0] as f32 / 255.0,
                    pixel.0[1] as f32 / 255.0,
                    pixel.0[2] as f32 / 255.0,
                    pixel.0[3] as f32 / 255.0,
                ),
            )
        }
    }
}
