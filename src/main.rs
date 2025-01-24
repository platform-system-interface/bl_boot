#![allow(unused)]
use std::thread::sleep;
use std::time::Duration;
use std::{fs, io::Write};

use clap::{Parser, Subcommand};
use log::{debug, error, info};
use zerocopy::FromBytes;

mod boot;
mod efuses;
mod mem_map;
mod protocol;

const PORT: &str = "/dev/ttyUSB1";

#[derive(Debug, Subcommand)]
enum Command {
    /// Identify a SPI flash on the board (JEDEC ID).
    FlashId {
        #[clap(long, short, action, default_value = PORT)]
        port: String,
    },
    /// Dump content of a SPI flash on the board.
    DumpFlash {
        #[clap(long, short, action, default_value = PORT)]
        port: String,
        #[arg(index = 1, value_parser=clap_num::maybe_hex::<u32>)]
        offset: u32,
        #[arg(index = 2, value_parser=clap_num::maybe_hex::<u32>)]
        size: u32,
        #[arg(index = 3)]
        file_name: String,
    },
    /// Reset the platform
    Reset {
        #[clap(long, short, action, default_value = PORT)]
        port: String,
    },
    /// Reenable the mask ROM's logging function, necessary for the log command.
    ReenableLog {
        #[clap(long, short, action, default_value = PORT)]
        port: String,
    },
    /// Read out the log from the mask ROM. Needs efuse configuration, see above.
    Log {
        #[clap(long, short, action, default_value = PORT)]
        port: String,
    },
    /// Read fuses in the SoC to a file
    ReadFuses {
        file_name: String,
        #[clap(long, short, action, default_value = PORT)]
        port: String,
    },
    /// Burn fuses in the SoC with data read from file, must be 128 (0x80) bytes
    SetFuses {
        file_name: String,
        #[clap(long, short, action, default_value = PORT)]
        port: String,
    },
    /// Print information on the SoC.
    Info {
        #[clap(long, short, action, default_value = PORT)]
        port: String,
    },
    /// Write file(s) to SRAM and execute
    #[clap(verbatim_doc_comment)]
    Run {
        #[clap(long, short, action)]
        m0_binary: Option<String>,
        #[clap(long, short, action)]
        d0_binary: Option<String>,
        #[clap(long, short, action)]
        lp_binary: Option<String>,
        #[clap(long, short, action, default_value = PORT)]
        port: String,
    },
    /// Write a prebuilt image to flash.
    FlashImage {
        /// Image file to flash
        file_name: String,
        #[clap(long, short, action, default_value = PORT)]
        port: String,
    },
    BuildImage {
        #[clap(long, short, action)]
        m0_binary: Option<String>,
        #[clap(long, short, action)]
        d0_binary: Option<String>,
        #[clap(long, short, action)]
        lp_binary: Option<String>,
        /// Output file
        file_name: String,
    },
    /// Parse a flash image.
    ParseImage { file_name: String },
}

/// Bouffalo Lab mask ROM loader tool
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Command to run
    #[command(subcommand)]
    cmd: Command,
}

fn main() -> std::io::Result<()> {
    let cmd = Cli::parse().cmd;
    // Default to log level "info". Otherwise, you get no "regular" logs.
    let env = env_logger::Env::default().default_filter_or("info");
    env_logger::Builder::from_env(env).init();

    match cmd {
        Command::Run {
            m0_binary,
            d0_binary,
            lp_binary,
            port,
        } => {
            let m0_bin = m0_binary.map(|f| fs::read(f).unwrap());
            let d0_bin = d0_binary.map(|f| fs::read(f).unwrap());
            let lp_bin = lp_binary.map(|f| fs::read(f).unwrap());
            info!("Using port {port}");
            let mut port = protocol::init(port);
            protocol::run(&mut port, m0_bin, d0_bin, lp_bin);
            info!("🎉 Done. Now read from serial port...");
            let mut c = &mut [0u8];
            loop {
                match port.read(c) {
                    Ok(n) => print!("{}", c[0] as char),
                    _ => sleep(Duration::from_millis(500)),
                }
            }
        }
        Command::Reset { port } => {
            info!("Using port {port}");
            let mut port = protocol::init(port);
            protocol::reset(&mut port);
        }
        Command::ReenableLog { port } => {
            info!("Using port {port}");
            let mut port = protocol::init(port);
            protocol::reenable_log(&mut port);
        }
        Command::Log { port } => {
            info!("Using port {port}");
            let mut port = protocol::init(port);
            protocol::read_log(&mut port);
        }
        Command::Info { port } => {
            info!("Using port {port}");
            let mut port = protocol::init(port);
            protocol::get_info(&mut port);
        }
        Command::ReadFuses { port, file_name } => {
            info!("Using port {port}");
            let mut f = fs::File::create(file_name)?;
            let mut port = protocol::init(port);
            let r = protocol::get_efuses(&mut port);
            f.write_all(&r);
        }
        Command::SetFuses { port, file_name } => {
            info!("Using port {port}");
            let mut payload = std::fs::read(file_name).unwrap();
            if payload.len() != 0x80 {
                panic!("File must be 128 (0x80) bytes!");
            }
            match efuses::EfuseBlock0::read_from_bytes(&payload) {
                Ok(f) => info!("Efuses:\n{f}"),
                Err(e) => error!("Could not parse efuse data"),
            }
            let mut port = protocol::init(port);
            protocol::set_efuses(&mut port, 0, &payload);
        }
        Command::FlashId { port } => {
            info!("Using port {port}");
            let mut port = protocol::init(port);
            protocol::get_info(&mut port);
            protocol::get_flash_id(&mut port);
        }
        Command::DumpFlash {
            port,
            offset,
            size,
            file_name,
        } => {
            info!("Using port {port}");
            let mut port = protocol::init(port);
            protocol::dump_flash(&mut port, offset, size, &file_name);
        }
        Command::FlashImage { port, file_name } => {
            info!("Using port {port}");
            let mut port = protocol::init(port);
            let d = fs::read(file_name).unwrap();
            protocol::flash_image(&mut port, &d);
        }
        Command::BuildImage {
            m0_binary,
            d0_binary,
            lp_binary,
            file_name,
        } => {
            let mut f = fs::File::create(file_name)?;
            let m0_bin = m0_binary.map(|f| fs::read(f).unwrap());
            let d0_bin = d0_binary.map(|f| fs::read(f).unwrap());
            let lp_bin = lp_binary.map(|f| fs::read(f).unwrap());
            let image = boot::build_image(m0_bin, d0_bin, lp_bin);
            f.write_all(&image);
        }
        Command::ParseImage { file_name } => {
            let f = fs::read(file_name).unwrap();
            boot::parse_image(&f);
        }
    }

    Ok(())
}
