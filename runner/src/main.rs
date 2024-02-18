use color_eyre;
use dll_syringe::{process::OwnedProcess, Syringe};
use macroquad::prelude::*;
use tracing::info;
use tracing::metadata::LevelFilter;

#[macroquad::main("SKTTMP")]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    tracing_subscriber::fmt()
        .with_max_level(LevelFilter::INFO)
        .init();

    while !is_quit_requested() {
        clear_background(BLACK);
        if is_key_pressed(KeyCode::A) {
            inject_payload(
                "ShovelKnight".to_string(),
                "./../payload/target/debug/payload.dll".to_string(),
            );
        }
        next_frame().await;
    }

    Ok(())
}

fn inject_payload(proc_name: String, payload_path: String) {
    // find target process by name
    info!("Finding target process called {}", &proc_name);
    let target = {
        let this = OwnedProcess::find_first_by_name(&proc_name);
        match this {
            Some(process) => process,
            None => panic!("{} is not running", &proc_name),
        }
    };
    info!("Found process called {}", &proc_name);
    // create a new syringe for the target process
    let syringe = Syringe::for_process(target);
    info!("Injecting the payload into the target process");
    // inject the payload into the target process
    let _payload = {
        let this = syringe.inject(&payload_path);
        match this {
            Ok(t) => t,
            Err(e) => panic!("Failed to inject payload. err: {}", e),
        }
    };
    info!("Injecting Successful!");
}
