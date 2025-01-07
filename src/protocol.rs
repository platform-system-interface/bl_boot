use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::Write;

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

const OK: &[u8; 2] = b"OK";
const CHUNK_SIZE: usize = 4096;

fn send(port: &mut Port, command: CommandValue, data: &[u8]) -> Vec<u8> {
    let cmd = Command {
        command: command as u8,
        size: data.len() as u16,
    }
    .to_slice();
    debug!("Command: {cmd:02x?}, data: {data:02x?}");
    match port.write(&cmd) {
        Ok(n) => debug!("Sent command, {n} bytes"),
        Err(e) => error!("Error sending command: {e}"),
    }
    match port.write(data) {
        Ok(n) => debug!("Sent data, {n} bytes"),
        Err(e) => error!("Error sending data: {e}"),
    }
    let mut resp = vec![0u8; 2];
    match port.read(resp.as_mut_slice()) {
        Ok(n) => debug!("Read status, {n} bytes"),
        Err(e) => panic!("Error reading data: {e}"),
    };
    if resp != OK {
        panic!("Unexpected response: {resp:02x?} (wanted OK / {OK:02x?})");
    }
    debug!("Got OK: {resp:02x?}");
    // Depending on the command, we may not read a response.
    match command {
        CommandValue::FlashSetParam => {
            vec![]
        }
        _ => {
            let mut size_resp = vec![0u8; 2];
            _ = port.read(size_resp.as_mut_slice()).expect("");
            let size = u16::from_le_bytes([size_resp[0], size_resp[1]]) as usize;
            debug!("Read {size} bytes...");
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
    debug!("Handshake");
    loop {
        let written = port.write(&[b'U'; 32]);
        debug!("Wrote UU...: {written:?} bytes");
        let written = port.write(&MAGIC);
        debug!("Wrote magic: {written:?} bytes");
        let mut resp = vec![0u8; 2];
        match port.read(resp.as_mut_slice()) {
            Ok(_read) => {
                if resp == OK {
                    debug!("Response okay, now send command");
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

impl Display for BootInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let ci = self.chip_id;
        let fp = self.flash_pin;
        write!(f, "chip ID {ci:02x?}, flash pin {fp}")
    }
}

// TODO: other fields, support non-BL808 chips
fn get_boot_info(port: &mut Port) -> BootInfo {
    debug!("Get boot info");
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

// NOTE: values hardcoded from vendor config;
// `chips/bl808/eflash_loader/eflash_loader_cfg.conf` section [FLASH_CFG]
fn init_flash(port: &mut Port, bi: &BootInfo) {
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
}

pub fn get_flash_id(port: &mut Port) {
    let bi = get_boot_info(port);
    init_flash(port, &bi);

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

const STEP_SIZE: usize = 32;

fn get_flash_sha(port: &mut Port, bi: &BootInfo) {
    debug!("Read flash SHA");

    let a = 0x00u32;
    let l = 0x10u32;
    let d = [a.to_le_bytes(), l.to_le_bytes()].concat();

    init_flash(port, bi);
    let res = send(port, CommandValue::FlashReadSha, &d);
    for o in (0..res.len()).step_by(STEP_SIZE) {
        debug!("{:08x}: {:02x?}", a as usize + o, &res[o..o + STEP_SIZE]);
    }
}

// NOTE: The vendor code apparently accesses 3 slots, but I could only read 2.
const EFUSE_SLOT_COUNT: u32 = 2;

fn get_efuses(port: &mut Port) {
    debug!("Read efuses");

    let l = 0x80u32;
    for slot in 0..EFUSE_SLOT_COUNT {
        let a = slot * l;
        let d = [a.to_le_bytes(), l.to_le_bytes()].concat();
        let res = send(port, CommandValue::EfuseRead, &d);
        for o in (0..res.len()).step_by(STEP_SIZE) {
            debug!("{:08x}: {:02x?}", a as usize + o, &res[o..o + STEP_SIZE]);
        }
    }
}

pub fn get_info(port: &mut Port) {
    let bi = get_boot_info(port);
    info!("Boot info: {bi}");

    get_flash_sha(port, &bi);
    get_efuses(port);
}

pub fn dump_flash(port: &mut Port, offset: u32, size: u32, file: &str) -> std::io::Result<()> {
    get_flash_id(port);
    info!("Dump {size:08x} bytes from flash @ {offset:08x}");
    let mut f = File::create(file)?;
    for a in (offset..offset + size).step_by(CHUNK_SIZE) {
        let p = ((a as f32) / (size as f32) * 100.0) as u32;
        debug!("Now reading from {a:08x}, {p}%");
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
        f.write_all(&res);
    }
    Ok(())
}

pub fn read_log(port: &mut Port) {
    let res = send(port, CommandValue::LogRead, &[]);
    // TODO: Parse as ASCII / UTF-8?
    println!("{res:02x?}");
}
