use ron::ser::PrettyConfig;
use serde::{Deserialize, Serialize};
use std::{fs, io, path::PathBuf};

use crate::tilemap::{Layer as TMLayer, Tile, TileMap};

#[derive(Serialize, Deserialize)]
struct TileMapStorage {
    width: u16,
    height: u16,
    background: Layer,
    foreground: Layer,
}

#[derive(Serialize, Deserialize)]
struct Layer {
    tiles: Vec<Option<Tile>>,
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

impl From<TileMapStorage> for TileMap {
    fn from(map: TileMapStorage) -> Self {
        let (width, height) = (map.width, map.height);

        let mut out_map = TileMap::new(width, height);

        for y in 0..height {
            for x in 0..width {
                out_map.set_tile(
                    x,
                    y,
                    map.background.tiles[(x + y * width) as usize],
                    TMLayer::Background,
                );

                out_map.set_tile(
                    x,
                    y,
                    map.foreground.tiles[(x + y * width) as usize],
                    TMLayer::Foreground,
                );
            }
        }

        out_map
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
                .depth_limit(2)
                .compact_arrays(true)
                .new_line(String::from("\n")),
        )
        .unwrap(),
    )?;

    Ok(())
}

pub fn load_from_file(file: &PathBuf) -> io::Result<TileMap> {
    let content = fs::read(file)?;

    let map: TileMapStorage = ron::de::from_bytes(&content).unwrap();
    Ok(map.into())
}
