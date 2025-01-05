use log::{debug, error, info};

type Port = std::boxed::Box<dyn serialport::SerialPort>;

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
enum CommandValue {
    GetChipId = 0x05,
    GetBootInfo = 0x10,
    LoadBootHeader = 0x11,
    LoadPublicKey = 0x12,
    LoadPublicKey2 = 0x13,
    LoadSignature = 0x14,
    LoadSignature2 = 0x15,
    LoadAesIV = 0x16,
    LoadSegHeader = 0x17,
    LoadSegData = 0x18,
    CheckImage = 0x19,
    RunImage = 0x1a,
    ChangeRate = 0x20,
    Reset = 0x21,
    FlashErase = 0x30,
    FlashWrite = 0x31,
    FlashRead = 0x32,
    FlashBoot = 0x33,
    FlashReadJedecId = 0x36,
    FlashSetParam = 0x3b,
    EfuseWrite = 0x40,
    EfuseRead = 0x41,
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
        let l0 = sz as u8;
        let l1 = (sz >> 8) as u8;
        // NOTE: The second field is reserved, just zero it.
        [self.command, 0, l0, l1]
    }
}

const CHUNK_SIZE: usize = 4096;

fn send(port: &mut Port, command: CommandValue, data: &[u8]) -> Vec<u8> {
    let cmd = Command {
        command: command as u8,
        size: data.len() as u16,
    }
    .to_slice();
    let mut resp = vec![0u8; 2];
    debug!("Command: {cmd:02x?}, data: {data:02x?}");
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
    // Depending on the command, we may not read a response.
    match command {
        CommandValue::FlashSetParam => {
            vec![]
        }
        _ => {
            _ = port.read(resp.as_mut_slice()).expect("");
            let size = u16::from_le_bytes([resp[0], resp[1]]) as usize;
            info!("size: {size} ({resp:02x?})");
            let mut resp = vec![0u8; size];
            port.read_exact(resp.as_mut_slice()).expect("");
            resp
        }
    }
}

const MAGIC: [u8; 12] = [
    0x50, 0x00, 0x08, 0x00, 0x38, 0xF0, 0x00, 0x20, 0x00, 0x00, 0x00, 0x18,
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
    let res = send(port, CommandValue::GetBootInfo, &[]);
    debug!("{res:02x?}");
    let x = [res[9], res[10]];
    let xx = u16::from_le_bytes(x);
    let pcfg = (xx >> 6) & 0x1f;
    debug!(" flash pin {xx:02x?} - {pcfg:02x}");
}

pub fn get_flash_id(port: &mut Port) {
    info!("Get JEDEC flash manufacturer/device ID");
    let cfg = 0x0001_4104;
    let data = [
        cfg as u8,
        (cfg >> 8) as u8,
        (cfg >> 16) as u8,
        (cfg >> 24) as u8,
    ];
    let res = send(port, CommandValue::FlashSetParam, &data);
    let res = send(port, CommandValue::FlashReadJedecId, &[]);
    debug!("{res:02x?}");
}

pub fn dump_flash(port: &mut Port, offset: u32, size: u32) {
    get_flash_id(port);
    info!("Dump {size:08x} bytes from flash @ {offset:08x}");
    for a in (offset..offset + size).step_by(CHUNK_SIZE) {
        let data: [u8; 8] = [
            a as u8,
            (a >> 8) as u8,
            (a >> 16) as u8,
            (a >> 24) as u8,
            CHUNK_SIZE as u8,
            (CHUNK_SIZE >> 8) as u8,
            (CHUNK_SIZE >> 16) as u8,
            (CHUNK_SIZE >> 24) as u8,
        ];
        let res = send(port, CommandValue::FlashRead, &data);
        for o in (0..CHUNK_SIZE).step_by(32) {
            debug!("{:08x}: {:02x?}", a as usize + o, &res[o..o + 32]);
        }
    }
}
