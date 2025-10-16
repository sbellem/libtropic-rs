#![allow(deprecated)] // Suppress aes-gcm warnings in tropic01

use std::env;
use std::thread;
use std::time::Duration;

use embedded_hal::spi::{ErrorType, SpiDevice, Error as SpiError, ErrorKind};
use serialport::{DataBits, FlowControl, Parity, StopBits};
use tropic01::{Error as TropicError, Tropic01};
use std::io;

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

    // Parse chip ID based on lt_chip_id_t
    // CHIP_ID version (bytes 0-3, version as vX.Y.Z.W)
    let chip_id_ver = format!(
        "0x{:02x}{:02x}{:02x}{:02x} (v{}.{}.{}.{})",
        chip_id[0], chip_id[1], chip_id[2], chip_id[3],
        chip_id[0], chip_id[1], chip_id[2], chip_id[3]
    );
    println!("CHIP_ID ver: {}", chip_id_ver);

    // Factory level test info (bytes 4-19, hex)
    let fl_chip_info: String = chip_id[4..20]
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<String>>()
        .join("");
    println!("FL_PROD_DATA: 0x{} (N/A)", fl_chip_info);

    // Manufacturing test info (bytes 20-27, hex)
    let func_test_info: String = chip_id[20..28]
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<String>>()
        .join("");
    println!("MAN_FUNC_TEST: 0x{} (PASSED)", func_test_info);

    // Silicon revision (bytes 28-31, ASCII)
    let silicon_rev = String::from_utf8(chip_id[28..32].to_vec())
        .unwrap_or_else(|_| "Unknown".to_string());
    println!("Silicon rev: 0x{:02x}{:02x}{:02x}{:02x} ({})",
        chip_id[28], chip_id[29], chip_id[30], chip_id[31], silicon_rev);

    // Package type ID (bytes 32-33, hex)
    let packg_type_id: String = chip_id[32..34]
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<String>>()
        .join("");
    println!("Package ID: 0x{} (QFN32, 4x4mm)", packg_type_id);

    // Provisioning info (bytes 36-39, hex: version, fab ID, part number ID)
    let prov_info_ver = chip_id[36];
    let fab_id = u16::from_be_bytes([chip_id[37], chip_id[38] & 0x0f]); // 12-bit fab ID
    let pn_id = u16::from_be_bytes([(chip_id[38] & 0xf0) >> 4, chip_id[39]]); // 12-bit part number ID
    println!("Prov info ver: 0x{:02x} (v{})", prov_info_ver, prov_info_ver);
    println!("Fab ID: 0x{:03x} (EPS Global - Brno)", fab_id);
    println!("P/N ID (short P/N): 0x{:03x}", pn_id);

    // Provisioning date (bytes 40-41, u16 big-endian)
    let provisioning_date = u16::from_be_bytes([chip_id[40], chip_id[41]]);
    println!("Prov date: 0x{:04x}", provisioning_date);

    // HSM version (bytes 42-45, version as 0.Major.Minor.Patch)
    let hsm_ver = format!(
        "0x{:02x}{:02x}{:02x}{:02x} (v0.{}.{}.{})",
        chip_id[42], chip_id[43], chip_id[44], chip_id[45],
        chip_id[43], chip_id[44], chip_id[45]
    );
    println!("HSM HW/FW/SW ver: {}", hsm_ver);

    // Program version (bytes 46-49, version as 0.Major.Minor.Patch)
    let prog_ver = format!(
        "0x{:02x}{:02x}{:02x}{:02x} (v0.{}.{}.{})",
        chip_id[46], chip_id[47], chip_id[48], chip_id[49],
        chip_id[47], chip_id[48], chip_id[49]
    );
    println!("Programmer ver: {}", prog_ver);

    // Serial number (bytes 52-67, lt_ser_num_t)
    let serial_number = &chip_id[52..68];
    let sn = serial_number[0];
    let fab_data = u32::from_be_bytes([0, serial_number[1], serial_number[2], serial_number[3]]);
    let fab_id_sn = (fab_data >> 12) & 0xfff; // Upper 12 bits for Fab ID
    let pn_id_sn = fab_data & 0xfff; // Lower 12 bits for P/N ID
    let fab_date = u16::from_be_bytes([serial_number[4], serial_number[5]]);
    let lot_id: String = serial_number[6..11]
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<String>>()
        .join("");
    let wafer_id = serial_number[11];
    let x_coord = u16::from_be_bytes([serial_number[12], serial_number[13]]);
    let y_coord = u16::from_be_bytes([serial_number[14], serial_number[15]]);
    let serial_number_hex: String = serial_number
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<String>>()
        .join("");
    println!("S/N: 0x{}", serial_number_hex);
    println!("  SN: 0x{:02x}", sn);
    println!("  Fab ID: 0x{:03x}", fab_id_sn);
    println!("  P/N ID: 0x{:03x}", pn_id_sn);
    println!("  Fabrication Date: 0x{:04x}", fab_date);
    println!("  Lot ID: 0x{}", lot_id);
    println!("  Wafer ID: 0x{:02x}", wafer_id);
    println!("  X-Coordinate: 0x{:04x}", x_coord);
    println!("  Y-Coordinate: 0x{:04x}", y_coord);

    let part_num_data = &chip_id[68..84]; // 16 bytes, including length
    let pn_len = part_num_data[0] as usize;

    // Get ASCII string
    let pn_data = String::from_utf8(part_num_data[1..=pn_len].to_vec())
        .unwrap_or_else(|_| "Unknown".to_string());

    // Convert all bytes (including length byte) to uppercase hex
    let part_num_hex: String = part_num_data
        .iter()
        .map(|b| format!("{:02X}", b))
        .collect();

    println!("P/N (long) = 0x{} ({})", part_num_hex, pn_data);



    // Provisioning template version (bytes 84-85, u16 big-endian)
    let prov_templ_ver = u16::from_be_bytes([chip_id[84], chip_id[85]]);
    println!("Prov template ver: 0x{:04x} (v{}.{})", prov_templ_ver, prov_templ_ver >> 8, prov_templ_ver & 0xff);

    // Provisioning template tag (bytes 86-89, hex)
    let prov_templ_tag: String = chip_id[86..90]
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<String>>()
        .join("");
    println!("Prov template tag: 0x{}", prov_templ_tag);

    // Provisioning specification version (bytes 90-91, u16 big-endian)
    let prov_spec_ver = u16::from_be_bytes([chip_id[90], chip_id[91]]);
    println!("Prov specification ver: 0x{:04x} (v0.{})", prov_spec_ver, prov_spec_ver & 0xff);

    // Provisioning specification tag (bytes 92-95, hex)
    let prov_spec_tag: String = chip_id[92..96]
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<String>>()
        .join("");
    println!("Prov specification tag: 0x{}", prov_spec_tag);

    // Batch ID (bytes 96-100, hex)
    let batch_id: String = chip_id[96..101]
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect::<Vec<String>>()
        .join("");
    println!("Batch ID: 0x{}", batch_id);

    println!("Example completed successfully!");
    Ok(())
}
