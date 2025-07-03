use serde::{Deserialize, Serialize};

use crate::body::Body;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum EnemyType {
    CopperOrb,
    DeceptiveFlower,
    PurpleBlob,
    SeaGoblin,
}

pub struct Enemy {
    obj: Body,
}
