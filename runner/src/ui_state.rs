use device_query::Keycode;

pub struct UIState {
    pub proc_name: String,
    pub payload_path: String,
    pub local_player: String,
    pub local_port: String,
    pub remote_addrs: String,
    pub keymap: InputMap,
    pub wait_for_input: bool,
    pub last_bind_action: String,
}

impl Default for UIState {
    fn default() -> Self {
        Self {
            local_player: String::default(),
            local_port: String::default(),
            remote_addrs: String::default(),
            proc_name: String::from("ShovelKnight"),
            payload_path: String::from("./../payload/target/debug/payload.dll"),
            keymap: InputMap::default(),
            wait_for_input: false,
            last_bind_action: String::default(),
        }
    }
}

pub struct InputMap {
    pub up: Keycode,
    pub down: Keycode,
    pub left: Keycode,
    pub right: Keycode,
}

impl Default for InputMap {
    fn default() -> Self {
        Self {
            up: Keycode::W,
            down: Keycode::S,
            left: Keycode::A,
            right: Keycode::D,
        }
    }
}
