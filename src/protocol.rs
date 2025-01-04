use std::str::from_utf8;

use log::{debug, error, info};

type Port = std::boxed::Box<dyn serialport::SerialPort>;

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
enum CommandValue {
    GetChipId = 0x05,
    GetBootInfo = 0x10,
    /*
    LOAD_BOOT_HEADER = 0x11,
    LOAD_PUBLIC_KEY = 0x12,
    LOAD_PUBLIC_KEY2 = 0x13,
    LOAD_SIGNATURE = 0x14,
    LOAD_SIGNATURE2 = 0x15,
    LOAD_AES_IV = 0x16,
    LOAD_SEG_HEADER = 0x17,
    LOAD_SEG_DATA = 0x18,
    CHECK_IMAGE = 0x19,
    RUN_IMAGE = 0x1a,
    CHANGE_RATE = 0x20,
    RESET = 0x21,
    FLASH_ERASE = 0x30,
    FLASH_WRITE = 0x31,
    FLASH_READ = 0x32,
    FLASH_BOOT = 0x33,
    EFUSE_WRITE = 0x40,
    EFUSE_READ = 0x41,
    */
}

#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
struct Command {
    command: u8,
    size: u16,
}

const CMD_SIZE: usize = 4;

impl Command {
    fn to_slice(&self) -> [u8; CMD_SIZE] {
        let sz = self.size;
        let l1 = (sz >> 8) as u8;
        let l0 = sz as u8;
        // NOTE: The second field is reserved, just zero it.
        [self.command, 0, l1, l0]
    }
}

const RESPONSE_SIZE: usize = 4096;
// let read = port.read(resp.as_mut_slice()).expect("Found no data!");

fn send(port: &mut Port, data: &[u8]) {
    debug!("data: {data:x?}");
    let sent = port.write(data).expect("Write failed!");
    let _ = port.write(&[]);
    let mut resp = vec![0u8; 2];
    let read = port.read(resp.as_mut_slice()).expect("Found no data!");
    // check_response(data, &resp);
    info!("sent {sent} bytes, read {read} bytes");
    debug!("{resp:?}");
    if resp == "OK".as_bytes() {
        info!("Got OK");
        _ = port.read(resp.as_mut_slice()).expect("Found no data!");
        debug!("{resp:?}");
        let size = u16::from_le_bytes([resp[0], resp[1]]) as usize;
        info!("size: {size}");
        let mut resp = vec![0u8; size];
        _ = port.read(resp.as_mut_slice()).expect("Found no data!");
        debug!("{resp:02x?}");
    }
}

pub fn get_info(port: &mut Port) {
    info!("Get boot info");
    let cmd = Command {
        command: CommandValue::GetBootInfo as u8,
        size: 0,
    };
    let data: &[u8] = &cmd.to_slice();
    send(port, data);
}
