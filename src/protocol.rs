use core::str;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::Write;
use std::thread::sleep;
use std::time::Duration;

use bitfield_struct::bitfield;
use log::{debug, error, info};
use zerocopy::{FromBytes, IntoBytes};
use zerocopy_derive::{FromBytes, IntoBytes};

use crate::boot::{BootHeader, Segment, D0_LOAD_ADDR, LP_LOAD_ADDR, M0_LOAD_ADDR};
use crate::efuses::{EfuseBlock0, EfuseBlock1, SwConfig0};

type Port = std::boxed::Box<dyn serialport::SerialPort>;

// should be plenty
const HALF_SEC: Duration = Duration::from_millis(500);
const BAUD_RATE: u32 = 2_000_000;

pub fn init(port: String) -> Port {
    let mut port = serialport::new(port, BAUD_RATE)
        .timeout(HALF_SEC)
        .open()
        .expect("Failed to open port {port}");
    handshake(&mut port);
    port
}

/// TODO: We could split up into two enums to ensure some can only send while
/// others also retrieve.
/// Reference: https://github.com/openbouffalo/bflb-mcu-tool
///
/// libs/bflb_eflash_loader.py + libs/bflb_img_loader.py
#[derive(Eq, PartialEq, Clone, Copy, Debug)]
#[repr(u8)]
enum Command {
    GetChipId = 0x05,
    GetBootInfo = 0x10,
    LoadBootHeader = 0x11,
    LoadPublicKey1 = 0x12,
    LoadPublicKey2 = 0x13,
    LoadSignature1 = 0x14,
    LoadSignature2 = 0x15,
    LoadAesIV = 0x16,
    LoadSegHeader = 0x17,
    LoadSegData = 0x18,
    // no response
    CheckImage = 0x19,
    // no response
    RunImage = 0x1a,
    ChangeRate = 0x20,
    // no response
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
    // no response
    FlashSetParam = 0x3b,
    FlashChipErase = 0x3c,
    FlashReadSha = 0x3d,
    FlashXipReadSha = 0x3e,
    FlashDecompressWrite = 0x3f,
    // no response
    EfuseWrite = 0x40,
    EfuseRead = 0x41,
    EfuseReadMac = 0x42,
    EfuseWriteMac = 0x43,
    FlashXipReadStart = 0x60,
    FlashXipReadFinish = 0x61,
    LogRead = 0x71,
    EfuseSecurityWrite = 0x80,
    EfuseSecurityRead = 0x81,
    EcdhGetPk = 0x90,
    EcdhChallenge = 0x91,
}

#[derive(Clone, Copy, Debug)]
#[repr(C, packed)]
struct CommandPacket {
    command: u8,
    size: u16,
}

const CMD_SIZE: usize = 4;

impl CommandPacket {
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

const CHUNK_SIZE: u32 = 4096;

// libs/bflb_utils.py
fn code_to_msg(code: u16) -> &'static str {
    match code {
        0x0102 => "command length error",
        0x0104 => "command sequence error",
        0x0204 => "image boot header CRC error",
        0x0205 => "fuses expected encryption but none in image boot header",
        0x0210 => "image section header CRC error",
        0x0217 => "image hash error",
        0x0405 => "eFuse read addr error",
        _ => "unknown error",
    }
}

// Read the status after sending a command.
fn get_ok(port: &mut Port) -> Result<(), String> {
    debug!("Check for command OK");
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
                let msg = code_to_msg(err_code);
                return Err(format!("Command error {err_code:04x} ({msg})"));
            }
            Err(e) => return Err(format!("Error reading error code: {e}")),
        };
    }
    if stat != OK {
        return Err(format!(
            "Unexpected status: {stat:02x?} (wanted OK / {OK:02x?})"
        ));
    }
    debug!("Command OK");
    Ok(())
}

