use std::error::Error;
use tokio_modbus::client::Reader;
use tokio_modbus::client::Writer;
use tokio_modbus::{client, Slave};

pub mod modbus {
    use super::*;

    pub struct Client {
        builder: tokio_serial::SerialPortBuilder,
        context: Option<client::Context>,
    }

    impl Client {
        pub fn new(com_port: String, baudrate: u32) -> Client {
            let builder: tokio_serial::SerialPortBuilder = tokio_serial::new(com_port, baudrate);

            Client {
                builder,
                context: None,
            }
        }

        pub fn open(&mut self) -> Result<(), tokio_serial::Error> {
            match tokio_serial::SerialStream::open(&self.builder) {
                Ok(stream) => {
                    self.context = Some(client::rtu::attach_slave(stream, Slave(0x01)));
                    Ok(())
                }
                Err(e) => Err(e),
            }
        }

        pub fn close(&mut self) {
            self.context = None;
        }

        pub async fn read(
            &mut self,
            register: u16,
            count: u16,
        ) -> Result<Vec<u16>, Box<dyn Error>> {
            match &mut self.context {
                Some(ctx) => {
                    let rsp;
                    if (30000..40000).contains(&register) {
                        rsp = ctx.read_input_registers(register - 30001, count).await?;
                    } else if (40000..50000).contains(&register) {
                        rsp = ctx.read_holding_registers(register - 40001, count).await?;
                    } else {
                        return Err("Register outside valid register range...")?;
                    }

                    Ok(rsp)
                }
                None => Err("No context set for self. Did you forget to connect?")?,
            }
        }

        pub async fn write(&mut self, register: u16, data: Vec<u16>) -> Result<(), Box<dyn Error>> {
            match &mut self.context {
                Some(ctx) => {
                    if register > 40001 && register < 50000 {
                        ctx.write_multiple_registers(register - 40001, &data)
                            .await?
                    } else {
                        Err("Register outside valid register range...")?
                    }
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
