use std::{fmt::Display, iter};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct TileMap {
    background: LayerContent,
    foreground: LayerContent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layer {
    Background,
    Foreground,
}

impl Layer {
    pub const ALL: [Layer; 2] = [Layer::Background, Layer::Foreground];
}

impl Display for Layer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Layer::Background => "Background",
                Layer::Foreground => "Foreground",
            }
        )
    }
}

#[derive(Debug, Clone)]
struct LayerContent {
    width: u16,
    height: u16,
    tiles: Vec<Vec<Option<Tile>>>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Tile {
    pub value: u32,
    pub h_flip: bool,
    pub v_flip: bool,
}

impl Tile {
    pub fn new(value: u32, h_flip: bool, v_flip: bool) -> Self {
        Tile {
            value,
            h_flip,
            v_flip,
        }
    }
}

impl Default for TileMap {
    fn default() -> Self {
        TileMap::new(32, 32)
    }
}

impl TileMap {
    pub fn new(width: u16, height: u16) -> Self {
        TileMap {
            background: LayerContent::new(width, height),
            foreground: LayerContent::new(width, height),
        }
    }

    pub fn set_tile(&mut self, x: u16, y: u16, value: Option<Tile>, layer: Layer) {
        match layer {
            Layer::Background => self.background.set_tile(x, y, value),
            Layer::Foreground => self.foreground.set_tile(x, y, value),
        }
    }

    pub fn get_tile(&self, x: u16, y: u16) -> (Option<Tile>, Option<Tile>) {
        (
            self.background.get_tile(x, y),
            self.foreground.get_tile(x, y),
        )
    }

    pub fn resize(&mut self, new_width: u16, new_height: u16) {
        self.background.resize(new_width, new_height);
        self.foreground.resize(new_width, new_height);
    }

    pub fn get_dimensions(&self) -> (u16, u16) {
        (self.background.width, self.background.height)
        // need only to return one since they are always equal
    }
}

impl LayerContent {
    fn new(width: u16, height: u16) -> Self {
        LayerContent {
            width,
            height,
            tiles: Vec::from_iter(
                iter::repeat(Vec::from_iter(iter::repeat(None).take(height.into())))
                    .take(width.into()),
            ),
        }
    }

    fn set_tile(&mut self, x: u16, y: u16, value: Option<Tile>) {
        let x = x as usize;
        let y = y as usize;

        self.tiles[x][y] = value;
    }

    fn get_tile(&self, x: u16, y: u16) -> Option<Tile> {
        let x = x as usize;
        let y = y as usize;

        self.tiles[x][y]
    }

    fn resize(&mut self, new_width: u16, new_height: u16) {
        // first take care of height
        if new_height > self.height {
            // grow
            self.tiles.extend(
                iter::repeat(Vec::from_iter(iter::repeat(None).take(new_width as usize)))
                    .take((new_height - self.height).into()),
            )
        } else {
            // height
            self.tiles.truncate(new_height as usize);
        }
        self.height = new_height;

        // then width
        if new_width > self.width {
            // grow
            for row in &mut self.tiles[..] {
                row.extend(iter::repeat(None).take((new_width - self.width).into()))
            }
        } else {
            // shrink
            for row in &mut self.tiles[..] {
                row.truncate(new_width as usize);
            }
        }
        self.width = new_width;
    }
}