fn send_cmd(port: &mut Port, command: Command, data: &[u8]) {
    let cmd = CommandPacket {
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
}

fn get_response(port: &mut Port) -> Vec<u8> {
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

fn send(port: &mut Port, command: Command, data: &[u8]) {
    send_cmd(port, command, data);
    if let Err(e) = get_ok(port) {
        panic!("{e}");
    }
}

fn send_and_retrieve(port: &mut Port, command: Command, data: &[u8]) -> Vec<u8> {
    send_cmd(port, command, data);
    if let Err(e) = get_ok(port) {
        panic!("{e}");
    }
    get_response(port)
}

const MAGIC: [u8; 12] = [
    0x50, 0x00, 0x08, 0x00, 0x38, 0xF0, 0x00, 0x20, 0x00, 0x00, 0x00, 0x18,
];

const RETRIES: u64 = 5;

pub fn handshake(port: &mut Port) {
    debug!("Handshake");
    for r in 0..RETRIES {
        let written = port.write(&[b'U'; 32]);
        debug!("Wrote UU...: {written:?} bytes");
        // Give the auto baud rate detection + adjustment some time.
        sleep(Duration::from_millis(100));
        let written = port.write(&MAGIC);
        debug!("Wrote magic: {written:?} bytes");
        match get_ok(port) {
            Ok(()) => {
                debug!("Status okay, now send command");
                return;
            }
            Err(e) => {
                error!("{e}, retry...");
                sleep(Duration::from_millis(r * 200));
            }
        }
    }
    error!("Tried handshake {RETRIES} times, to no avail. :(");
    panic!("Failed to connect");
}

#[derive(Clone, Debug, Copy, FromBytes, IntoBytes)]
#[repr(C, packed)]
pub struct BootInfo {
    rom_driver_version: u32,
    x0: u32,
    sw_config0: crate::efuses::SwConfig0,
    // NOTE: BootInfo appears to drop relevant bits here which EfuseBlock0 has.
    // E.g., `5c 00`, while the fuses really contain `5c 06`.
    wifi_mac_x: crate::efuses::WifiMacAndInfo,
    sw_config1: crate::efuses::SwConfig1,
}

impl Display for BootInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let x0 = self.x0;
        let rom_drv = self.rom_driver_version;
        let rom_drv = format!("ROM driver vesion: {rom_drv:08x}");
        let cfg0 = self.sw_config0;
        let cfg1 = self.sw_config1;
        let macx = self.wifi_mac_x;
        let mac = macx.mac_addr();
        let mac = format!("Wi-Fi MAC: {mac:012x}");
        let info = macx.info();

        write!(
            f,
            "{x0:08x}\n{rom_drv}\n{mac}\n{info}\n{cfg0:#?}\n{cfg1:#?}"
        )
    }
}

// TODO: other fields, support non-BL808 chips
fn get_boot_info(port: &mut Port) -> BootInfo {
    debug!("Get boot info");
    let mut res = send_and_retrieve(port, Command::GetBootInfo, &[]);
    debug!("{res:02x?}");

    let bi = BootInfo::read_from_bytes(&res).unwrap();
    bi
}

// NOTE: values hardcoded from vendor config;
// TODO: define struct for variants
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
    send(port, Command::FlashSetParam, &data)
}

pub fn get_flash_id(port: &mut Port) {
    let bi = get_boot_info(port);
    init_flash(port, &bi);

    info!("Get JEDEC flash manufacturer/device ID");
    let res = send_and_retrieve(port, Command::FlashReadJedecId, &[]);
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
    let res = send_and_retrieve(port, Command::FlashReadSha, &d);
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
    let res = send_and_retrieve(port, Command::EfuseRead, &d);
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
    let res = send_and_retrieve(port, Command::EfuseRead, &d);
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
    send(port, Command::Reset, &[]);
}

pub fn set_efuses(port: &mut Port, address: u32, data: &[u8]) {
    debug!("Write efuses @ {address:08x}: {data:02x?}");
    let mut d = Vec::<u8>::new();
    d.extend_from_slice(&address.to_le_bytes());
    d.extend_from_slice(data);
    send(port, Command::EfuseWrite, &d);
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
    for a in (offset..offset + size).step_by(CHUNK_SIZE as usize) {
        let p = ((a as f32) / (size as f32) * 100.0) as u32;
        debug!("Now reading from {a:08x}, {p}%");
        if (a - offset) % (0x20 * CHUNK_SIZE) == 0 {
            info!("{p}%");
        }
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
        let res = send_and_retrieve(port, Command::FlashRead, &data);
        f.write_all(&res);
    }
    Ok(())
}

