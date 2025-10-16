#![allow(deprecated)] // Suppress aes-gcm warnings in tropic01

use std::env;
use std::thread;
use std::time::Duration;
use std::convert::TryInto;

use embedded_hal::spi::{ErrorType, SpiDevice, Error as SpiError, ErrorKind};
use serialport::{DataBits, FlowControl, Parity, StopBits};
use tropic01::{Error as TropicError, Tropic01};
use std::io;

use tropic01_example_usb::ChipId;

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
    Tropic(TropicError<Box<SerialTransportError>, std::convert::Infallible>),
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

impl From<TropicError<SerialTransportError, std::convert::Infallible>> for SerialTransportError {
    fn from(err: TropicError<SerialTransportError, std::convert::Infallible>) -> Self {
        match err {
            TropicError::BusError(inner) => Self::Tropic(TropicError::BusError(Box::new(inner))),
            TropicError::AlarmMode => Self::Tropic(TropicError::AlarmMode),
            TropicError::ChipBusy => Self::Tropic(TropicError::ChipBusy),
            TropicError::Decryption(e) => Self::Tropic(TropicError::Decryption(e)),
            TropicError::Encryption(e) => Self::Tropic(TropicError::Encryption(e)),
            TropicError::GPIOError(_) => Self::InvalidResponse, // Infallible cannot occur
            TropicError::HandshakeFailed => Self::Tropic(TropicError::HandshakeFailed),
            TropicError::InvalidChipStatus(e) => Self::Tropic(TropicError::InvalidChipStatus(e)),
            TropicError::InvalidCRC => Self::Tropic(TropicError::InvalidCRC),
            TropicError::InvalidKey => Self::Tropic(TropicError::InvalidKey),
            TropicError::InvalidL2Response => Self::Tropic(TropicError::InvalidL2Response),
            TropicError::InvalidL3Cmd => Self::Tropic(TropicError::InvalidL3Cmd),
            TropicError::InvalidPublicKey => Self::Tropic(TropicError::InvalidPublicKey),
            TropicError::L2ResponseError(e) => Self::Tropic(TropicError::L2ResponseError(e)),
            TropicError::L3CmdFailed => Self::Tropic(TropicError::L3CmdFailed),
            TropicError::L3ResponseBufferOverflow => Self::Tropic(TropicError::L3ResponseBufferOverflow),
            TropicError::NoSession => Self::Tropic(TropicError::NoSession),
            TropicError::ParsingError(e) => Self::Tropic(TropicError::ParsingError(e)),
            TropicError::RequestExceedsSize => Self::Tropic(TropicError::RequestExceedsSize),
            TropicError::Unauthorized => Self::Tropic(TropicError::Unauthorized),
            TropicError::UnexpectedResponseStatus => Self::Tropic(TropicError::UnexpectedResponseStatus),
        }
    }
}

pub struct SerialTransport {
    port: Box<dyn serialport::SerialPort>,
}

impl SerialTransport {
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
    fn transaction(&mut self, operations: &mut [embedded_hal::spi::Operation<'_, u8>]) -> Result<(), Self::Error> {
        self.cs_high()?;
        for op in operations {
            match op {
                embedded_hal::spi::Operation::Write(data) => {
                    let mut buf = data.to_vec();
                    self.transfer(&mut buf)?;
                }
                embedded_hal::spi::Operation::Transfer(read, write) => {
                    if read.len() != write.len() {
                        return Err(SerialTransportError::InvalidBufferLength);
                    }
                    read.copy_from_slice(write);
                    self.transfer(read)?;
                }
                embedded_hal::spi::Operation::TransferInPlace(data) => {
                    self.transfer(data)?;
                }
                embedded_hal::spi::Operation::Read(data) => {
                    data.fill(0);
                    self.transfer(data)?;
                }
                embedded_hal::spi::Operation::DelayNs(_ns) => {
                    thread::sleep(Duration::from_nanos(1));
                }
            }
        }
        self.cs_high()?;
        Ok(())
    }
}

fn main() -> Result<(), SerialTransportError> {
    env_logger::init();

    let args: Vec<String> = env::args().collect();
    let port_name = args.get(1).cloned().unwrap_or_else(|| "/dev/ttyACM0".to_string());
    let baud_rate = args
        .get(2)
        .and_then(|s| s.parse().ok())
        .filter(|&r| [4800, 9600, 19200, 38400, 115200].contains(&r))
        .unwrap_or(115200);

    println!("Opening TS1302 dongle on {} @ {} baud", port_name, baud_rate);

    let transport = SerialTransport::new(&port_name, baud_rate)?;
    let mut tropic01 = Tropic01::new(transport);

    let chip_id = tropic01.get_info_chip_id()?;
    let chip_info = ChipId::try_from(&chip_id[..])
        .map_err(|e| {
            println!("Failed to parse chip ID: {}", e);
            SerialTransportError::InvalidResponse
        })?;

    chip_info.print_details();

    let res = tropic01.get_info_cert()?;
    println!("Cert: {res:x?}");

    println!("Example completed successfully!");
    Ok(())
}
