use core::str;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::Write;

use bitfield_struct::bitfield;
use log::{debug, error, info};
use zerocopy::{FromBytes, IntoBytes};
use zerocopy_derive::{FromBytes, IntoBytes};

use crate::efuses::{EfuseBlock0, EfuseBlock1, SwConfig0};

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

// Response "okay"
const OK: &[u8; 2] = b"OK";
// Response "fail"
const FL: &[u8; 2] = b"FL";

const CHUNK_SIZE: usize = 4096;

// libs/bflb_utils.py
fn code_to_msg(code: u16) -> &'static str {
    match code {
        0x0405 => "eFuse read addr error",
        _ => "unknown error",
    }
}

fn send(port: &mut Port, command: CommandValue, data: &[u8]) -> Vec<u8> {
    let cmd = Command {
        command: command as u8,
        size: data.len() as u16,
    }
    .to_slice();
    debug!("Command: {cmd:02x?}, data: {data:02x?}");
    // First, send the command and data.
    match port.write(&cmd) {
        Ok(n) => debug!("Sent command, {n} bytes"),
        Err(e) => error!("Error sending command: {e}"),
    }
    match port.write(data) {
        Ok(n) => debug!("Sent data, {n} bytes"),
        Err(e) => error!("Error sending data: {e}"),
    }
    // Now read the status.
    let mut stat = vec![0u8; 2];
    match port.read(stat.as_mut_slice()) {
        Ok(n) => debug!("Read status, {n} bytes"),
        Err(e) => panic!("Error reading status: {e}"),
    };
    if stat == FL {
        error!("Command failed");
        let mut code = vec![0u8; 2];
        match port.read(code.as_mut_slice()) {
            Ok(n) => {
                let err_code = u16::from_le_bytes([code[0], code[1]]);
                error!("Error code: {err_code:04x} ({})", code_to_msg(err_code))
            }
            Err(e) => panic!("Error reading error code: {e}"),
        };
    }
    if stat != OK {
        panic!("Unexpected status: {stat:02x?} (wanted OK / {OK:02x?})");
    }
    debug!("Command OK");
    // Depending on the command, we may not read a response.
    match command {
        // TODO: We could split up into two functions to send and retrieve.
        // How would we best encode which commands do retrieve data?
        CommandValue::FlashSetParam | CommandValue::EfuseWrite | CommandValue::Reset => {
            vec![]
        }
        _ => {
            // First we get the size of the response.
            let mut size = vec![0u8; 2];
            match port.read_exact(size.as_mut_slice()) {
                Ok(_) => debug!("Reponse size read successfully"),
                Err(e) => panic!("Error reading response size: {e}"),
            };
            let size = u16::from_le_bytes([size[0], size[1]]) as usize;

            debug!("Read {size} bytes...");
            let mut resp = vec![0u8; size];
            match port.read_exact(resp.as_mut_slice()) {
                Ok(_) => debug!("Reponse data read successfully"),
                Err(e) => panic!("Error reading response data: {e}"),
            };
            resp
        }
    }
}

const MAGIC: [u8; 12] = [
    0x50, 0x00, 0x08, 0x00, 0x38, 0xF0, 0x00, 0x20, 0x00, 0x00, 0x00, 0x18,
];

const RETRIES: usize = 5;

pub fn handshake(port: &mut Port) {
    debug!("Handshake");
    for _ in 0..RETRIES {
        let written = port.write(&[b'U'; 32]);
        debug!("Wrote UU...: {written:?} bytes");
        let written = port.write(&MAGIC);
        debug!("Wrote magic: {written:?} bytes");
        let mut stat = vec![0u8; 2];
        match port.read(stat.as_mut_slice()) {
            Ok(_read) => {
                if stat == OK {
                    debug!("Status okay, now send command");
                    return;
                } else {
                    debug!("Unexpected status {stat:02x?}, retry...");
                }
            }
            Err(e) => {
                error!("Error: {e}, retry...");
            }
        }
    }
    error!("Tried {RETRIES} times, to no avail. :(");
    panic!("Failed to connect");
}

