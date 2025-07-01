use macroquad::{
    color::WHITE,
    math::{Rect, Vec2},
    texture::{draw_texture_ex, DrawTextureParams},
};

use crate::resources::sprites::Sprite;

enum Direction {
    Up,
    Down,
    Left,
    Right,
}

pub struct Animator {
    direction: Direction,
    time_moving: f32,
    sprite: Sprite,
}

impl Animator {
    pub fn new(sprite: Sprite) -> Self {
        Self {
            direction: Direction::Down,
            time_moving: 0.0,
            sprite,
        }
    }

    pub fn apply_delta(&mut self, delta: Vec2, dt: f32) {
        if delta.x != 0.0 || delta.y != 0.0 {
            let delta_abs = delta.abs();
            if delta_abs.x >= delta_abs.y {
                self.direction = match delta.x > 0.0 {
                    true => Direction::Right,
                    false => Direction::Left,
                }
            } else if delta_abs.y > delta_abs.x {
                self.direction = match delta.y > 0.0 {
                    true => Direction::Down,
                    false => Direction::Up,
                }
            }

            self.time_moving += dt;
        } else {
            self.time_moving = 0.0;
        }
    }

    pub fn render(&self, r#box: &Rect) {
        let (frame_span, flip_x) = match self.direction {
            Direction::Up => (&self.sprite.up, false),
            Direction::Down => (&self.sprite.down, false),
            Direction::Left => (&self.sprite.side, true),
            Direction::Right => (&self.sprite.side, false),
        };

        let frame;
        if self.time_moving > 0.0 || frame_span.duration_seconds == 0.0 {
            let prog = self.time_moving / frame_span.duration_seconds;
            frame = frame_span.start_frame
                + (prog * frame_span.number_of_frames as f32).floor() as usize
                    % frame_span.number_of_frames;
        } else {
            frame = frame_span.start_frame;
        }

        let frame = self.sprite.frames[frame];

        draw_texture_ex(
            &self.sprite.tex,
            r#box.center().x - self.sprite.frame_w / 2.0,
            r#box.bottom() - self.sprite.frame_h,
            WHITE,
            DrawTextureParams {
                source: Some(Rect::new(
                    frame.0,
                    frame.1,
                    self.sprite.frame_w,
                    self.sprite.frame_h,
                )),
                flip_x,
                ..Default::default()
            },
        );
    }
}
