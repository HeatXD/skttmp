use std::collections::HashMap;

use process_memory::{DataMember, Memory, Pid, TryIntoProcessHandle};
use retour::GenericDetour;
use tracing::info;
use winapi::um::libloaderapi::GetProcAddress;

use crate::mem::{get_base_addr_and_hook, get_mem_locations, MemoryItem};

// techincally quite bad practice but we doin it..
static mut DETOUR: Option<GenericDetour<extern "cdecl" fn(u32)>> = None;
static mut MEMBERS: Option<HashMap<MemoryItem, DataMember<f32>>> = None;

pub fn run_comms() {
    if let Some((base_address, module_handle)) = get_base_addr_and_hook() {
        info!("Base address is 0x{:X}", base_address);

        unsafe {
            // try to get the sdl_delay hook.. at some point hook into sdl_pollevent instead
            let func_name: Vec<u8> = "SDL_Delay\0".as_bytes().to_vec();
            let sdl_delay = GetProcAddress(module_handle, func_name.as_ptr() as *const i8);
            info!("fn addr: 0x{:X}", (sdl_delay as usize));
            let sdl_delay: extern "cdecl" fn(u32) = std::mem::transmute(sdl_delay);
            if let Ok(hook) = GenericDetour::new(sdl_delay, sdl_delay_hooked) {
                if let Ok(_) = hook.enable() {
                    info!("Hooked!");
                    DETOUR = Some(hook);
                    info!("Collecting Memory items");
                    let mut members: HashMap<MemoryItem, DataMember<f32>> = HashMap::new();
                    let handle = (std::process::id() as Pid)
                        .try_into_process_handle()
                        .unwrap();
                    let mem_items = get_mem_locations(base_address);
                    members.insert(
                        MemoryItem::P1PosX,
                        DataMember::<f32>::new_offset(
                            handle,
                            mem_items.get(&MemoryItem::P1PosX).unwrap().1.clone(),
                        ),
                    );
                    members.insert(
                        MemoryItem::P1PosY,
                        DataMember::<f32>::new_offset(
                            handle,
                            mem_items.get(&MemoryItem::P1PosY).unwrap().1.clone(),
                        ),
                    );
                    members.insert(
                        MemoryItem::P2PosX,
                        DataMember::<f32>::new_offset(
                            handle,
                            mem_items.get(&MemoryItem::P2PosX).unwrap().1.clone(),
                        ),
                    );
                    members.insert(
                        MemoryItem::P2PosY,
                        DataMember::<f32>::new_offset(
                            handle,
                            mem_items.get(&MemoryItem::P2PosY).unwrap().1.clone(),
                        ),
                    );
                    MEMBERS = Some(members);
                }
            } else {
                info!("Hook Failed!");
            }
        }
    }
}

extern "cdecl" fn sdl_delay_hooked(ms: u32) {
    unsafe {
        DETOUR.as_mut().unwrap().call(ms);
        if let Some(members) = MEMBERS.as_ref() {
            if let Ok(xpos) = members.get(&MemoryItem::P1PosX).unwrap().read() {
                if let Ok(ypos) = members.get(&MemoryItem::P1PosY).unwrap().read() {
                    info!("PX:{},PY:{}", xpos, ypos);
                }
            }
        }
    }
}
