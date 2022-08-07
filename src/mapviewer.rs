use iced::{
    canvas::event::Status,
    pure::{
        widget::{canvas, Canvas},
        Element,
    },
    Color, Length, Point, Size,
};

use crate::{tilemap::TileMap, Message, Tiles};

pub struct MapViewer {
    map: TileMap,
    cache: canvas::Cache,
    tiles: Tiles,
}

impl MapViewer {
    pub fn new(tiles: Tiles) -> Self {
        let mut map: TileMap = Default::default();

        map.set_tile(0, 0, Some(0), crate::tilemap::Layer::Background);
        map.set_tile(0, 1, Some(0), crate::tilemap::Layer::Background);
        map.set_tile(0, 0, Some(1), crate::tilemap::Layer::Foreground);

        MapViewer {
            map,
            cache: Default::default(),
            tiles,
        }
    }

    pub fn view(&self) -> Element<'_, Message> {
        Canvas::new(self)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    pub fn reset(&mut self) {
        self.cache.clear();
    }
}

const SCALE_FACTOR: f32 = 2.0;
const BORDER_SIZE: f32 = 1.0;

impl canvas::Program<Message> for MapViewer {
    type State = ();

    fn update(
        &self,
        _state: &mut Self::State,
        _event: iced::canvas::Event,
        _bounds: iced::Rectangle,
        _cursor: iced::canvas::Cursor,
    ) -> (iced::canvas::event::Status, Option<Message>) {
        (Status::Ignored, None)
    }

    fn draw(
        &self,
        _state: &Self::State,
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

            frame.fill_rectangle(
                Point::new(0.0, 0.0),
                Size::new(width as f32 * tile_side, height as f32 * tile_side),
                default_colour,
            );

            if let Some(tiles) = &*self.tiles.borrow() {
                // draw tiles
                for y in 0..height {
                    for x in 0..width {
                        let (bg_tile, fg_tile) = self.map.get_tile(x, y);

                        // draw background first
                        if let Some(tile) = bg_tile {
                            if tile < tiles.num_frames() {
                                // this is a valid index for the current tiles
                                for (idx, pixel) in
                                    tiles.frame(tile).image().pixels().take(64).enumerate()
                                {
                                    frame.fill_rectangle(
                                        Point::new(
                                            x as f32 * (8.0 * SCALE_FACTOR + BORDER_SIZE)
                                                + (idx % 8) as f32 * SCALE_FACTOR,
                                            y as f32 * (8.0 * SCALE_FACTOR + BORDER_SIZE)
                                                + (idx / 8) as f32 * SCALE_FACTOR,
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

                        // then draw foreground above
                        if let Some(tile) = fg_tile {
                            if tile < tiles.num_frames() {
                                // this is a valid index for the current tiles
                                for (idx, pixel) in
                                    tiles.frame(tile).image().pixels().take(64).enumerate()
                                {
                                    frame.fill_rectangle(
                                        Point::new(
                                            x as f32 * (8.0 * SCALE_FACTOR + BORDER_SIZE)
                                                + (idx % 8) as f32 * SCALE_FACTOR,
                                            y as f32 * (8.0 * SCALE_FACTOR + BORDER_SIZE)
                                                + (idx / 8) as f32 * SCALE_FACTOR,
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
