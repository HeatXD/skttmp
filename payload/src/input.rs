use std::{rc::Rc, thread, time::Duration};
use tracing::info;
use vigem_client::{Client, TargetId, XButtons, XGamepad, Xbox360Wired};

const BUFFER_SIZE: i32 = 64;
const DELAY: i32 = 5;
const MAX_WAIT_TIME: u32 = 60 * 4 * 10; // 10 secs

struct GameInput {
    frame: i32,
    input: u16,
}

impl GameInput {
    fn default() -> GameInput {
        Self {
            frame: -1,
            input: 0,
        }
    }
}

struct GameInputQueue {
    queue: Vec<GameInput>,
}

impl GameInputQueue {
    fn new() -> GameInputQueue {
        let mut q = Vec::<GameInput>::new();
        // fill buffer with empty inputs
        for _ in 0..BUFFER_SIZE {
            q.push(GameInput::default());
        }
        // fill the first few delay frames
        for i in 0..DELAY {
            if let Some(inp) = q.get_mut(i as usize) {
                inp.frame = i;
            }
        }
        Self { queue: q }
    }

    fn has_input(&self, frame: i32) -> bool {
        if let Some(inp) = self.queue.get((frame % BUFFER_SIZE) as usize) {
            return frame == inp.frame;
        }
        false
    }

    fn get_input(&self, frame: i32) -> u16 {
        if let Some(inp) = self.queue.get((frame % BUFFER_SIZE) as usize) {
            return inp.input;
        }
        0
    }
}

pub struct GameInputHandler {
    current_frame: i32,
    p1_pad: Xbox360Wired<Rc<Client>>,
    p2_pad: Xbox360Wired<Rc<Client>>,
    p1_buffer: GameInputQueue,
    p2_buffer: GameInputQueue,
    first_time: bool,
    init_sequence: InputSequence,
}

impl Default for GameInputHandler {
    fn default() -> Self {
        if let Ok(client) = vigem_client::Client::connect() {
            let cli = Rc::new(client);
            let mut pad1 = vigem_client::Xbox360Wired::new(cli.clone(), TargetId::XBOX360_WIRED);
            let mut pad2 = vigem_client::Xbox360Wired::new(cli.clone(), TargetId::XBOX360_WIRED);

            pad1.plugin().unwrap();
            pad1.wait_ready().unwrap();
            info!("P1 PAD CONNECTED");

            pad2.plugin().unwrap();
            pad2.wait_ready().unwrap();
            info!("P2 PAD CONNECTED");

            Self {
                current_frame: -1,
                p1_buffer: GameInputQueue::new(),
                p2_buffer: GameInputQueue::new(),
                p1_pad: pad1,
                p2_pad: pad2,
                first_time: true,
                init_sequence: InputSequence::init_game_seq(),
            }
        } else {
            info!("Failed to init GameInputHandler");
            panic!("Failed to init GameInputHandler");
        }
    }
}

impl GameInputHandler {
    pub fn on_hook_call(&mut self, ms: u32) -> u32 {
        let mut wait_time = ms;
        self.current_frame += 1;

        if self.first_time {
            self.do_init_sequence();
        }

        if self.current_frame >= 0 && !self.first_time {
            wait_time = self.wait_input_available(ms);
            self.apply_input();
        }
        // info!("C:{}/{}", self.current_frame, self.current_frame / 60);
        wait_time
    }

    fn do_init_sequence(&mut self) {
        let mut inp = XButtons!();
        let idx = self.init_sequence.seq_index;
        let seq = self.init_sequence.sequence.get(idx);

        if seq.is_none() {
            self.first_time = false;
            return;
        }

        let seq = seq.unwrap();
        let player = seq.player;

        if self.current_frame >= 0 && self.current_frame < seq.hold as i32 {
            inp = seq.input;
        }

        if self.current_frame == (seq.hold + seq.delay_after) as i32 && inp == XButtons!() {
            self.current_frame = -1;
            self.init_sequence.seq_index += 1
        }

        self.update_pad(inp, player);
    }

    fn update_pad(&mut self, btns: XButtons, player: u8) {
        let mut pad = XGamepad::default();
        pad.buttons = btns;
        if player == 0 {
            self.p1_pad.update(&pad).unwrap();
        } else if player == 1 {
            self.p2_pad.update(&pad).unwrap();
        }
    }

    fn wait_input_available(&self, ms: u32) -> u32 {
        let frame = self.current_frame;
        let num_iterations: u32 = 4;
        let mut frame_wait: u32 = 0;
        let mut wait_time: u32 = ms;

        loop {
            if self.p1_buffer.has_input(frame) && self.p2_buffer.has_input(frame) {
                break;
            }

            frame_wait += 1;
            if frame_wait == MAX_WAIT_TIME {
                info!("Failed to Sync (f:{})", frame);
                panic!("Could Not Sync In Time");
            }

            thread::sleep(Duration::from_millis((ms / num_iterations) as u64));
        }

        // return the remaining wait time
        if frame_wait > 0 && frame_wait <= num_iterations {
            wait_time = ms - ((ms / num_iterations) * frame_wait);
        }

        // we are spilling over our frame time make sure that we dont add extra delay
        if frame_wait > num_iterations {
            wait_time = 0;
        }

        wait_time
    }

