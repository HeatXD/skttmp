use dll_syringe::{process::OwnedProcess, Syringe};

use crate::ui_state::UIState;

pub fn inject_payload(ui_state: &mut UIState) -> Result<(), String> {
    let proc_name = &ui_state.proc_name;
    let payload_path = &ui_state.payload_path;
    // find target process by name
     ui_state.last_logs.add(format!("Finding target process called {}", proc_name));
    let target = {
        let this = OwnedProcess::find_first_by_name(proc_name);
        match this {
            Some(process) => process,
            None => return Err(std::format!("{} is not running", proc_name)),
        }
    };
    ui_state.last_logs.add(format!("Found process called {}", proc_name));
    // create a new syringe for the target process
    let syringe = Syringe::for_process(target);
    ui_state.last_logs.add("Injecting the payload into the target process".to_string());
    // inject the payload into the target process
    let _payload = {
        let this = syringe.inject(payload_path);
        match this {
            Ok(t) => t,
            Err(e) => return Err(std::format!("Failed to inject payload. err: {}", e)),
        }
    };
    ui_state.last_logs.add("Injecting Successful!".to_string());
    Ok(())
}