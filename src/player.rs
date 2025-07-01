use macroquad::math::vec2;

use crate::{
    input::Input,
    object::Object,
    resources::{levels::Level, sprites::Sprite, AssetLoadError},
    world::World,
};

pub struct Player {
    pub obj: Object,
}

impl Player {
    pub async fn new(world: &World) -> Result<Self, AssetLoadError> {
        Ok(Self {
            obj: Object::new(
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
