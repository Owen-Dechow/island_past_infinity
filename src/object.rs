use serde::{Deserialize, Serialize};
use std::ops::Range;

use crate::{
    body::Body,
    enemies::{Enemy, EnemyType},
    world::World,
    TILE_SIZE,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ObjectListing {
    row: usize,
    col: usize,
    r#type: ObjectType,
}

impl ObjectListing {
    pub fn is_in_range(&self, row_range: &Range<usize>, col_range: &Range<usize>) -> bool {
        return row_range.contains(&self.row) && col_range.contains(&self.col);
    }

    pub fn resolve(&self) -> Object {
        let x = self.col as f32 * TILE_SIZE + TILE_SIZE / 2.0;
        let y = self.row as f32 * TILE_SIZE + TILE_SIZE / 2.0;

        return match &self.r#type {
            ObjectType::Enemy(enemy_type) => Object::Enemy(Enemy::new(enemy_type.clone(), x, y)),
        };
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ObjectType {
    Enemy(EnemyType),
}

pub enum Object {
    Enemy(Enemy),
}

impl Object {
    fn get_y_sort_key(&self) -> i32 {
        match self {
            Object::Enemy(enemy) => enemy.body.get_y_sort_key(),
        }
    }

    fn render(&self, world: &World) {
        match self {
            Object::Enemy(enemy) => enemy.body.render(world),
        }
    }
}

pub struct LevelObjects {
    lst: Vec<Object>,
}

impl LevelObjects {
    pub fn new() -> Self {
        Self { lst: Vec::new() }
    }

    pub fn add_listing(&mut self, listing: &ObjectListing) {
        self.lst.push(listing.resolve());
    }

    pub fn render(&mut self, other_bodies: &mut [&Body], world: &World) {
        other_bodies.sort_by_key(|body| body.get_y_sort_key());
        self.lst.sort_by_key(|obj| obj.get_y_sort_key());

        let mut obj_idx = 0;
        let calc_obj_y = |idx: usize| match self.lst.get(idx) {
            Some(obj) => Some(obj.get_y_sort_key()),
            None => None,
        };
        let mut obj_y = calc_obj_y(obj_idx);

        let mut body_idx = 0;
        let calc_bodies_y = |idx: usize| match other_bodies.get(idx) {
            Some(first) => Some(first.get_y_sort_key()),
            None => None,
        };
        let mut bodies_y = calc_bodies_y(body_idx);

        loop {
            match (obj_y, bodies_y) {
                (None, None) => break,
                (None, Some(_)) => {
                    other_bodies[body_idx].render(world);
                    body_idx += 1;
                    bodies_y = calc_bodies_y(body_idx);
                }
                (Some(_), None) => {
                    self.lst[obj_idx].render(world);
                    obj_idx += 1;
                    obj_y = calc_obj_y(obj_idx);
                }
                (Some(obj), Some(body)) => match obj > body {
                    true => {
                        other_bodies[body_idx].render(world);
                        body_idx += 1;
                        bodies_y = calc_bodies_y(body_idx);
                    }
                    false => {
                        self.lst[body_idx].render(world);
                        obj_idx += 1;
                        obj_y = calc_obj_y(obj_idx);
                    }
                },
            }
        }
    }
}
