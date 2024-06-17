use std::error::Error;
use std::net;
use tokio::time;
use tokio_modbus::client::Reader;
use tokio_modbus::client::Writer;
use tokio_modbus::{client, Slave};

pub mod modbus {
    use std::str::FromStr;

    use super::*;

    const TIMEOUT: time::Duration = time::Duration::from_millis(250);

    pub enum ModbusClientType {
        Rtu(tokio_serial::SerialPortBuilder),
        Tcp(std::net::SocketAddr),
    }

    pub struct Client {
        client_type: ModbusClientType,
        context: Option<client::Context>,
    }

    impl Client {
        pub fn new(com_port: &String, baudrate: u32, ip: &String) -> Client {
            if !com_port.is_empty() {
                // Serial port client request
                let builder: tokio_serial::SerialPortBuilder =
                    tokio_serial::new(com_port, baudrate);

                Client {
                    client_type: ModbusClientType::Rtu(builder),
                    context: None,
                }
            } else {
                // TCP client request
                let ipv4_addr = match net::IpAddr::from_str(ip) {
                    Ok(val) => val,
                    Err(_) => net::IpAddr::from_str("0.0.0.0").unwrap(),
                };
                let socket_addr = net::SocketAddr::new(ipv4_addr, 502);

                Client {
                    client_type: ModbusClientType::Tcp(socket_addr),
                    context: None,
                }
            }
        }

        pub async fn open(&mut self) -> Result<(), tokio_serial::Error> {
            match &self.client_type {
                ModbusClientType::Rtu(builder) => match tokio_serial::SerialStream::open(builder) {
                    Ok(stream) => {
                        self.context = Some(client::rtu::attach_slave(stream, Slave(0x01)));
                        Ok(())
                    }
                    Err(e) => Err(e),
                },
                ModbusClientType::Tcp(socket_addr) => {
                    self.context = Some(client::tcp::connect(*socket_addr).await?);
                    Ok(())
                }
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
                        rsp = time::timeout(
                            TIMEOUT,
                            ctx.read_input_registers(register - 30001, count),
                        )
                        .await?;

                        match rsp {
                            Ok(val) => Ok(val),
                            Err(_) => Err("Timeout during read request")?,
                        }
                    } else if (40000..50000).contains(&register) {
                        rsp = time::timeout(
                            TIMEOUT,
                            ctx.read_holding_registers(register - 40001, count),
                        )
                        .await?;

                        match rsp {
                            Ok(val) => Ok(val),
                            Err(_) => Err("Timeout during read request")?,
                        }
                    } else {
                        return Err("Register outside valid register range...")?;
                    }
                }
                None => Err("No context set for self. Did you forget to connect?")?,
            }
        }

        pub async fn write(&mut self, register: u16, data: Vec<u16>) -> Result<(), Box<dyn Error>> {
            match &mut self.context {
                Some(ctx) => {
                    if register > 40001 && register < 50000 {
                        let rsp = time::timeout(
                            TIMEOUT,
                            ctx.write_multiple_registers(register - 40001, &data),
                        )
                        .await?;

                        match rsp {
                            Ok(val) => val,
                            Err(_) => Err("Timeout during write request")?,
                        };
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
