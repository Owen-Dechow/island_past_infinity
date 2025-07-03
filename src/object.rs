use serde::{Deserialize, Serialize};

use crate::enemies::EnemyType;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ObjectListing {
    row: usize,
    col: usize,
    r#typw: ObjectType,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ObjectType {
    Enemy(EnemyType),
}
