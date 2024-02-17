use std::sync::Mutex;
use std::net::{TcpStream, SocketAddr};
use tracing::info;

use crate::comm::run_comms;

mod mem;
mod comm;
mod input;

#[ctor::ctor]
fn init() {
    let addrs = [
        SocketAddr::from(([127, 0, 0, 1], 6677)),
        SocketAddr::from(([127, 0, 0, 1], 7788)),
    ];
    let stream = TcpStream::connect(&addrs[..]).unwrap();

    stream.set_nodelay(true).unwrap();
    stream.set_nonblocking(true).unwrap();

    tracing_subscriber::fmt()
        .with_writer(Mutex::new(stream))
        .init();

    info!("Starting Comms");
    run_comms();
}
