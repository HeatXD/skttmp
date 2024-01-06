use process_memory::{LocalMember, Memory};
use std::sync::Mutex;
use std::{net::TcpStream, thread};
use tracing::info;
use winapi::um::errhandlingapi::GetLastError;

use winapi::um::libloaderapi::GetModuleHandleW;
use winapi::um::processthreadsapi::GetCurrentProcess;
use winapi::um::psapi::{GetModuleInformation, MODULEINFO};

#[ctor::ctor]
fn init() {
    let stream = TcpStream::connect("127.0.0.1:7788").unwrap();
    stream.nodelay().unwrap();
    tracing_subscriber::fmt()
        .with_writer(Mutex::new(stream))
        .init();

    info!("Game Info:");
    let exe_name = "shovelknight.exe";

    // Convert the executable name to a wide string (UTF-16)
    let exe_name_wide: Vec<u16> = exe_name.encode_utf16().chain(Some(0)).collect();

    let module_handle = unsafe { GetModuleHandleW(exe_name_wide.as_ptr()) };

    if module_handle.is_null() {
        info!("Failed to get module handle. Error code: {}", unsafe {
            GetLastError()
        });
        return;
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
        return;
    }

    let base_address = module_info.lpBaseOfDll as usize;
    info!("Base address of {}: 0x{:X}", exe_name, base_address);

    thread::spawn(move || {
        info!("Spawning thread");
        let mut member: LocalMember<u32> = LocalMember::<u32>::new();
        info!("Setting offset");
        member.set_offset(vec![base_address + 0x9072CC, 0xEE4]); // gold in shovel of hope
        info!("Getting offset");
        match member.get_offset() {
            Err(err) => {
                info!("Finding gold failed! {:?}", err);
                return;
            }
            Ok(offset) => info!("Target memory location 0x{:X}", offset),
        }
        info!("Current value: {}", unsafe { member.read().unwrap() });
        let x: u32 = 9999;
        member.write(&x).unwrap();
        info!("New value: {}", unsafe { member.read().unwrap() });
    });
}
