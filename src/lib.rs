use std::error::Error;
use std::io;
use std::time;
use tokio_modbus::client::sync::Reader;
use tokio_modbus::{client::sync, Slave};
use tokio_serial;

const CONNECTION_TIMEOUT: u64 = 500;

pub struct ModbusClient {
    builder: tokio_serial::SerialPortBuilder,
    context: Option<sync::Context>,
}

impl ModbusClient {
    pub fn new(com_port: String, baudrate: u32) -> ModbusClient {
        let builder: tokio_serial::SerialPortBuilder = tokio_serial::new(com_port, baudrate);

        ModbusClient {
            builder,
            context: None,
        }
    }

    pub fn connect(&mut self) -> Result<(), io::Error> {
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

    pub fn read(&mut self, register: u16, count: u16) -> Result<Vec<u16>, Box<dyn Error>> {
        match &mut self.context {
            Some(ctx) => {
                let retval: Vec<u16>;
                let rsp;
                if register >= 30000 && register < 40000 {
                    rsp = ctx.read_input_registers(register - 30001, count)?;
                } else if register >= 40000 && register < 50000 {
                    rsp = ctx.read_holding_registers(register - 40001, count)?;
                } else {
                    return Err("Register outside valid register range...")?;
                }

                retval = rsp?;
                Ok(retval)
            }
            None => Err("No context set for self. Did you forget to connect?")?,
        }
    }
}
