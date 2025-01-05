#![allow(unused)]
use clap::{Parser, Subcommand};
use log::{debug, error, info};
use std::time::Duration;

mod protocol;

// should be plenty
const HALF_SEC: Duration = Duration::from_millis(100);

const PORT: &str = "/dev/ttyUSB1";

#[derive(Debug, Subcommand)]
enum Command {
    DumpFlash {
        #[clap(long, short, action, default_value = PORT)]
        port: String,
        #[arg(index = 1, value_parser=clap_num::maybe_hex::<u32>)]
        offset: u32,
        #[arg(index = 2, value_parser=clap_num::maybe_hex::<u32>)]
        size: u32,
    },
    Info {
        #[clap(long, short, action, default_value = PORT)]
        port: String,
    },
    FlashId {
        #[clap(long, short, action, default_value = PORT)]
        port: String,
    },
    /// Write file to SRAM and execute
    #[clap(verbatim_doc_comment)]
    Run {
        file_name: String,
        #[clap(long, short, action, default_value = "/dev/ttyUSB1")]
        port: String,
    },
}

/// Bouffalo Lab mask ROM loader tool
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Command to run
    #[command(subcommand)]
    cmd: Command,
}

fn main() {
    let cmd = Cli::parse().cmd;
    env_logger::init();

    match cmd {
        Command::Run { file_name, port } => {
            let mut port = serialport::new(port, 115_200)
                .timeout(HALF_SEC)
                .open()
                .expect("Failed to open port {port}");
            let mut payload = std::fs::read(file_name).unwrap();
            let sz = payload.len();
            info!("Payload size: {sz}");
            // TODO: send file
            info!("🎉 Done. Nothing happened.");
        }
        Command::Info { port } => {
            info!("Using port {port}");
            let mut port = serialport::new(port, 115_200)
                .timeout(HALF_SEC)
                .open()
                .expect("Failed to open port {port}");
            protocol::handshake(&mut port);
            protocol::get_info(&mut port);
        }
        Command::FlashId { port } => {
            info!("Using port {port}");
            let mut port = serialport::new(port, 115_200)
                .timeout(HALF_SEC)
                .open()
                .expect("Failed to open port {port}");
            protocol::handshake(&mut port);
            protocol::get_info(&mut port);
            protocol::get_flash_id(&mut port);
        }
        Command::DumpFlash { port, offset, size } => {
            info!("Using port {port}");
            let mut port = serialport::new(port, 115_200)
                .timeout(HALF_SEC)
                .open()
                .expect("Failed to open port {port}");
            protocol::handshake(&mut port);
            protocol::dump_flash(&mut port, offset, size);
        }
    }
}
