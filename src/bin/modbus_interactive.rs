use std::io;

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
}

fn main() {
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async_main());
}

async fn async_main() {
    let args = Args::parse();

    let mut modbus_client = modbus::Client::new(args.com_port, args.baudrate);
    if modbus_client.open().is_err() {
        println!("Failed to connect to target, exiting..");
        return;
    }

    loop {
        let mut user_input: String = String::new();
        io::stdin().read_line(&mut user_input).unwrap();

        let parts: Vec<&str> = user_input.trim().split(' ').collect();

        if parts.is_empty() || parts.len() > 2 {
            println!("Please provied input on format: <MODBUS_REG> [VALUE]");
            continue;
        }

        let register: u16 = parts[0].parse().unwrap();
        if parts.len() == 1 {
            // Single register given -> Read it.

            let rsp = modbus_client.read(register, 1).await;
            match rsp {
                Ok(data) => {
                    println!("Read: {:?}", data[0]);
                }
                Err(_) => {
                    println!("Failed to read register: {register:?}");
                }
            }
        } else if parts.len() == 2 {
            // Register and data given -> Write it.
            let data: u16 = parts[1].parse().unwrap();
            let rsp = modbus_client.write(register, vec![data]).await;
            match rsp {
                Ok(_) => {
                    println!("Wrote {data:?} to {register:?}");
                }
                Err(_) => {
                    println!("Failed to write register: {register:?}");
                }
            }
        }
    }
}
