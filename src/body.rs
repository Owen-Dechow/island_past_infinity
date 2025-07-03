use macroquad::{
    color::BLUE,
    math::{vec2, Rect, Vec2},
    shapes::draw_rectangle,
};

use crate::{
    animator::Animator, levels::Level, sprites::Sprite, world::World, TILE_COLLISION_SECTIONS, TILE_SIZE
};

pub struct Body {
    pub hitbox: Rect,
    animator: Option<Animator>,
}

impl Body {
    pub fn new(x: f32, y: f32, w: f32, h: f32, sprite: Option<Sprite>) -> Self {
        let x = x - w / 2.0;
        let y = y - h / 2.0;

        return Self {
            hitbox: Rect::new(x, y, w, h),
            animator: match sprite {
                Some(sprite) => Some(Animator::new(sprite)),
                None => None,
            },
        };
    }

    pub fn screen_x(&self, world: &World) -> f32 {
        self.hitbox.x - world.x
    }

    pub fn screen_y(&self, world: &World) -> f32 {
        self.hitbox.y - world.y
    }

    pub fn get_y_sort_key(&self) -> i32 {
        (self.hitbox.bottom() * 100.0) as i32
    }

    pub fn r#move(&mut self, delta: Vec2, level: &Level, dt: f32) {
        if let Some(ref mut animator) = self.animator {
            animator.apply_delta(delta, dt);
        }

        let delta = delta * dt;
        self.hitbox.x += delta.x;
        let mut vert_check_point = self.hitbox.y;
        loop {
            let bottom = self.hitbox.y + self.hitbox.h;
            if vert_check_point > bottom {
                vert_check_point = bottom;
            }

            if delta.x > 0.0 {
                let right = self.hitbox.x + self.hitbox.w;
                if let Some(collision_point) = level.check_for_collision(right, vert_check_point) {
                    self.hitbox.x = collision_point.from_left() - self.hitbox.w;
                    break;
                }
            } else {
                if let Some(collision_point) =
                    level.check_for_collision(self.hitbox.x, vert_check_point)
                {
                    self.hitbox.x = collision_point.from_right();
                    break;
                }
            }

            if vert_check_point == bottom {
                break;
            } else {
                vert_check_point += TILE_SIZE / TILE_COLLISION_SECTIONS;
            }
        }

        self.hitbox.y += delta.y;
        let mut horizontal_check_point = self.hitbox.x;
        loop {
            let right = self.hitbox.x + self.hitbox.w;
            let bottom = self.hitbox.y + self.hitbox.h;

            if horizontal_check_point > right {
                horizontal_check_point = right;
            }

            if delta.y > 0.0 {
                if let Some(collision_info) =
                    level.check_for_collision(horizontal_check_point, bottom)
                {
                    self.hitbox.y = collision_info.from_top() - self.hitbox.h;
                    break;
                }
            } else {
                if let Some(collision_info) =
                    level.check_for_collision(horizontal_check_point, self.hitbox.y)
                {
                    self.hitbox.y = collision_info.from_bottom();
                    break;
                }
            }

            if horizontal_check_point == right {
                break;
            } else {
                horizontal_check_point += TILE_SIZE / TILE_COLLISION_SECTIONS;
            }
        }
    }

    pub fn render(&self, world: &World) {
        if let Some(animator) = &self.animator {
            let screen_box = self.hitbox.offset(-vec2(world.x, world.y));
            animator.render(&screen_box)
        } else {
            draw_rectangle(
                self.screen_x(world),
                self.screen_y(world),
                self.hitbox.w,
                self.hitbox.h,
                BLUE,
            );
        }
    }
}