#[bitfield(u64)]
#[derive(FromBytes, IntoBytes)]
pub struct BootInfo0 {
    pub x: u64,
}

#[derive(Clone, Debug, Copy, FromBytes, IntoBytes)]
#[repr(C, packed)]
pub struct BootInfo {
    x0: BootInfo0,
    sw_config0: crate::efuses::SwConfig0,
    // NOTE: BootInfo appears to drop relevant bits here which EfuseBlock0 has.
    // E.g., `5c 00`, while the fuses really contain `5c 06`.
    wifi_mac_x: crate::efuses::WifiMacAndInfo,
    sw_config1: crate::efuses::SwConfig1,
}

impl Display for BootInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let x0 = self.x0;
        let cfg0 = self.sw_config0;
        let cfg1 = self.sw_config1;
        let macx = self.wifi_mac_x;
        let mac = macx.mac_addr();
        let mac = format!("Wi-Fi MAC: {mac:012x}");
        let info = macx.info();

        write!(f, "{x0:016x?}\n{mac}\n{info}\n{cfg0:#?}\n{cfg1:#?}")
    }
}

// TODO: other fields, support non-BL808 chips
fn get_boot_info(port: &mut Port) -> BootInfo {
    debug!("Get boot info");
    let mut res = send(port, CommandValue::GetBootInfo, &[]);
    debug!("{res:02x?}");

    let bi = BootInfo::read_from_bytes(&res).unwrap();
    bi
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

    let sw_cfg0 = bi.sw_config0;
    let data = [
        sw_cfg0.spi_flash_pin_cfg() as u8,
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

const EFUSE_SLOT_SIZE: u32 = 0x80;

// NOTE: The vendor code apparently accesses 3 slots, but I could only read 2.
pub fn get_efuses(port: &mut Port) -> Vec<u8> {
    debug!("Read efuses");

    let mut ret = Vec::<u8>::new();
    let a = 0u32;
    let size = EFUSE_SLOT_SIZE.to_le_bytes();
    let d = [a.to_le_bytes(), size].concat();
    let res = send(port, CommandValue::EfuseRead, &d);
    ret.extend_from_slice(&res);
    for o in (0..res.len()).step_by(STEP_SIZE) {
        debug!("{:08x}: {:02x?}", a as usize + o, &res[o..o + STEP_SIZE]);
    }
    match EfuseBlock0::read_from_bytes(&res) {
        Ok(f) => info!("eFuse block 0:\n{f}"),
        Err(e) => error!("Could not parse eFuse data"),
    }
    let a = EFUSE_SLOT_SIZE;
    let d = [a.to_le_bytes(), size].concat();
    let res = send(port, CommandValue::EfuseRead, &d);
    ret.extend_from_slice(&res);
    for o in (0..res.len()).step_by(STEP_SIZE) {
        debug!("{:08x}: {:02x?}", a as usize + o, &res[o..o + STEP_SIZE]);
    }
    match EfuseBlock1::read_from_bytes(&res) {
        Ok(f) => info!("eFuse block 1:\n{f}"),
        Err(e) => error!("Could not parse eFuse data"),
    }

    ret
}

pub fn reset(port: &mut Port) {
    debug!("Reset");
    _ = send(port, CommandValue::Reset, &[]);
}

pub fn set_efuses(port: &mut Port, address: u32, data: &[u8]) {
    debug!("Write efuses @ {address:08x}: {data:02x?}");
    let mut d = Vec::<u8>::new();
    d.extend_from_slice(&address.to_le_bytes());
    d.extend_from_slice(data);
    _ = send(port, CommandValue::EfuseWrite, &d);
}

pub fn set_efuse(port: &mut Port, address: u32, value: u32) {
    set_efuses(port, address, &value.to_le_bytes());
}

pub fn reenable_log(port: &mut Port) {
    let a: u32 = 0x5c;
    let mut cfg = &SwConfig0::new().with_uart_log_reopen(true);
    let v = cfg.into_bits();
    set_efuse(port, a, v);
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
    match str::from_utf8(&res) {
        Ok(s) => {
            info!("=== Log start\n{s}");
            info!("=== Log end");
        }
        Err(e) => error!("{e}"),
    }
}
