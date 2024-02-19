use tracing::info;
use winapi::{um::{libloaderapi::GetModuleHandleW, errhandlingapi::GetLastError, psapi::{GetModuleInformation, MODULEINFO}, processthreadsapi::GetCurrentProcess}, shared::minwindef::HINSTANCE__};

pub fn get_base_addr_and_hook() -> Option<(usize, *mut HINSTANCE__)> {

    let path = std::env::current_exe().unwrap();
    let res: Vec<&str> = path.to_str().and_then(|p_str| Some(p_str.split("\\"))).unwrap().collect();
    let exe_name = format!("{}\0", res.last().unwrap());
    // Convert the executable name to a wide string (UTF-16)
    let exe_name_wide: Vec<u16> = exe_name.encode_utf16().chain(Some(0)).collect();
    let module_handle = unsafe { GetModuleHandleW(exe_name_wide.as_ptr()) };

    if module_handle.is_null() {
        info!("Failed to get module handle. Error code: {}", unsafe {
            GetLastError()
        });
        return None;
    }

    let mut module_info: MODULEINFO = unsafe { std::mem::zeroed() };
    if unsafe {
        GetModuleInformation(
            GetCurrentProcess(),
            module_handle,
            &mut module_info,
            std::mem::size_of::<MODULEINFO>() as u32,
        )
    } == 0
    {
        info!("Failed to get module information. Error code: {}", unsafe {
            GetLastError()
        });

        return None;
    }

    let base_address = module_info.lpBaseOfDll as usize;

    Some((base_address, module_handle))
}


//use std::collections::HashMap;

//use process_memory::{DataMember, Pid, TryIntoProcessHandle};

// //this is bad practive but doing it for now.
// static mut MEMBERS: Option<HashMap<MemoryItem, DataMember<f32>>> = None;

// // general exploration of memory addresses

// // later maybe move this to a file based system. that way recompiling isnt needed.
// #[derive(PartialEq, Eq, Hash, Debug)]
// pub enum MemoryItem {
//     GoldShovelPlague, // gold value in Shovel of Hope and Plague of Shadows
//     GoldSpecter,      // gold value in Specter of Torment
//     GoldKing,         // gold value in King of Cards
//     P1PosX,           // player 1 X position
//     P1PosY,           // player 1 y position
//     P2PosX,           // player 2 x position
//     P2PosY,           // player 2 y position
// } 

// pub fn get_mem_locations(base_address: usize) -> HashMap<MemoryItem, Vec<usize>> {
//     let mut res: HashMap<MemoryItem, Vec<usize>> = HashMap::new();
//     // mem position values
//     // adding the memory type just 
//     res.insert(MemoryItem::GoldShovelPlague, vec![base_address + 0x9072CC, 0xEE4]); // uint
//     res.insert(MemoryItem::GoldSpecter,vec![base_address + 0x90727C, 0xEE4]); 
//     res.insert(MemoryItem::GoldKing,  vec![base_address + 0x907B8C, 0xEE4]);
//     res.insert(MemoryItem::P1PosX,  vec![base_address + 0x008DCB34, 0x80, 0x24, 0xC]); // f32
//     res.insert(MemoryItem::P1PosY,  vec![base_address + 0x008DCB34, 0x80, 0x24, 0x10]);
//     res.insert(MemoryItem::P2PosX,  vec![base_address + 0x008D231C, 0x60, 0x24, 0xC]);
//     res.insert(MemoryItem::P2PosY, vec![base_address + 0x008DCB34, 0x80, 0x24, 0x290]);
//     // return mem map
//     res
// }

// pub fn collect_mem_items(base_address: usize) {
//     info!("Collecting Memory items");
//     unsafe {
//         let mut members: HashMap<MemoryItem, DataMember<f32>> = HashMap::new();
//         let handle = (std::process::id() as Pid)
//             .try_into_process_handle()
//             .unwrap();
//         let mem_items = get_mem_locations(base_address);
//         members.insert(
//             MemoryItem::P1PosX,
//             DataMember::<f32>::new_offset(
//                 handle,
//                 mem_items.get(&MemoryItem::P1PosX).unwrap().clone(),
//             ),
//         );
//         members.insert(
//             MemoryItem::P1PosY,
//             DataMember::<f32>::new_offset(
//                 handle,
//                 mem_items.get(&MemoryItem::P1PosY).unwrap().clone(),
//             ),
//         );
//         members.insert(
//             MemoryItem::P2PosX,
//             DataMember::<f32>::new_offset(
//                 handle,
//                 mem_items.get(&MemoryItem::P2PosX).unwrap().clone(),
//             ),
//         );
//         members.insert(
//             MemoryItem::P2PosY,
//             DataMember::<f32>::new_offset(
//                 handle,
//                 mem_items.get(&MemoryItem::P2PosY).unwrap().clone(),
//             ),
//         );
//         MEMBERS = Some(members);
//     }
// }