#[allow(unused)]
use clap::{Parser, Subcommand};
use log::{debug, error, info};
use std::time::Duration;

mod protocol;

// should be plenty
const HALF_SEC: Duration = Duration::from_millis(100);

const PORT: &str = "/dev/ttyUSB1";

#[derive(Debug, Subcommand)]
enum Command {
    Dump {
        #[clap(long, short, action, default_value = PORT)]
        port: String,
        #[arg(index = 1, value_parser=clap_num::maybe_hex::<u32>)]
        address: u32,
    },
    Info {
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
            info!("Nothing to see here :)");
        }
        Command::Dump { port, address } => {
            info!("Using port {port}");
            let mut port = serialport::new(port, 115_200)
                .timeout(HALF_SEC)
                .open()
                .expect("Failed to open port {port}");
            protocol::handshake(&mut port);
            protocol::dump(&mut port, address);
            info!("Nothing to see here :)");
        }
    }
}
