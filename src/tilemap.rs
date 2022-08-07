use std::iter;
pub struct TileMap {
    background: LayerContent,
    foreground: LayerContent,
}

pub enum Layer {
    Background,
    Foreground,
}

struct LayerContent {
    width: u16,
    height: u16,
    tiles: Vec<Vec<Option<u32>>>,
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

    pub fn set_tile(&mut self, x: u16, y: u16, value: Option<u32>, layer: Layer) {
        match layer {
            Layer::Background => self.background.set_tile(x, y, value),
            Layer::Foreground => self.foreground.set_tile(x, y, value),
        }
    }

    pub fn get_tile(&self, x: u16, y: u16) -> (Option<u32>, Option<u32>) {
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

    fn set_tile(&mut self, x: u16, y: u16, value: Option<u32>) {
        let x = x as usize;
        let y = y as usize;

        self.tiles[x][y] = value;
    }

    fn get_tile(&self, x: u16, y: u16) -> Option<u32> {
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