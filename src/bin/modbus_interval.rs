use std::thread;
use std::time;

use clap::Parser;
use comm_util::modbus;

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    /// COM port for target communication
    #[arg()]
    com_port: String,

    /// Baudrate for target communication
    #[arg()]
    baudrate: u32,

    /// Modbus number to repeatedly read
    #[arg()]
    modbus_nbr: u16,

    /// Interval for repeated reads in [ms]
    #[arg()]
    interval: u64,
}

fn main() {
    let args = Args::parse();

    let mut modbus_client = modbus::Client::new(args.com_port, args.baudrate);
    if let Err(_) = modbus_client.connect() {
        println!("Failed to connect to target, exiting..");
        return;
    }

    loop {
        let response = modbus_client.read(args.modbus_nbr, 1);
        match response {
            Ok(values) => {
                println!("{}: {}", args.modbus_nbr, values[0]);
            }
            Err(e) => {
                println!("Error in response from target: {e:?}");
            }
        }

        thread::sleep(time::Duration::from_millis(args.interval));
    }
}
