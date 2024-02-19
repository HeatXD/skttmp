use dll_syringe::{process::OwnedProcess, Syringe};
use tracing::info;

pub fn inject_payload(proc_name: &String, payload_path: &String) -> Result<(), String> {
    // find target process by name
    info!("Finding target process called {}", &proc_name);
    let target = {
        let this = OwnedProcess::find_first_by_name(&proc_name);
        match this {
            Some(process) => process,
            None => return Err(std::format!("{} is not running", &proc_name)),
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
            Err(e) => return Err(std::format!("Failed to inject payload. err: {}", e)),
        }
    };
    info!("Injecting Successful!");
    Ok(())
}