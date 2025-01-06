use log::{debug, error, info};

type Port = std::boxed::Box<dyn serialport::SerialPort>;

/// Reference: https://github.com/openbouffalo/bflb-mcu-tool
///
/// libs/bflb_eflash_loader.py + libs/bflb_img_loader.py
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
    ClockSet = 0x22,
    OptFinish = 0x23,
    FlashErase = 0x30,
    FlashWrite = 0x31,
    FlashRead = 0x32,
    FlashBoot = 0x33,
    FlashXipRead = 0x34,
    FlashSwitchBank = 0x35,
    FlashReadJedecId = 0x36,
    FlashReadStatusReg = 0x37,
    FlashWriteStatusReg = 0x38,
    FlashWriteCheck = 0x3a,
    FlashSetParam = 0x3b,
    FlashChipErase = 0x3c,
    FlashReadSha = 0x3d,
    FlashXipReadSha = 0x3e,
    FlashDecompressWrite = 0x3f,
    EfuseWrite = 0x40,
    EfuseRead = 0x41,
    EfuseReadMac = 0x42,
    EfuseWriteMac = 0x43,
    FlashXipReadStart = 0x60,
    FlashXipReadFinish = 0x61,
    LogRead = 0x71, // Interesting!
    EfuseSecurityWrite = 0x80,
    EfuseSecurityRead = 0x81,
    EcdhGetPk = 0x90,
    EcdhChallenge = 0x91,
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

#[derive(Debug)]
struct BootInfo {
    chip_id: [u8; 6],
    flash_pin: u8,
}

fn get_boot_info(port: &mut Port) -> BootInfo {
    let mut res = send(port, CommandValue::GetBootInfo, &[]);
    debug!("{res:02x?}");

    let ci = &mut res[12..18];
    ci.reverse();
    let mut chip_id = [0u8; 6];
    chip_id.copy_from_slice(ci);

    let pcfg_bytes = u16::from_le_bytes([res[9], res[10]]);
    let flash_pin = ((pcfg_bytes >> 6) & 0x1f) as u8;
    BootInfo { chip_id, flash_pin }
}

pub fn get_info(port: &mut Port) {
    info!("Get boot info");
    let bi = get_boot_info(port);
    info!("{bi:02x?}");
}

// NOTE: values hardcoded from vendor config;
// `chips/bl808/eflash_loader/eflash_loader_cfg.conf` section [FLASH_CFG]
pub fn get_flash_id(port: &mut Port) {
    let bi = get_boot_info(port);

    // IO mode
    //   0: NIO,
    //   1: DO,
    //   2: QO,
    //   3: DIO,
    //   4: QIO
    let flash_io_mode = 1;

    // bit 7-4 flash_clock_type:
    //   0:120M wifipll,
    //   1:xtal,
    //   2:128M cpupll,
    //   3:80M wifipll,
    //   4:bclk,
    //   5:96M wifipll
    // bit 3-0 flash_clock_div
    let flash_clock_cfg = 0x41;

    // delay (in T):
    //   0: 0.5,
    //   1: 1,
    //   2: 1.5,
    //   3: 2
    let flash_clock_delay = 0;

    let data = [
        bi.flash_pin,
        flash_io_mode,
        flash_clock_cfg,
        flash_clock_delay,
    ];
    let res = send(port, CommandValue::FlashSetParam, &data);
    info!("Get JEDEC flash manufacturer/device ID");
    let res = send(port, CommandValue::FlashReadJedecId, &[]);
    let m = res[0];
    // https://github.com/SourceArcade/flashprog/blob/main/include/flashchips.h
    let manuf = match m {
        0xef => "Winbond",
        0xc8 => "GigaDevice",
        _ => "unknown",
    };
    // TODO: match manufacturer first; is there a library?
    let device = u16::from_le_bytes([res[1], res[2]]);
    info!("Manufacturer: {manuf} ({m:02x}), device: {device:04x}");
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
