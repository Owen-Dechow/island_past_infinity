mod input;
mod player;
mod resources;
mod utils;
mod world;
mod animator;
mod object;

use input::Input;
use macroquad::{
    camera::{set_camera, set_default_camera, Camera2D},
    color::{BLACK, WHITE},
    math::{vec2, Rect},
    miniquad::conf::Platform,
    texture::{draw_texture_ex, render_target, DrawTextureParams, RenderTarget},
    time::{draw_fps, get_frame_time},
    window::{clear_background, next_frame, screen_height, screen_width, Conf},
};
use player::Player;
use resources::levels::LevelEditorSettings;
use world::World;

use crate::resources::levels::Level;

const TILE_SIZE: f32 = 16 as f32;
const TILE_COLLISION_SECTIONS: f32 = 3 as f32;
const VIRTUAL_W: f32 = TILE_SIZE * 24 as f32;
const VIRTUAL_H: f32 = TILE_SIZE * 16 as f32;
const SUB_PIX_LEVEL: f32 = 3 as f32;

fn window_config() -> Conf {
    let window_scale = 3;

    Conf {
        window_title: "Island Past Infinity".to_owned(),
        window_width: VIRTUAL_W as i32 * window_scale,
        window_height: VIRTUAL_H as i32 * window_scale,
        window_resizable: false,
        fullscreen: false,
        platform: Platform {
            swap_interval: Some(0),
            ..Default::default()
        },
        ..Default::default()
    }
}

fn get_render_target(vw: u32, vh: u32) -> RenderTarget {
    return render_target(vw, vh);
}

#[macroquad::main(window_config)]
async fn main() {
    let mut world = World::new();
    let mut player = Player::new(&world).await.unwrap();
    let mut level = Level::load("beach").await.unwrap();

    let render_target = get_render_target(
        (VIRTUAL_W * SUB_PIX_LEVEL) as u32,
        (VIRTUAL_H * SUB_PIX_LEVEL) as u32,
    );

    render_target
        .texture
        .set_filter(macroquad::texture::FilterMode::Linear);

    let mut editor = LevelEditorSettings::new();

    loop {
        let dt = get_frame_time();
        let input = Input::get();

        if input.toggle_editor {
            editor.toggle();
        }

        if !editor.open || input.mouse_x > -0.33 {
            player.move_player(&mut level, &input, dt);
        }

        world.x += (player.obj.hitbox.center().x - VIRTUAL_W / 2.0 - world.x) * 2.0 * dt;
        world.y += (player.obj.hitbox.center().y - VIRTUAL_H / 2.0 - world.y) * 2.0 * dt;

        let world = world.rounded();

        set_camera(&Camera2D {
            zoom: vec2(2.0 / VIRTUAL_W, 2.0 / VIRTUAL_H),
            target: vec2(VIRTUAL_W / 2.0, VIRTUAL_H / 2.0),
            render_target: Some(render_target.clone()),
            ..Default::default()
        });
        clear_background(BLACK);

        if editor.show_background {
            level.render_background(&world);
        }

        if editor.show_object {
            level.render_object_layer(&world);
        }

        player.obj.render(&world);

        if editor.show_overlay {
            level.render_overlay(&world);
        }

        if editor.open {
            level
                .level_editor(&mut editor, &input, dt, &world)
                .await
                .unwrap();
        }

        set_default_camera();
        draw_texture_ex(
            &render_target.texture,
            0.0,
            0.0,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(screen_width().round(), screen_height().round())),
                source: Some(Rect::new(
                    0.0,
                    0.0,
                    VIRTUAL_W * SUB_PIX_LEVEL,
                    VIRTUAL_H * SUB_PIX_LEVEL,
                )),
                ..Default::default()
            },
        );

        draw_fps();

        next_frame().await;
    }
}
