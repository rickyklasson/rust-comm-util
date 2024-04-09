use std::error::Error;
use std::io;
use std::time;
use tokio_modbus::client::sync::Reader;
use tokio_modbus::client::sync::Writer;
use tokio_modbus::{client::sync, Slave};
use tokio_serial;

pub mod modbus {
    use super::*;

    const CONNECTION_TIMEOUT: u64 = 500;

    pub struct Client {
        builder: tokio_serial::SerialPortBuilder,
        context: Option<sync::Context>,
    }

    impl Client {
        pub fn new(com_port: String, baudrate: u32) -> Client {
            let builder: tokio_serial::SerialPortBuilder = tokio_serial::new(com_port, baudrate);

            Client {
                builder,
                context: None,
            }
        }

        pub fn open(&mut self) -> Result<(), io::Error> {
            match sync::rtu::connect_slave_with_timeout(
                &self.builder,
                Slave(0x01),
                Some(time::Duration::from_millis(CONNECTION_TIMEOUT)),
            ) {
                Ok(ctx) => {
                    self.context = Some(ctx);
                    Ok(())
                }
                Err(e) => Err(e),
            }
        }

        pub fn close(&mut self) -> () {
            self.context = None;
        }

        pub fn read(&mut self, register: u16, count: u16) -> Result<Vec<u16>, Box<dyn Error>> {
            match &mut self.context {
                Some(ctx) => {
                    let rsp;
                    if register >= 30000 && register < 40000 {
                        rsp = ctx.read_input_registers(register - 30001, count)?;
                    } else if register >= 40000 && register < 50000 {
                        rsp = ctx.read_holding_registers(register - 40001, count)?;
                    } else {
                        return Err("Register outside valid register range...")?;
                    }

                    Ok(rsp)
                }
                None => Err("No context set for self. Did you forget to connect?")?,
            }
        }

        pub fn write(&mut self, register: u16, data: Vec<u16>) -> Result<(), Box<dyn Error>> {
            match &mut self.context {
                Some(ctx) => {
                    let rsp;
                    if register > 40001 && register < 50000 {
                        rsp = ctx.write_multiple_registers(register - 40001, &data)?;
                    } else {
                        return Err("Register outside valid register range...")?;
                    }
                    return Ok(rsp);
                }
                None => Err("No context set for self. Did you forget to connect?")?,
            }

            Ok(())
        }
    }
}

pub mod serial {
    pub fn get_serial_ports() -> Vec<String> {
        match tokio_serial::available_ports() {
            Ok(ports) => {
                let mut retval: Vec<String> = Vec::new();

                for p in ports {
                    retval.push(p.port_name);
                }
                retval
            }
            Err(_) => vec![],
        }
    }
}
