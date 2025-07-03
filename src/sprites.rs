use macroquad::texture::Texture2D;
use serde::{Deserialize, Serialize};

use crate::asset_loading::{load_tex_with_meta, AssetManageResult};

#[derive(Serialize, Deserialize)]
pub struct SpriteFrameSpan {
    pub start_frame: usize,
    pub number_of_frames: usize,
    pub duration_seconds: f32,
}

impl Default for SpriteFrameSpan {
    fn default() -> Self {
        Self {
            start_frame: 0,
            number_of_frames: 0,
            duration_seconds: 0.0,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct SpriteSerializable {
    pub up: SpriteFrameSpan,
    pub down: SpriteFrameSpan,
    pub side: SpriteFrameSpan,
    pub frames: Vec<(f32, f32)>,
    pub frame_w: f32,
    pub frame_h: f32,
}

impl Default for SpriteSerializable {
    fn default() -> Self {
        Self {
            up: SpriteFrameSpan::default(),
            down: SpriteFrameSpan::default(),
            side: SpriteFrameSpan::default(),
            frames: Vec::new(),
            frame_w: 0.0,
            frame_h: 0.0,
        }
    }
}

pub struct Sprite {
    pub tex: Texture2D,
    pub up: SpriteFrameSpan,
    pub down: SpriteFrameSpan,
    pub side: SpriteFrameSpan,
    pub frames: Vec<(f32, f32)>,
    pub frame_w: f32,
    pub frame_h: f32,
}

impl Sprite {
    const PATH: &str = "assets/art/sprites";

    async fn load(serializable: SpriteSerializable, tex: Texture2D) -> Self {
        Self {
            tex,
            up: serializable.up,
            down: serializable.down,
            side: serializable.side,
            frames: serializable.frames,
            frame_w: serializable.frame_w,
            frame_h: serializable.frame_h,
        }
    }

    pub async fn load_player() -> AssetManageResult<Sprite> {
        let path = format!("{}/player.png", Self::PATH);
        let (serializable, tex) = load_tex_with_meta(path).await?;
        return Ok(Self::load(serializable, tex).await);
    }
}
