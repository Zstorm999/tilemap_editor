use ron::ser::PrettyConfig;
use serde::Serialize;
use std::{fs, io, path::PathBuf};

use crate::tilemap::TileMap;

#[derive(Serialize)]
struct TileMapStorage {
    width: u16,
    height: u16,
    background: Layer,
    foreground: Layer,
}

#[derive(Serialize)]
struct Layer {
    tiles: Vec<Option<u32>>,
}

impl From<TileMap> for TileMapStorage {
    fn from(map: TileMap) -> Self {
        let (width, height) = map.get_dimensions();
        let mut out = TileMapStorage {
            width,
            height,
            background: Layer {
                tiles: Vec::with_capacity((width * height) as usize),
            },
            foreground: Layer {
                tiles: Vec::with_capacity((width * height) as usize),
            },
        };

        for y in 0..height {
            for x in 0..width {
                let (bg_tile, fg_tile) = map.get_tile(x, y);
                out.background.tiles.push(bg_tile);
                out.foreground.tiles.push(fg_tile);
            }
        }

        out
    }
}

pub fn save_in_file(map: TileMap, file: PathBuf) -> io::Result<()> {
    let storage: TileMapStorage = map.into();

    // unwrap because the struct cannot be ill-formed, and I don’t care enough
    fs::write(
        file,
        ron::ser::to_string_pretty(
            &storage,
            PrettyConfig::new()
                .compact_arrays(true)
                .new_line(String::from("\n")),
        )
        .unwrap(),
    )?;

    Ok(())
}