    fn apply_input(&mut self) {
        let frame = self.current_frame;
        let p1 = self.p1_buffer.get_input(frame);
        let p2 = self.p2_buffer.get_input(frame);

        self.update_pad(XButtons { raw: p1 }, 0);
        self.update_pad(XButtons { raw: p2 }, 1);
    }
}

struct SeqInput {
    input: XButtons,
    hold: u32,
    delay_after: u32,
    player: u8,
}

impl SeqInput {
    fn new(btns: XButtons, hold: u32, player: u8, delay_after: u32) -> SeqInput {
        Self {
            input: btns,
            hold,
            player,
            delay_after,
        }
    }
}

struct InputSequence {
    sequence: Vec<SeqInput>,
    seq_index: usize,
}

impl InputSequence {
    fn init_game_seq() -> InputSequence {
        let seq: Vec<SeqInput> = vec![
            // Starting on the Main Menu at "start game"
            // Select Empty Save Slot #20
            SeqInput::new(XButtons!(A), 5, 0, 0),
            SeqInput::new(XButtons!(A), 5, 0, 0),
            SeqInput::new(XButtons!(A), 5, 0, 0),
            SeqInput::new(XButtons!(A), 5, 0, 180),
            SeqInput::new(XButtons!(LEFT), 1, 0, 0),
            SeqInput::new(XButtons!(LEFT), 1, 0, 0),
            SeqInput::new(XButtons!(LEFT), 1, 0, 0),
            SeqInput::new(XButtons!(A), 1, 0, 0),
            SeqInput::new(XButtons!(LEFT), 1, 0, 0),
            SeqInput::new(XButtons!(A), 1, 0, 30),
            SeqInput::new(XButtons!(A), 1, 0, 240),
            // SPELL NTPLY
            SeqInput::new(XButtons!(RIGHT), 1, 0, 0),
            SeqInput::new(XButtons!(RIGHT), 1, 0, 0),
            SeqInput::new(XButtons!(DOWN), 1, 0, 0),
            SeqInput::new(XButtons!(A), 1, 0, 0),
            SeqInput::new(XButtons!(LEFT), 1, 0, 0),
            SeqInput::new(XButtons!(LEFT), 1, 0, 0),
            SeqInput::new(XButtons!(LEFT), 1, 0, 0),
            SeqInput::new(XButtons!(LEFT), 1, 0, 0),
            SeqInput::new(XButtons!(LEFT), 1, 0, 0),
            SeqInput::new(XButtons!(A), 1, 0, 0),
            SeqInput::new(XButtons!(LEFT), 1, 0, 0),
            SeqInput::new(XButtons!(LEFT), 1, 0, 0),
            SeqInput::new(XButtons!(LEFT), 1, 0, 0),
            SeqInput::new(XButtons!(LEFT), 1, 0, 0),
            SeqInput::new(XButtons!(A), 1, 0, 0),
            SeqInput::new(XButtons!(LEFT), 1, 0, 0),
            SeqInput::new(XButtons!(LEFT), 1, 0, 0),
            SeqInput::new(XButtons!(LEFT), 1, 0, 0),
            SeqInput::new(XButtons!(LEFT), 1, 0, 0),
            SeqInput::new(XButtons!(A), 1, 0, 0),
            SeqInput::new(XButtons!(DOWN), 1, 0, 0),
            SeqInput::new(XButtons!(RIGHT), 1, 0, 0),
            SeqInput::new(XButtons!(RIGHT), 1, 0, 0),
            SeqInput::new(XButtons!(A), 1, 0, 0),
            SeqInput::new(XButtons!(DOWN), 1, 0, 0),
            SeqInput::new(XButtons!(LEFT), 1, 0, 0),
            SeqInput::new(XButtons!(LEFT), 1, 0, 0),
            SeqInput::new(XButtons!(LEFT), 1, 0, 0),
            SeqInput::new(XButtons!(DOWN), 1, 0, 0),
            SeqInput::new(XButtons!(A), 1, 0, 4),
            // Say NO to body swap
            SeqInput::new(XButtons!(A), 1, 0, 0),
            // Enable coop
            SeqInput::new(XButtons!(BACK), 1, 0, 20),
            // Set P1 Controller
            SeqInput::new(XButtons!(A), 100, 0, 10),
            // Set P2 Controller
            SeqInput::new(XButtons!(A), 100, 1, 20),
            // Confirm to play on save
            SeqInput::new(XButtons!(A), 2, 0, 0),
            SeqInput::new(XButtons!(A), 2, 0, 0),
        ];
        Self {
            seq_index: 0,
            sequence: seq,
        }
    }
}
