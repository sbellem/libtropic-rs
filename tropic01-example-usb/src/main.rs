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

use rand_core::OsRng;
use serialport::{DataBits, FlowControl, Parity, StopBits};
use sha2::Digest as _;

use tropic01::{Error as TropicError, Tropic01};
use tropic01::EccCurve;
use tropic01::X25519Dalek;
use tropic01::keys::SH0PRIV;
use tropic01::keys::SH0PUB;

use tropic01_example_usb::ChipId;
use tropic01_example_usb::cert::Cert;

use x25519_dalek::PublicKey;
use x25519_dalek::StaticSecret;


// Helper function for hex formatting
fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect::<Vec<_>>().join("")
}

/// XXX

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
/// XXX

/*
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
*/
/*
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
*/

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

use x509_parser::parse_x509_der;

fn parse_and_print_cert(cert_bytes: &[u8]) {
    match parse_x509_der(cert_bytes) {
        Ok((_, cert)) => {
            println!("X.509 Certificate Parsed:");
            println!("  Subject: {}", cert.subject());
            println!("  Issuer: {}", cert.issuer());
            println!("  Serial: {}", cert.serial);
            println!("  Validity: From {} to {}", cert.validity().not_before, cert.validity().not_after);
            println!("  Signature Algorithm OID: {:?}", cert.signature_algorithm.oid());
            println!("  Public Key Algorithm OID: {:?}", cert.public_key().algorithm.oid());
            println!("  Extensions:");
            for ext in cert.extensions() {
                println!("    OID: {:?} Critical: {} Value: {:?}", ext.oid, ext.critical, ext.value);
            }
        }
        Err(e) => {
            println!("Certificate parse error: {:?}", e);
        }
    }
}


/*
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

    /// Get the certificate
    //let res = tropic01.get_info_cert()?;
    //println!("Cert: {res:x?}");
    //println!("Cert data: {:?}", res.as_bytes());
    //
    //// Print the raw certificate bytes as hex
    //println!("Certificate (hex): {}", hex::encode(res.as_bytes()));
    //println!("Cert public key: {:?}", res.public_key());

    let res = tropic01.get_info_cert_store()?;
    //println!("Cert store: {:?}", res);
    println!("Cert store sizes: {:?}\n", res.cert_len);
    //println!("Cert store cert 0: {:?}", res.certs[0]);
    //println!("Cert store cert 1: {:?}", res.certs[1]);
    //println!("Cert store cert 2: {:?}", res.certs[2]);
    //println!("Cert store cert 3: {:?}", res.certs[3]);

    /// XXX
    use x509_parser::parse_x509_der;
    use std::fs::File;
    use std::io::Write;

    const CERT_NAMES: [&str; 4] = [
        "t01_ese_cert",
        "t01_xxxx_ca_cert",
        "t01_ca_cert",
        "tropicsquare_root_ca_cert",
    ];
    
    let store = res;
    for (i, cert_buf) in store.certs.iter().enumerate() {
        let der = &cert_buf[..store.cert_len[i]];
        let len = der.len();
        println!("Certificate {}, DER ({} bytes)", i, len);
        
        //let (_, cert) = parse_x509_der(&der[..len]).expect("Failed to parse DER");
        let cert = Cert::from_der(&der, len).expect("DER parse failed");

        //println!("Serial: {}", cert.serial_hex());
        println!("Serial: {}", cert.parsed.serial);
        println!("Subject: {}", cert.subject());
        println!("PEM:\n{}", cert.to_pem());
        println!("Hex:\n{}", cert.to_hex());

        std::fs::write(format!("_{}.pem", CERT_NAMES[i]), cert.to_pem())?;
        std::fs::write(format!("{}.der", CERT_NAMES[i]), &cert.der)?;
        //let filename = format!("{}.der", CERT_NAMES[i]);
        //let mut file = File::create(&filename)?;
        //file.write_all(&der[..len])?;
        //println!("Wrote {} bytes to {}", len, filename);

        println!();
    }
    /// XXX

    println!("Example completed successfully!");
    Ok(())
}
*/

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
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



    let res = tropic01.get_info_chip_id()?;
    println!("ChipId: {res:x?}");
    let chip_id = res.to_vec();

    //println!("Sleep");
    //tropic01.sleep_req(tropic01::SleepReq::Sleep)?;

    let res = tropic01.get_info_cert()?;
    println!("Cert: {res:x?}");

    println!("Reboot");
    tropic01.startup_req(tropic01::StartupReq::Reboot)?;
    println!("Rebooted");

    //let res = tropic01.get_info_chip_id()?;
    //println!("ChipId after reboot: {res:x?}");
    //assert_eq!(res, &chip_id);

    let csprng = OsRng;
    let ehpriv = StaticSecret::random_from_rng(csprng);
    let ehpub = PublicKey::from(&ehpriv);
    let shpub = SH0PUB.into();
    let shpriv = SH0PRIV.into();
    println!("ehpub: {:?}", ehpub);
    println!("shpub: {:?}", shpub);
    //println!("shpriv: {}", shpriv);
    tropic01.session_start(&X25519Dalek, shpub, shpriv, ehpub, ehpriv, 0)?;

    let res = tropic01.get_random_value(6)?;
    println!("random value get: {res:x?}");

    let ping_data = b"";
    let res = tropic01.ping(ping_data)?;
    // Test empty data loopback
    assert_eq!(res, ping_data);

    let ping_data = [6; 4096];
    let res = tropic01.ping(&ping_data)?;
    // Test long data loopback
    assert_eq!(res, ping_data);

    /*
    let key_slot = 0.into();
    tropic01.ecc_key_generate(key_slot, EccCurve::P256)?;

    let res = tropic01.ecc_key_read(key_slot)?;
    println!("key read response: {res:x?}");

    let public_key =
        VerifyingKey::from_bytes(res.pub_key().try_into()?).expect("public key to be valid");

    // Signature of hash
    let msg = "hello tropic";
    let mut hasher = sha2::Sha256::new();
    hasher.update(msg);
    let hash: [u8; 32] = hasher.finalize().into();
    let signature = tropic01.eddsa_sign(key_slot, &hash)?;
    println!("signature of hash: {signature:x?}");
    public_key
        .verify_strict(&hash, &Signature::from_bytes(signature))
        .expect("signature to be verified");

    // Produce an unauthorized error to test nonce behavior
    if shpub.as_bytes() == &SH0PUB {
        assert!(matches!(
            tropic01.ecc_key_generate(3.into(), EccCurve::P256),
            Err(TropicError::Unauthorized)
        ));
    }

    // Signature of raw message
    let msg = "hello tropic".repeat(341);
    let msg = msg.as_bytes();
    let signature = tropic01.eddsa_sign(key_slot, msg)?;
    println!("signature of long raw msg: {signature:x?}");
    public_key
        .verify_strict(msg, &Signature::from_bytes(signature))
        .expect("signature to be verified");
    */
    Ok(())
}
