use color_eyre;
use device_query::{DeviceQuery, DeviceState};
use inject::inject_payload;
use macroquad::miniquad::window::set_window_size;
use macroquad::prelude::*;
use macroquad::ui::{hash, root_ui, widgets};
use ui_state::UIState;

mod inject;
mod ui_state;

#[macroquad::main("SKTTMP")]
async fn main() -> color_eyre::Result<()> {
    set_window_size(500, 250);

    let mut ui_state = UIState::default();

    let device_state = DeviceState::new();

    while !is_quit_requested() {
        clear_background(WHITE);
        draw_ui(&mut ui_state, &device_state);
        next_frame().await;
    }

    Ok(())
}

fn draw_ui(ui_state: &mut UIState, device_state: &DeviceState) {
    if ui_state.wait_for_input {
        widgets::Window::new(hash!(), vec2(50., 0.), vec2(400., 250.))
            .label(&format!(
                "WAITING TO BIND ACTION: {}",
                ui_state.last_bind_action
            ))
            .titlebar(true)
            .movable(false)
            .ui(&mut *root_ui(), |ui| {
                if ui.button(vec2(0., 215.), "CANCEL") {
                    ui_state.wait_for_input = false;
                }
            });
        let keys = device_state.get_keys();
        if keys.len() != 0 {
            map_key_to_action(keys[0], ui_state);
            ui_state.wait_for_input = false;
        }
        return;
    }

    widgets::Window::new(hash!(), vec2(0., 0.), vec2(300., 150.))
        .label("Connection Settings")
        .titlebar(true)
        .movable(false)
        .ui(&mut *root_ui(), |ui| {
            ui.input_text(hash!(), "PROCESS NAME", &mut ui_state.proc_name);
            ui.input_text(hash!(), "PAYLOAD PATH", &mut ui_state.payload_path);
            ui.input_text(hash!(), "LOCAL PLAYER", &mut ui_state.local_player);
            ui.input_text(hash!(), "LOCAL PORT", &mut ui_state.local_port);
            ui.input_text(hash!(), "REMOTE IP:PORT", &mut ui_state.remote_addrs);

            if ui.button(vec2(10., 110.), "Connect") {}
            if ui.button(vec2(150., 110.), "Start Netplay") {
                if let Err(err) = inject_payload(&ui_state.proc_name, &ui_state.payload_path) {
                    println!("{}", err);
                }
            }
        });

    widgets::Window::new(hash!(), vec2(300., 0.), vec2(200., 150.))
        .label("Input Bindings")
        .titlebar(true)
        .movable(false)
        .ui(&mut *root_ui(), |ui| {
            ui.label(vec2(60., 0.), &format!("{:?}", ui_state.keymap.up));
            ui.label(vec2(60., 25.), &format!("{:?}", ui_state.keymap.down));
            ui.label(vec2(60., 50.), &format!("{:?}", ui_state.keymap.left));
            ui.label(vec2(60., 75.), &format!("{:?}", ui_state.keymap.right));

            if ui.button(vec2(10., 0.), "UP") {
                ui_state.wait_for_input = true;
                ui_state.last_bind_action = String::from("UP");
            }
            if ui.button(vec2(10., 25.), "DOWN") {
                ui_state.wait_for_input = true;
                ui_state.last_bind_action = String::from("DOWN");
            }
            if ui.button(vec2(10., 50.), "LEFT") {
                ui_state.wait_for_input = true;
                ui_state.last_bind_action = String::from("LEFT");
            }
            if ui.button(vec2(10., 75.), "RIGHT") {
                ui_state.wait_for_input = true;
                ui_state.last_bind_action = String::from("RIGHT");
            }
        });

    widgets::Window::new(hash!(), vec2(0., 150.), vec2(500., 100.))
        .label("Logging")
        .titlebar(true)
        .movable(false)
        .ui(&mut *root_ui(), |_ui| {});
}

fn map_key_to_action(key: device_query::Keycode, ui_state: &mut UIState) {
    match ui_state.last_bind_action.as_str() {
        "UP" => ui_state.keymap.up = key,
        "DOWN" => ui_state.keymap.down = key,
        "LEFT" => ui_state.keymap.left = key,
        "RIGHT" => ui_state.keymap.right = key,
        _ => println!("Invalid Action."),
    }
}
