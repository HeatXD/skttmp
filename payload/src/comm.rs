use retour::GenericDetour;
use tracing::info;
use winapi::um::libloaderapi::GetProcAddress;

use crate::{input::GameInputHandler, mem::{collect_mem_items, get_base_addr_and_hook}};

// techincally quite bad practice but we doin it..
static mut DETOUR: Option<GenericDetour<extern "cdecl" fn(u32)>> = None;
static mut INPUT_HANDLER: Option<GameInputHandler> = None;

pub fn run_comms() {
    if let Some((base_address, module_handle)) = get_base_addr_and_hook() {
        info!("Base address is 0x{:X}", base_address);

        unsafe {
            // try to get the sdl_delay hook.. at some point hook into sdl_pollevent instead
            let func_name: Vec<u8> = "SDL_Delay\0".as_bytes().to_vec();
            let sdl_delay = GetProcAddress(module_handle, func_name.as_ptr() as *const i8);
            info!("Hook address: 0x{:X}", (sdl_delay as usize));
            let sdl_delay: extern "cdecl" fn(u32) = std::mem::transmute(sdl_delay);
            if let Ok(hook) = GenericDetour::new(sdl_delay, sdl_delay_hooked) {
                if let Ok(_) = hook.enable() {
                    info!("Hooked!");
                    DETOUR = Some(hook);
                    collect_mem_items(base_address);
                }
            } else {
                info!("Hook Failed!");
            }

            INPUT_HANDLER = Some(GameInputHandler::default())
        }
    }
}

extern "cdecl" fn sdl_delay_hooked(ms: u32) {
    unsafe {
        if let Some(detour) = DETOUR.as_mut() {
            if let Some(handler) = INPUT_HANDLER.as_mut() {
                let wait = handler.on_hook_call(ms);
                detour.call(wait);
            }
        }
    }
}
