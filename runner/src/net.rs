use std::{net::SocketAddr, thread};

use crate::ui_state::UIState;
use crossbeam::channel::{Receiver, Sender};
use device_query::DeviceState;
use laminar::{Packet, Socket, SocketEvent};

pub struct NetMgr {
    sender: Sender<Packet>,
    events: Receiver<SocketEvent>,
    remotes: Vec<SocketAddr>,
}

impl NetMgr {
    pub fn init(ui_state: &mut UIState) -> Self {
        if let Ok(mut socket) = Socket::bind_any() {
            ui_state.last_logs.add(format!(
                "LOCAL UDP SOCK @ {:?}",
                socket.local_addr().unwrap()
            ));
            let sender = socket.get_packet_sender();
            let events = socket.get_event_receiver();

            // poll in the background
            thread::spawn(move || socket.start_polling());

            return Self {
                sender,
                events,
                remotes: Vec::new(),
            };
        }
        panic!("Failed to bind local socket")
    }

    pub fn handle_events(&mut self, ui_state: &mut UIState) {
        while let Ok(event) = self.events.try_recv() {
            match event {
                SocketEvent::Packet(pkt) => {
                    println!("Packet from :{}", &pkt.addr());
                    if !self.remotes.contains(&pkt.addr()) {
                        let pack = Packet::unreliable(pkt.addr(), b"A".to_vec());
                        if let Err(err) = self.sender.send(pack) {
                            ui_state.last_logs.add(format!("Send error: {}", err));
                        }
                    }
                }
                SocketEvent::Connect(addr) => {
                    ui_state.last_logs.add(format!("Connect from :{}", addr));
                    if !self.remotes.contains(&addr) {
                        self.remotes.push(addr);
                    }
                }
                SocketEvent::Timeout(addr) => {
                    ui_state.last_logs.add(format!("Timeout from :{}", addr));
                }
                SocketEvent::Disconnect(addr) => {
                    ui_state.last_logs.add(format!("Disconnect from :{}", addr));
                    self.remotes.retain(|addrs| &addr != addrs);
                }
            }
        }
    }

    pub fn send_input(&self, device_state: &DeviceState) {
        //todo!()
    }
}


enum NetPacket {
    Input(NetInput),
    Log(String),
}

struct NetInput {
    frame: i32,
    player: u8,
    input: u16,
}

