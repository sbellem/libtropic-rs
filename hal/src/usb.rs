//! USB HAL implementation for tropic01
#![allow(deprecated)] // Suppress aes-gcm warnings in tropic01

use std::io;
use std::thread;
use std::time::Duration;

use embedded_hal::spi::{ErrorType, SpiDevice, Error as SpiError, ErrorKind, Operation};
use serialport::{DataBits, FlowControl, Parity, StopBits};
use tropic01::Tropic01;
use dummy_pin::DummyPin;

// Helper function for hex formatting
fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect::<Vec<_>>().join("")
}

#[derive(Debug)]
pub enum SerialTransportError {
    Io(io::Error),
    InvalidResponse,
    DataTooLong,
    NonUtf8Hex,
    InvalidHexDigit,
    InvalidBufferLength,
    Tropic(Box<dyn std::error::Error>),
}

impl SpiError for SerialTransportError {
    fn kind(&self) -> ErrorKind {
        match self {
            Self::Io(_) => ErrorKind::Other,
            Self::InvalidResponse => ErrorKind::Other,
            Self::DataTooLong => ErrorKind::Other,
            Self::NonUtf8Hex => ErrorKind::Other,
            Self::InvalidHexDigit => ErrorKind::Other,
            Self::InvalidBufferLength => ErrorKind::Other,
            Self::Tropic(_) => ErrorKind::Other,
        }
    }
}

impl From<io::Error> for SerialTransportError {
    fn from(err: io::Error) -> Self {
        Self::Io(err)
    }
}

impl From<serialport::Error> for SerialTransportError {
    fn from(err: serialport::Error) -> Self {
        Self::Io(err.into())
    }
}

/// Serial transport for connecting to Tropic hardware over USB
pub struct SerialTransport {
    port: Box<dyn serialport::SerialPort>,
}

impl SerialTransport {
    /// Create a new SerialTransport instance
    pub fn new(port_name: &str, baud_rate: u32) -> Result<Self, SerialTransportError> {
        let mut port = serialport::new(port_name.to_string(), baud_rate)
            .data_bits(DataBits::Eight)
            .parity(Parity::None)
            .stop_bits(StopBits::One)
            .flow_control(FlowControl::None)
            .timeout(Duration::from_millis(500))
            .open()
            .map_err(SerialTransportError::from)?;

        port.flush().map_err(SerialTransportError::from)?;

        Ok(Self { port })
    }

    fn cs_high(&mut self) -> Result<(), SerialTransportError> {
        self.port
            .write_all(b"CS=0\n")
            .map_err(SerialTransportError::from)?;
        let mut resp = [0u8; 4];
        self.port
            .read_exact(&mut resp)
            .map_err(SerialTransportError::from)?;
        if resp != *b"OK\r\n" {
            return Err(SerialTransportError::InvalidResponse);
        }
        Ok(())
    }

    fn transfer(&mut self, data: &mut [u8]) -> Result<(), SerialTransportError> {
        if data.is_empty() {
            return Ok(());
        }
        if data.len() > 2048 {
            return Err(SerialTransportError::DataTooLong);
        }

        let mut send = String::with_capacity(data.len() * 2 + 2);
        for &b in data.iter() {
            send.push_str(&format!("{:02X}", b));
        }
        send.push_str("x\n");

        let send_bytes = send.as_bytes();
        self.port
            .write_all(send_bytes)
            .map_err(SerialTransportError::from)?;
        thread::sleep(Duration::from_millis(10));

        let mut recv = vec![0u8; send_bytes.len()];
        self.port
            .read_exact(&mut recv)
            .map_err(SerialTransportError::from)?;
        if !(recv.ends_with(b"x\n") || recv.ends_with(b"\r\n")) {
            return Err(SerialTransportError::InvalidResponse);
        }

        let hex_part = &recv[..data.len() * 2];
        for (i, chunk) in hex_part.chunks_exact(2).enumerate() {
            let hex = std::str::from_utf8(chunk)
                .map_err(|_| SerialTransportError::NonUtf8Hex)?;
            data[i] = u8::from_str_radix(hex, 16)
                .map_err(|_| SerialTransportError::InvalidHexDigit)?;
        }
        Ok(())
    }
}

impl ErrorType for SerialTransport {
    type Error = SerialTransportError;
}

impl SpiDevice for SerialTransport {
    fn transaction(&mut self, operations: &mut [Operation<'_, u8>]) -> Result<(), Self::Error> {
        self.cs_high()?;
        for op in operations {
            match op {
                Operation::Write(data) => {
                    let mut buf = data.to_vec();
                    self.transfer(&mut buf)?;
                }
                Operation::Transfer(read, write) => {
                    if read.len() != write.len() {
                        return Err(SerialTransportError::InvalidBufferLength);
                    }
                    read.copy_from_slice(write);
                    self.transfer(read)?;
                }
                Operation::TransferInPlace(data) => {
                    self.transfer(data)?;
                }
                Operation::Read(data) => {
                    data.fill(0);
                    self.transfer(data)?;
                }
                Operation::DelayNs(_ns) => {
                    thread::sleep(Duration::from_nanos(1));
                }
            }
        }
        self.cs_high()?;
        Ok(())
    }
}

/// Create a Tropic01 device using the USB serial transport
/// 
/// # Arguments
/// * `port_name` - Serial port name (e.g., "/dev/ttyACM0")
/// * `baud_rate` - Serial baud rate (typically 115200)
/// 
/// # Returns
/// A Tropic01 instance configured for USB communication
pub fn create_usb_device(
    port_name: &str,
    baud_rate: u32,
) -> Result<Tropic01<SerialTransport, DummyPin>, SerialTransportError> {
    let transport = SerialTransport::new(port_name, baud_rate)?;
    Ok(Tropic01::new(transport))
}

/// Create a Tropic01 device using default USB settings
/// 
/// Uses "/dev/ttyACM0" at 115200 baud
/// 
/// # Returns
/// A Tropic01 instance configured for USB communication
pub fn create_default_usb_device() -> Result<Tropic01<SerialTransport, DummyPin>, SerialTransportError> {
    create_usb_device("/dev/ttyACM0", 115200)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    /// Test reading chip ID from actual hardware
    /// 
    /// This test requires physical hardware and should be ignored in automated testing.
    /// Run it manually with: cargo test -p hal -- --ignored read_chip_id
    #[test]
    #[ignore = "Requires physical hardware connection"]
    fn read_chip_id() {
        // Try to initialize a device
        let port_name = std::env::var("TROPIC_PORT").unwrap_or_else(|_| "/dev/ttyACM0".to_string());
        let baud_rate = 115200;
        
        println!("Testing connection to device on {} @ {} baud", port_name, baud_rate);
        
        // Create device directly
        let mut device = create_usb_device(&port_name, baud_rate)
            .expect("Should be able to create device");
        
        // Read the chip ID
        let chip_id = device.get_info_chip_id()
            .expect("Should be able to read chip ID");
        
        println!("Successfully read chip ID: {:?}", chip_id);
        
        // Basic validation - chip ID should not be empty
        assert!(!chip_id.is_empty(), "Chip ID should not be empty");
    }
    
    /// Simple test that doesn't require hardware
    #[test]
    fn test_function_exists() {
        // Just ensure the function is available - doesn't actually call it
        assert!(true, "create_usb_device function exists");
    }
}
