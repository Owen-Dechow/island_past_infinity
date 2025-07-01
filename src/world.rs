use crate::{SUB_PIX_LEVEL, VIRTUAL_H, VIRTUAL_W};

#[derive(Debug)]
pub struct World {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl World {
    pub fn new() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            w: VIRTUAL_W,
            h: VIRTUAL_H,
        }
    }

    pub fn rounded(&self) -> Self {
        let sub = 1.0 / SUB_PIX_LEVEL;
        Self {
            x: (self.x / sub).round() * sub,
            y: (self.y / sub).round() * sub,
            w: self.w,
            h: self.h,
        }
    }
}
