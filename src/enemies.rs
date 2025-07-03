use serde::{Deserialize, Serialize};

use crate::{body::Body, world::World};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EnemyType {
    CopperOrb,
    DeceptiveFlower,
    PurpleBlob,
    SeaGoblin,
}

pub struct Enemy {
    pub body: Body,
    r#type: EnemyType,
}

impl Enemy {
    pub fn new(r#type: EnemyType, x: f32, y: f32) -> Self {
        return Enemy {
            body: Body::new(x, y, 16.0, 16.0, None),
            r#type,
        };
    }

    pub fn render(&self, world: &World) {
        self.body.render(world);
    }
}
