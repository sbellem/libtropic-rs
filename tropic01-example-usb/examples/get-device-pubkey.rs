/// Example to verify the secure element's provisioned certificate chain

use std::env;

use tropic01::Tropic01;
use tropic01_example_usb::port::UsbDevice;

use utils::x509::print_hex_dump;
use utils::x509::print_x509_info;
use utils::x509::PARSE_ERRORS_FATAL;

use x509_parser::asn1_rs::Any;
use x509_parser::asn1_rs::Input;
use x509_parser::parse_x509_certificate;
//use x509_parser::certificate::X509Certificate;
//use x509_parser::prelude::FromDer; // from asn1_rs
use x509_parser::public_key::PublicKey; // from asn1_rs
use x509_parser::prelude::DerParser;

//use x25519_dalek::{PublicKey, StaticSecret};
use x25519_dalek;


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

    let usb_device = UsbDevice::new(&port_name, baud_rate)?;
    let mut tropic = Tropic01::new(usb_device);

    //let cert = tropic.get_info_cert()?;
    //println!("pk 2: {:?}", cert.public_key());
    let store = tropic.get_info_cert_store()?;
    let device_cert_bytes = store.certs.first().expect("cert bytes");

    //println!("device cert: {:?}", cert);
    //println!("device cert data: {:?}", cert.as_bytes());

    //let res = X509Certificate::from_der(cert.as_bytes());
    //println!("x509: {:?}", res);

    //let x509 = match X509Certificate::from_der(cert.as_bytes()) {
    //let x509 = match X509Certificate::from_der(device_cert_bytes) {
    let x509 = match parse_x509_certificate(device_cert_bytes) {
        Ok((_, x509)) => x509,
        Err(e) => {
            let msg = format!("Error while parsing cert bytes: {e}");
            if PARSE_ERRORS_FATAL {
                return Err(anyhow::anyhow!(msg));
            } else {
                log::error!("{}", msg);
                return Ok(());
            }
        }
    };
    print_x509_info(&x509)?;

    let pk = x509.public_key();
    println!("public key algo: {:?}:", pk.algorithm);
    println!("pk: {:?}", pk.parsed());


    let spki = &x509.tbs_certificate.subject_pki; // SubjectPublicKeyInfo
    // subject_public_key is a bit string object; the raw bytes are in `.data`
    //let cert_pubkey_bytes = &spki.subject_public_key.data;
    let cert_pubkey_bytes = &spki.parsed();

    // Compare lengths + contents
    if cert_pubkey_bytes == &pk.parsed() {
        println!("Public key bytes match the certificate's subject public key");
    } else {
        println!("Mismatch: cert pubkey = {:02X?}, raw key = {:02X?}",
                 cert_pubkey_bytes, pk);
    }

    //let st_pub = PublicKey::from(cert_pubkey_bytes);
    //let parsed_pk = pk.parsed().expect("pk parsed");
    //println!("parsed pk: {:?}", parsed_pk);


    match pk.parsed() {
        Ok(PublicKey::Unknown(b)) => {
            println!("    Unknown key type");
            print_hex_dump(b, 256);

            let mut pubkey_arr = [0u8; 32];
            pubkey_arr.copy_from_slice(b);
            let peer_pk = x25519_dalek::PublicKey::from(pubkey_arr);
            println!("peer_pk: {:?}", peer_pk);

            if let Ok((rem, res)) = Any::parse_der(Input::from(b)) {
                eprintln!("rem: {} bytes", rem.len());
                eprintln!("res: {res:?}");
            } else {
                eprintln!("      <Could not parse key as DER>");
            }
        }
        Err(_) => {
            println!("    INVALID PUBLIC KEY");
        },
        _ => println!("...")
    }

    Ok(())
}
