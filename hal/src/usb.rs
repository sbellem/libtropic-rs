#![allow(deprecated)] // Suppress aes-gcm warnings in tropic01

use std::env;
use std::fmt;
use std::io;
use std::thread;
use std::convert::TryInto;
use std::error::Error;
use std::time::Duration;
use std::array::TryFromSliceError;

use serialport;

use ed25519_dalek::Signature;
use ed25519_dalek::VerifyingKey;

use embedded_hal::spi::{ErrorType, SpiDevice, Error as SpiError, ErrorKind};
//use embedded_hal::spi::{Error as SpiError, ErrorKind};

use serialport::{DataBits, FlowControl, Parity, StopBits};
//use sha2::Digest as _;

use tropic01::{Error as TropicError, Tropic01};


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

/// Error module

#[derive(Debug)]
pub enum SerialTransportError {
    Io(io::Error),
    InvalidResponse,
    DataTooLong,
    NonUtf8Hex,
    InvalidHexDigit,
    InvalidBufferLength,
    // Box is needed to break the recursion
    Tropic(TropicError<Box<SerialTransportError>, std::convert::Infallible>),
}

impl fmt::Display for SerialTransportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => write!(f, "USB/Serial I/O error: {}", err),
            Self::InvalidResponse => write!(f, "Invalid response from device"),
            Self::DataTooLong => write!(f, "Data too long for transport"),
            Self::NonUtf8Hex => write!(f, "Non-UTF8 hex characters in response"),
            Self::InvalidHexDigit => write!(f, "Invalid hex digit in response"),
            Self::InvalidBufferLength => write!(f, "Invalid buffer length"),
            Self::Tropic(err) => write!(f, "Tropic device error: {}", err),
        }
    }
}

impl Error for SerialTransportError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(err) => Some(err),
            Self::Tropic(err) => Some(err),
            _ => None,
        }
    }
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

impl From<TryFromSliceError> for SerialTransportError {
    fn from(_err: TryFromSliceError) -> Self {
        Self::InvalidBufferLength
    }
}

impl From<TropicError<SerialTransportError, std::convert::Infallible>> for SerialTransportError {
    fn from(err: TropicError<SerialTransportError, std::convert::Infallible>) -> Self {
        match err {
            // Special case for BusError which has a SerialTransportError inside
            TropicError::BusError(inner) => Self::Tropic(
                TropicError::BusError(Box::new(inner))
            ),
            // For variants that don't contain SerialTransportError, we can map directly
            TropicError::AlarmMode => Self::Tropic(TropicError::AlarmMode),
            TropicError::ChipBusy => Self::Tropic(TropicError::ChipBusy),
            TropicError::Decryption(e) => Self::Tropic(TropicError::Decryption(e)),
            TropicError::Encryption(e) => Self::Tropic(TropicError::Encryption(e)),
            TropicError::GPIOError(_) => Self::InvalidResponse, // Infallible
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
