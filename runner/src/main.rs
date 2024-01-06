use std::net::TcpListener;

use dll_syringe::{Syringe, process::OwnedProcess};
use color_eyre;
use tracing::metadata::LevelFilter;
use tracing::info;

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::INFO)
        .init();
    // find target process by name
    info!("Finding target process called ShovelKnight");
    let target = {
        let this = OwnedProcess::find_first_by_name("ShovelKnight");
        match this {
            Some(process) => process,
            None => panic!("Shovel Knight is not running")
        }
    };
    info!("Found process called ShovelKnight");
    // creating socket for communication
    info!("Creating communication socket");
    let listen = match TcpListener::bind("127.0.0.1:7788") {
        Ok(it) => it,
        Err(err) => panic!("Failed to create communication socket err: {}", err),
    };
    info!("Communication socket created");
    // create a new syringe for the target process
    let syringe = Syringe::for_process(target);
    info!("Injecting the payload into the target process");
    // inject the payload into the target process
    let _payload = {
        let this = syringe.inject("./../payload/target/release/payload.dll");
        match this {
            Ok(t) => t,
            Err(e) => panic!("Failed to inject payload. err: {}", e)
        }
    };
    info!("Injecting Successful!");
    let (_stream, addr) = match listen.accept() {
        Ok(it) => it,
        Err(err) => panic!("{}", err),
    };
    info!("{} Accepted!", &addr);
    Ok(())
}
