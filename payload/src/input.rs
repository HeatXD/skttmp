use std::rc::Rc;
use tracing::info;
use vigem_client::{Client, TargetId, XButtons, XGamepad, Xbox360Wired};

const BUFFER_SIZE: i32 = 64;
const DELAY: i32 = 5;

pub struct GameInput {
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

pub struct GameInputQueue {
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
            info!("inp frame: {}", inp.frame);
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
    pad_client: Rc<Client>,
    p1_pad: Xbox360Wired<Rc<Client>>,
    p2_pad: Xbox360Wired<Rc<Client>>,
    p1_buffer: GameInputQueue,
    p2_buffer: GameInputQueue,
    first_time: bool,
    init_sequence: InputSequence,
}

impl Default for GameInputHandler {
    fn default() -> Self {
        let client = Rc::new(vigem_client::Client::connect().unwrap());
        let mut pad1 = vigem_client::Xbox360Wired::new(client.clone(), TargetId::XBOX360_WIRED);
        let mut pad2 = vigem_client::Xbox360Wired::new(client.clone(), TargetId::XBOX360_WIRED);

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
            pad_client: client,
            p1_pad: pad1,
            p2_pad: pad2,
            first_time: true,
            init_sequence: InputSequence::init_game_seq(),
        }
    }
}

impl GameInputHandler {
    pub fn on_hook_call(&mut self, ms: u32) -> u32 {
        self.current_frame += 1;
        if self.first_time {
            self.do_init_sequence();
        }
        if self.current_frame >= 0 && !self.first_time {
            self.wait_input_available();
            self.apply_input();
        }
        info!("C:{}/{}", self.current_frame, self.current_frame / 60);
        ms
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

    fn wait_input_available(&self) {
        let frame = self.current_frame;
        loop {
            if self.p1_buffer.has_input(frame) && self.p2_buffer.has_input(frame) {
                break;
            }
            info!("SYNCING FRAME:{}", frame);
        }
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
