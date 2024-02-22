#![windows_subsystem = "windows"]

use device_query::{DeviceQuery, DeviceState};
use inject::inject_payload;
use macroquad::color::*;
use macroquad::input::is_quit_requested;
use macroquad::math::vec2;
use macroquad::miniquad::window::set_window_size;
use macroquad::ui::{hash, root_ui, widgets};
use macroquad::window::{clear_background, next_frame};
use net::NetMgr;
use ui_state::UIState;

mod inject;
mod log;
mod net;
mod ui_state;

#[macroquad::main("SKTTMP")]
async fn main() {
    set_window_size(500, 250);
    let mut ui_state = UIState::default();
    let device_state = DeviceState::new();
    let mut net_mgr = NetMgr::init(&mut ui_state);

    while !is_quit_requested() {
        handle_network(&mut net_mgr, &device_state, &mut ui_state);
        clear_background(WHITE);
        draw_ui(&mut ui_state, &device_state);
        next_frame().await;
    }
}

fn handle_network(net_mgr: &mut NetMgr, device_state: &DeviceState, ui_state: &mut UIState) {
    net_mgr.handle_events(ui_state);
    net_mgr.send_input(device_state);
}

fn draw_ui(ui_state: &mut UIState, device_state: &DeviceState) {
    if ui_state.wait_for_input {
        widgets::Window::new(hash!(), vec2(0., 0.), vec2(500., 250.))
            .label(&format!(
                "WAITING TO BIND ACTION: {}",
                ui_state.last_bind_action
            ))
            .titlebar(true)
            .movable(false)
            .ui(&mut *root_ui(), |ui| {
                if ui.button(vec2(0., 215.), "Cancel") {
                    ui_state.wait_for_input = false;
                }
            });
        let keys = device_state.get_keys();
        if let Some(key) = keys.first() {
            map_key_to_action(key, ui_state);
            ui_state.wait_for_input = false;
        };
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
            ui.input_text(hash!(), "REMOTE IP:PORT", &mut ui_state.remote_addrs);

            if ui.button(vec2(10., 100.), "Connect") {
                ui_state.last_logs.clear();
            }
            if ui.button(vec2(150., 100.), "Start Netplay") {
                if let Err(err) = inject_payload(ui_state) {
                    ui_state.last_logs.add(format!("{}", err));
                }
            }
        });

    widgets::Window::new(hash!(), vec2(300., 0.), vec2(200., 150.))
        .label("Input Bindings")
        .titlebar(true)
        .movable(false)
        .ui(&mut *root_ui(), |ui| {
            ui.label(vec2(60., 0.), &format!("= {:?}", ui_state.keymap.up));
            ui.label(vec2(60., 25.), &format!("= {:?}", ui_state.keymap.down));
            ui.label(vec2(60., 50.), &format!("= {:?}", ui_state.keymap.left));
            ui.label(vec2(60., 75.), &format!("= {:?}", ui_state.keymap.right));

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
        .ui(&mut *root_ui(), |ui| {
            for log in ui_state.last_logs.get_all() {
                ui.label(None, log);
            }
        });
}

fn map_key_to_action(key: &device_query::Keycode, ui_state: &mut UIState) {
    let action = ui_state.last_bind_action.as_str();
    match action {
        "UP" => ui_state.keymap.up = *key,
        "DOWN" => ui_state.keymap.down = *key,
        "LEFT" => ui_state.keymap.left = *key,
        "RIGHT" => ui_state.keymap.right = *key,
        _ => println!("Invalid Action."),
    }
    ui_state
        .last_logs
        .add(format!("{} has been binded to {:?}", action, key));
}