pub fn flash_image(port: &mut Port, data: &[u8]) {
    get_flash_id(port);

    let l = data.len();
    let start = 0u32.to_le_bytes();
    let end = (l as u32).to_le_bytes();
    let mut d = Vec::<u8>::new();
    d.extend_from_slice(&start);
    d.extend_from_slice(&end);
    info!("Erase {l} bytes");
    send(port, Command::FlashErase, &d);

    let cs = CHUNK_SIZE as usize;
    let full_chunks = l / cs;
    info!("Send chunks");
    for c in 0..full_chunks {
        let o = c * cs;
        let chunk = &data[o..o + cs];
        let offset = (o as u32).to_le_bytes();
        let mut d = Vec::<u8>::new();
        d.extend_from_slice(&offset);
        d.extend_from_slice(chunk);
        info!("Write chunk {c} at offset {o:08x}");
        send(port, Command::FlashWrite, &d);
    }
    if data.len() % cs > 0 {
        let remaining = &data[full_chunks * cs..];
        info!("Send remaining data, {} bytes", remaining.len());
        send(port, Command::FlashWrite, remaining);
    }
}

pub fn read_log(port: &mut Port) {
    let res = send_and_retrieve(port, Command::LogRead, &[]);
    match str::from_utf8(&res) {
        Ok(s) => {
            info!("=== Log start\n{s}");
            info!("=== Log end");
        }
        Err(e) => {
            let l = res.len();
            let max = if l > 4 { 4 } else { l };
            let first = &res[..max];
            let details = format!("Got {l} bytes starting with {first:02x?}");
            error!("Cannot parse log as UTF-8: {e}\n{details}");
        }
    }
}

pub fn send_segment(port: &mut Port, s: &crate::boot::Segment) {
    info!("Send segment header: {:#08x?}", s.header);
    let res = send_and_retrieve(port, Command::LoadSegHeader, s.header.as_bytes());
    debug!("Got: {res:02x?}");
    let cs = CHUNK_SIZE as usize;
    let full_chunks = s.data.len() / cs;
    info!("Send segment data");
    for c in 0..full_chunks {
        info!("Send chunk {c}");
        let o = c * cs;
        send(port, Command::LoadSegData, &s.data[o..o + cs]);
    }
    if s.data.len() % cs > 0 {
        info!("Send remaining data");
        send(port, Command::LoadSegData, &s.data[full_chunks * cs..]);
    }
}

pub fn run(
    port: &mut Port,
    data1: Option<Vec<u8>>,
    data2: Option<Vec<u8>>,
    data3: Option<Vec<u8>>,
) {
    let s1 = data1.as_ref().map(|d| Segment::new(M0_LOAD_ADDR, d));
    let s2 = data2.as_ref().map(|d| Segment::new(D0_LOAD_ADDR, d));
    let s3 = data3.as_ref().map(|d| Segment::new(LP_LOAD_ADDR, d));

    let header = BootHeader::new(s1, s2, s3);
    let header_bytes = header.as_bytes();
    let step_size = 8;
    for o in (0..header_bytes.len()).step_by(step_size) {
        debug!("{:08x}: {:02x?}", o, &header_bytes[o..o + step_size]);
    }
    info!("Send boot header");
    send(port, Command::LoadBootHeader, header_bytes);
    if let Some(s) = s1 {
        send_segment(port, &s);
    }
    if let Some(s) = s2 {
        send_segment(port, &s);
    }
    if let Some(s) = s3 {
        send_segment(port, &s);
    }
    info!("Check image");
    send(port, Command::CheckImage, &[]);
    info!("Run image");
    send(port, Command::RunImage, &[]);
}
