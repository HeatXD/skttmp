use crate::comm::run_comms;

mod mem;
mod comm;
mod input;

#[ctor::ctor]
fn init() {
    run_comms();
}
