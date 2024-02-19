use std::{collections::VecDeque, time::Instant};

const LOG_SIZE: usize = 32;

pub struct Logs {
    history: VecDeque<String>,
    startup_time: Instant,
}

impl Default for Logs {
    fn default() -> Self {
        Self {
            history: VecDeque::new(),
            startup_time: Instant::now(),
        }
    }
}

impl Logs {
    pub fn add(&mut self, msg: String) {
        let trim_msg = msg.trim();

        if trim_msg.is_empty() {
            return;
        }

        if self.history.len() == LOG_SIZE {
            self.history.pop_back();
        }

        self.history.push_front(format!(
            "{} | {}",
            self.startup_time.elapsed().as_secs(),
            trim_msg
        ));
    }

    pub fn clear(&mut self) {
        self.startup_time = Instant::now();
        self.history.clear()
    }

    pub fn get_all(&self) -> &VecDeque<String> {
        &self.history
    }
}
