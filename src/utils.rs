use macroquad::{
    camera::set_default_camera,
    color::DARKGRAY,
    input::{is_key_pressed, KeyCode},
    ui::{hash, root_ui},
    window::{clear_background, next_frame},
};

pub async fn prompt(text: &str) -> Option<String> {
    next_frame().await;
    let mut input_text = String::new();

    loop {
        set_default_camera();
        clear_background(DARKGRAY);

        let hash = hash!();
        root_ui().label(None, text);
        root_ui().input_text(hash, "", &mut input_text);
        root_ui().set_input_focus(hash);

        if root_ui().button(None, "Submit") || is_key_pressed(KeyCode::Enter) {
            return Some(input_text);
        }
        if root_ui().button(None, "Cancel") {
            return None;
        }

        next_frame().await;
    }
}

pub async fn alert(text: &str) {
    next_frame().await;

    loop {
        set_default_camera();
        clear_background(DARKGRAY);
        root_ui().label(None, text);

        if root_ui().button(None, "Ok") || is_key_pressed(KeyCode::Enter) {
            return;
        }

        next_frame().await;
    }
}

pub fn splitter() {
    root_ui().label(None, &"-".repeat(20))
}
