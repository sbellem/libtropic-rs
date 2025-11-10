#![allow(deprecated)] // Suppress aes-gcm warnings in tropic01

use std::array::TryFromSliceError;
use std::convert::TryInto;
use std::env;
use std::error::Error;
use std::fmt;
use std::io;
use std::thread;
use std::time::Duration;

use ed25519_dalek::Signature;
use ed25519_dalek::VerifyingKey;
use embedded_hal::spi::Error as SpiError;
use embedded_hal::spi::ErrorKind;
use embedded_hal::spi::ErrorType;
use embedded_hal::spi::SpiDevice;
//use embedded_hal::spi::{Error as SpiError, ErrorKind};
use rand_core::OsRng;
use serialport;
use serialport::DataBits;
use serialport::FlowControl;
use serialport::Parity;
use serialport::StopBits;
use sha2::Digest as _;
use tropic01::EccCurve;
use tropic01::Error as TropicError;
use tropic01::Tropic01;
use tropic01::X25519Dalek;
use tropic01::keys::SH0PRIV;
use tropic01::keys::SH0PUB;
use tropic01_example_usb::cert::Cert;
use tropic01_example_usb::chipid::ChipId;
use tropic01_example_usb::port::UsbDevice;
use x509_parser::parse_x509_der;
use x25519_dalek::PublicKey;
use x25519_dalek::StaticSecret;

fn parse_and_print_cert(cert_bytes: &[u8]) {
    match parse_x509_der(cert_bytes) {
        Ok((_, cert)) => {
            println!("X.509 Certificate Parsed:");
            println!("  Subject: {}", cert.subject());
            println!("  Issuer: {}", cert.issuer());
            println!("  Serial: {}", cert.serial);
            println!(
                "  Validity: From {} to {}",
                cert.validity().not_before,
                cert.validity().not_after
            );
            println!(
                "  Signature Algorithm OID: {:?}",
                cert.signature_algorithm.oid()
            );
            println!(
                "  Public Key Algorithm OID: {:?}",
                cert.public_key().algorithm.oid()
            );
            println!("  Extensions:");
            for ext in cert.extensions() {
                println!(
                    "    OID: {:?} Critical: {} Value: {:?}",
                    ext.oid, ext.critical, ext.value
                );
            }
        },
        Err(e) => {
            println!("Certificate parse error: {:?}", e);
        },
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
    let port_name = args
        .get(1)
        .cloned()
        .unwrap_or_else(|| "/dev/ttyACM0".to_string());

    let baud_rate = args
        .get(2)
        .and_then(|s| s.parse().ok())
        .filter(|&r| [4800, 9600, 19200, 38400, 115200].contains(&r))
        .unwrap_or(115200);

    println!(
        "Opening TS1302 dongle on {} @ {} baud",
        port_name, baud_rate
    );

    let transport = UsbDevice::new(&port_name, baud_rate)?;
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
