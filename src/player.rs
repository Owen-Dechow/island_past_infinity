use macroquad::math::vec2;

use crate::{
    asset_loading::AssetManageResult, input::Input, levels::Level, body::Body, sprites::Sprite,
    world::World,
};

pub struct Player {
    pub obj: Body,
}

impl Player {
    pub async fn new(world: &World) -> AssetManageResult<Self> {
        Ok(Self {
            obj: Body::new(
                world.w / 2.0,
                world.h / 2.0,
                14.0,
                12.0,
                Some(Sprite::load_player().await?),
            ),
        })
    }

    pub fn move_player(&mut self, level: &Level, input: &Input, dt: f32) {
        let move_input = vec2(input.horizontal, input.vertical).normalize_or_zero();
        self.obj.r#move(move_input * 60.0, level, dt);
    }
}
