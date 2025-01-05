use log::{debug, error, info};

type Port = std::boxed::Box<dyn serialport::SerialPort>;

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
enum CommandValue {
    GetChipId = 0x05,
    GetBootInfo = 0x10,
    LoadBootHeader = 0x11,
    /*
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
    */
    FlashRead = 0x32,
    /*
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
    fn to_slice(self) -> [u8; CMD_SIZE] {
        let sz = self.size;
        let l1 = (sz >> 8) as u8;
        let l0 = sz as u8;
        // NOTE: The second field is reserved, just zero it.
        [self.command, 0, l1, l0]
    }
}

const CHUNK_SIZE: usize = 4096;

fn send(port: &mut Port, command: CommandValue, data: &[u8]) {
    let cmd = Command {
        command: command as u8,
        size: data.len() as u16,
    }
    .to_slice();
    let mut resp = vec![0u8; 2];
    debug!("Command: {cmd:?}, data: {data:x?}");
    match port.write(&cmd) {
        Ok(n) => debug!("Sent command, {n} bytes"),
        Err(e) => error!("Error sending command: {e}"),
    }
    match port.write(data) {
        Ok(n) => debug!("Sent data, {n} bytes"),
        Err(e) => error!("Error sending data: {e}"),
    }
    match port.read(resp.as_mut_slice()) {
        Ok(n) => debug!("Read status, {n} bytes"),
        Err(e) => panic!("Error reading data: {e}"),
    };
    if resp != "OK".as_bytes() {
        panic!("Unexpected response: {resp:02x?}");
    }
    info!("Got OK");
    _ = port.read(resp.as_mut_slice()).expect("");
    debug!("{resp:?}");
    let size = u16::from_le_bytes([resp[0], resp[1]]) as usize;
    info!("size: {size}");
    let mut resp = vec![0u8; size];
    _ = port.read(resp.as_mut_slice()).expect("");
    debug!("{resp:02x?}");
}

const MAGIC: [u8; 12] = [
    0x50, 0x00, 0x08, 0x00, 0x38, 0xF0, 0x00, 0x20, 0, 0, 0, 0x18,
];

pub fn handshake(port: &mut Port) {
    info!("Handshake");
    loop {
        let written = port.write(&[b'U'; 32]);
        debug!("Wrote UU...: {written:?} bytes");
        let written = port.write(&MAGIC);
        debug!("Wrote magic: {written:?} bytes");
        let mut resp = vec![0u8; 2];
        match port.read(resp.as_mut_slice()) {
            Ok(_read) => {
                if resp == "OK".as_bytes() {
                    info!("Response okay, now send command");
                    break;
                } else {
                    debug!("Unexpected response, got {resp:02x?}, retry...");
                }
            }
            Err(e) => {
                error!("Error: {e}, retry...");
            }
        }
    }
}

pub fn get_info(port: &mut Port) {
    info!("Get boot info");
    send(port, CommandValue::GetBootInfo, &[]);
}

pub fn dump(port: &mut Port, addr: u32) {
    info!("Dump memory @ {addr:08x}");
    let data: [u8; 8] = [
        addr as u8,
        (addr >> 8) as u8,
        (addr >> 16) as u8,
        (addr >> 24) as u8,
        CHUNK_SIZE as u8,
        (CHUNK_SIZE >> 8) as u8,
        (CHUNK_SIZE >> 16) as u8,
        (CHUNK_SIZE >> 24) as u8,
    ];
    send(port, CommandValue::FlashRead, &data);
    let mut resp = vec![0u8; CHUNK_SIZE];
    info!("Read data: {CHUNK_SIZE} bytes");
    let read = port.read(resp.as_mut_slice()).expect("Found no data!");
    info!("{resp:02x?}");
}
