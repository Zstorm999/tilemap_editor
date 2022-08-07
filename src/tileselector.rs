use iced::{
    canvas::{event::Status, Event},
    mouse,
    pure::{
        widget::{canvas, Canvas},
        Element,
    },
    Color, Length, Point, Size, Vector,
};

use crate::{Message, Tiles};

const TILES_PER_LINE: u32 = 5;
const SCALE_FACTOR: u32 = 4;

pub struct TileSelector {
    selected: Option<u32>,
    content: Tiles,
    cache: canvas::Cache,
}

impl TileSelector {
    pub fn new(tiles: Tiles) -> Self {
        TileSelector {
            selected: None,
            content: tiles,
            cache: Default::default(),
        }
    }

    pub fn view(&self) -> Element<'_, crate::Message> {
        Canvas::new(self)
            .width(Length::Units(
                ((1 + 9 * TILES_PER_LINE) * SCALE_FACTOR)
                    .try_into()
                    .unwrap(),
            ))
            .height(Length::Units(match &*self.content.borrow() {
                Some(content) => {
                    ((((content.num_frames() as f32 / TILES_PER_LINE as f32).ceil() as u32) * 9
                        + 1)
                        * SCALE_FACTOR)
                        .try_into()
                        .unwrap()
                }
                None => 0,
            }))
            .into()
    }

    pub fn select(&mut self, i: u32) {
        match &*self.content.borrow() {
            Some(content) => {
                if i < content.num_frames() {
                    self.selected = Some(i);
                    self.cache.clear();
                }
            }
            None => {}
        }
    }

    pub fn unselect(&mut self) {
        self.selected = None;
        self.cache.clear();
    }

    pub fn reset(&mut self) {
        self.selected = None;
        self.cache.clear();
        // do NOT reset reference to tiles, otherwise it’s lost forever !
    }
}

impl canvas::Program<Message> for TileSelector {
    type State = ();

    fn update(
        &self,
        _state: &mut Self::State,
        event: iced::canvas::Event,
        bounds: iced::Rectangle,
        cursor: iced::canvas::Cursor,
    ) -> (iced::canvas::event::Status, Option<Message>) {
        if self.content.borrow().is_none() {
            return (Status::Ignored, None);
        }

        let cursor_position = if let Some(position) = cursor.position_in(&bounds) {
            position
        } else {
            return (Status::Ignored, None);
        };

        match event {
            Event::Mouse(mouse_event) => match mouse_event {
                mouse::Event::ButtonPressed(button) => match button {
                    mouse::Button::Left => {
                        let x_tile = cursor_position.x.round() as u32 / (9 * SCALE_FACTOR);
                        let y_tile = cursor_position.y.round() as u32 / (9 * SCALE_FACTOR);

                        let pressed = x_tile + y_tile * TILES_PER_LINE;

                        if let Some(current) = self.selected {
                            if current == pressed {
                                // same, ignore
                                return (Status::Captured, None);
                            }
                        }
                        (Status::Captured, Some(Message::TileSelected(pressed)))
                    }
                    mouse::Button::Right => (Status::Captured, Some(Message::TileUnSelected)),
                    _ => (Status::Ignored, None),
                },
                _ => (Status::Ignored, None),
            },
            _ => (Status::Ignored, None),
        }
    }

    fn draw(
        &self,
        _state: &Self::State,
        bounds: iced::Rectangle,
        _cursor: iced::canvas::Cursor,
    ) -> Vec<iced::canvas::Geometry> {
        let selector = self.cache.draw(bounds.size(), |frame| {
            if let Some(content) = &*self.content.borrow() {
                // for each tile
                for i in 0..content.num_frames() {
                    // for each pixel in the tile

                    if let Some(selected) = self.selected {
                        if selected == i {
                            frame.with_save(|frame| {
                                frame.translate(Vector::new(
                                    (9 * (i % TILES_PER_LINE) * SCALE_FACTOR) as f32,
                                    (9 * (i / TILES_PER_LINE) * SCALE_FACTOR) as f32,
                                ));

                                let fill = Color::new(1.0, 0.0, 0.0, 0.7);

                                // top
                                frame.fill_rectangle(
                                    Point { x: 0.0, y: 0.0 },
                                    Size::new((10 * SCALE_FACTOR) as f32, SCALE_FACTOR as f32),
                                    fill,
                                );

                                // left
                                frame.fill_rectangle(
                                    Point { x: 0.0, y: 0.0 },
                                    Size::new(SCALE_FACTOR as f32, (10 * SCALE_FACTOR) as f32),
                                    fill,
                                );

                                // down
                                frame.fill_rectangle(
                                    Point {
                                        x: 0.0,
                                        y: (9 * SCALE_FACTOR) as f32,
                                    },
                                    Size::new((10 * SCALE_FACTOR) as f32, SCALE_FACTOR as f32),
                                    fill,
                                );

                                // right
                                frame.fill_rectangle(
                                    Point {
                                        x: (9 * SCALE_FACTOR) as f32,
                                        y: 0.0,
                                    },
                                    Size::new(SCALE_FACTOR as f32, (10 * SCALE_FACTOR) as f32),
                                    fill,
                                );
                            });
                        }
                    }

                    for (idx, pixel) in content.frame(i).image().pixels().take(64).enumerate() {
                        frame.with_save(|frame| {
                            // move at pixel location

                            frame.translate(Vector::new(
                                ((9 * (i % TILES_PER_LINE) + 1 + (idx as u32 % 8)) * SCALE_FACTOR)
                                    as f32,
                                ((9 * (i / TILES_PER_LINE) + 1 + (idx as u32 / 8)) * SCALE_FACTOR)
                                    as f32,
                            ));

                            frame.fill_rectangle(
                                Point::new(0 as f32, 0 as f32),
                                Size::new(SCALE_FACTOR as f32, SCALE_FACTOR as f32),
                                Color::new(
                                    pixel.0[0] as f32 / 255.0,
                                    pixel.0[1] as f32 / 255.0,
                                    pixel.0[2] as f32 / 255.0,
                                    pixel.0[3] as f32 / 255.0,
                                ),
                            )
                        })
                    }
                }
            }
        });

        vec![selector]
    }
}
