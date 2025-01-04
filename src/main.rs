#[allow(dead_code)]
use clap::{Parser, Subcommand};
use log::{debug, error, info};
use std::{thread::sleep, time::Duration};

mod protocol;

// should be plenty
const HALF_SEC: Duration = Duration::from_millis(500);

#[derive(Debug, Subcommand)]
enum Command {
    Info {
        #[clap(long, short, action, default_value = "/dev/ttyUSB1")]
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

const MAGIC: [u8; 12] = [
    0x50, 0x00, 0x08, 0x00, 0x38, 0xF0, 0x00, 0x20, 0, 0, 0, 0x18,
];

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
            loop {
                let written = port.write(&[b'U'; 32]);
                debug!("wrote UU...: {written:?} bytes");
                let written = port.write(&MAGIC);
                debug!("wrote magic: {written:?} bytes");
                let mut resp = vec![0u8; 2];
                info!("Handshake");
                match port.read(resp.as_mut_slice()) {
                    Ok(_read) => {
                        if resp == "OK".as_bytes() {
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
            protocol::get_info(&mut port);
            info!("Nothing to see here :)");
        }
    }
}
