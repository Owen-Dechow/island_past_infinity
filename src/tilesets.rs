use std::path::PathBuf;

use macroquad::texture::{Image, Texture2D};
use serde::{Deserialize, Serialize};

use crate::{
    asset_loading::{load_tex_with_meta, AssetManageResult},
    TILE_COLLISION_SECTIONS, TILE_SIZE,
};

pub struct TilesetAsset {
    pub tex: Texture2D,
    pub tiles: Vec<TileAsset>,
    pub meta_path: PathBuf,
}

impl TilesetAsset {
    fn new(serializable: TilesetAssetSerializable, tex: Texture2D) -> TilesetAsset {
        TilesetAsset {
            tex,
            tiles: serializable.tiles,
            meta_path: serializable.meta_path,
        }
    }

    pub async fn load(tile_asset: &str) -> AssetManageResult<Self> {
        let path = format!("assets/art/tiles/{}.png", tile_asset);
        let (serializable, tex) = load_tex_with_meta(path).await?;

        return Ok(Self::new(serializable, tex));
    }

    pub fn get_tile_at_pos(&self, x: f32, y: f32) -> Option<usize> {
        return self
            .tiles
            .iter()
            .enumerate()
            .filter_map(|(idx, t)| {
                let a = x - t.x;
                let b = y - t.y;
                if a == 0.0 && b == 0.0 {
                    Some(idx)
                } else {
                    None
                }
            })
            .last();
    }

    fn is_section_transparent(
        &self,
        img: &Image,
        start_y: usize,
        end_y: usize,
        start_x: usize,
        end_x: usize,
    ) -> bool {
        let bytes = &img.bytes;
        let width = img.width as usize;

        for y in start_y..end_y {
            for x in start_x..end_x {
                let idx = (y * width + x) * 4;
                if bytes[idx + 3] > 0 {
                    return false;
                }
            }
        }

        return true;
    }

    pub fn cut(&mut self) {
        let rows = (self.tex.width() / TILE_SIZE) as usize;
        let cols = (self.tex.height() / TILE_SIZE) as usize;
        let img = self.tex.get_texture_data();
        for row in 0..rows {
            for col in 0..cols {
                let x = TILE_SIZE * col as f32;
                let y = TILE_SIZE * row as f32;

                if let Some(tile) = self.get_tile_at_pos(x, y) {
                    let tile = self.tiles.get_mut(tile).expect("Tile should exist");
                    if let TileLayer::Object = tile.layer {
                        if let None = tile.collision_matrix {
                            tile.collision_matrix = Some(CollisionMatrix::new());
                        }
                    }
                } else {
                    let start_y = y as usize;
                    let end_y = start_y + TILE_SIZE as usize;
                    let start_x = x as usize;
                    let end_x = start_x + TILE_SIZE as usize;

                    if !self.is_section_transparent(&img, start_y, end_y, start_x, end_x) {
                        self.tiles.push(TileAsset {
                            x,
                            y,
                            auto_rule: None,
                            layer: TileLayer::Object,
                            group: None,
                            collision_matrix: Some(CollisionMatrix::new()),
                        });
                    }
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TilesetAssetSerializable {
    pub tiles: Vec<TileAsset>,
    pub meta_path: PathBuf,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum TileLayer {
    Background,
    Object,
    Overlay,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TileAutoRule {
    pub top_left: Option<bool>,
    pub top: Option<bool>,
    pub top_right: Option<bool>,
    pub right: Option<bool>,
    pub bottom_right: Option<bool>,
    pub bottom: Option<bool>,
    pub bottom_left: Option<bool>,
    pub left: Option<bool>,
}

impl TileAutoRule {
    pub fn from_array(array: [bool; 8]) -> Self {
        TileAutoRule {
            top_left: Some(array[0]),
            top: Some(array[1]),
            top_right: Some(array[2]),
            right: Some(array[3]),
            bottom_right: Some(array[4]),
            bottom: Some(array[5]),
            bottom_left: Some(array[6]),
            left: Some(array[7]),
        }
    }

    pub fn cmp(&self, other: &TileAutoRule) -> Option<usize> {
        let mut points = 0;

        let sets = [
            (self.top_left, other.top_left),
            (self.top, other.top),
            (self.top_right, other.top_right),
            (self.right, other.right),
            (self.bottom_right, other.bottom_right),
            (self.bottom, other.bottom),
            (self.bottom_left, other.bottom_left),
            (self.left, other.left),
        ];

        for set in sets {
            match set {
                (Some(a), Some(b)) => match a == b {
                    true => points += 1,
                    false => return None,
                },
                _ => points += 0,
            };
        }

        return Some(points);
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CollisionMatrix {
    pub matrix: [[bool; TILE_COLLISION_SECTIONS as usize]; TILE_COLLISION_SECTIONS as usize],
}

impl CollisionMatrix {
    pub fn new() -> Self {
        Self {
            matrix: [[true, true, true], [true, true, true], [true, true, true]],
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TileAsset {
    pub x: f32,
    pub y: f32,
    pub auto_rule: Option<TileAutoRule>,
    pub layer: TileLayer,
    pub group: Option<u8>,
    pub collision_matrix: Option<CollisionMatrix>,
}
