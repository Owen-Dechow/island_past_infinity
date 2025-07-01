use macroquad::input::{
    is_key_down, is_key_pressed, is_mouse_button_down, is_mouse_button_pressed,
    mouse_position_local, mouse_wheel, KeyCode, MouseButton,
};

pub struct Input {
    pub vertical: f32,
    pub horizontal: f32,
    pub toggle_editor: bool,
    pub scroll: f32,
    pub mouse_x: f32,
    pub mouse_y: f32,
    pub click: bool,
    pub mouse_down: bool,
    pub enter: bool,
}

impl Input {
    pub fn get() -> Input {
        let vertical = match (
            is_key_down(KeyCode::Up) || is_key_down(KeyCode::W),
            is_key_down(KeyCode::Down) || is_key_down(KeyCode::S),
        ) {
            (true, false) => -1.0,
            (false, true) => 1.0,
            _ => 0.0,
        };

        let horizontal = match (
            is_key_down(KeyCode::Left) || is_key_down(KeyCode::A),
            is_key_down(KeyCode::Right) || is_key_down(KeyCode::D),
        ) {
            (true, false) => -1.0,
            (false, true) => 1.0,
            _ => 0.0,
        };

        let toggle_editor = is_key_pressed(KeyCode::P);
        let scroll = mouse_wheel().1;
        let mpos = mouse_position_local();

        let click = is_mouse_button_pressed(MouseButton::Left);
        let mouse_down = is_mouse_button_down(MouseButton::Left);

        Input {
            vertical,
            horizontal,
            toggle_editor,
            scroll,
            mouse_x: mpos.x,
            mouse_y: mpos.y,
            click,
            mouse_down,
            enter: is_key_down(KeyCode::Enter),
        }
    }
}
