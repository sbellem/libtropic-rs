/// Example to verify the secure element's provisioned certificate chain
use arrayvec::ArrayVec;

use std::env;
//use std::fs::File;
//use std::io::Write;

use tropic01::Tropic01;
use tropic01_example_usb::cert::Cert;
use tropic01_example_usb::cert::NUM_CERTIFICATES;
use tropic01_example_usb::port::UsbDevice;

use x509_parser::prelude::{FromDer, X509Certificate};
use x509_parser::x509::SubjectPublicKeyInfo;

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

    let store = tropic.get_info_cert_store()?;
    println!("Cert store sizes: {:?}\n", store.cert_len);

    //const CERT_NAMES: [&str; 4] = [
    //    "t01_ese_cert",
    //    "t01_xxxx_ca_cert",
    //    "t01_ca_cert",
    //    "tropicsquare_root_ca_cert",
    //];

    let mut certs: ArrayVec<X509Certificate<'_>, NUM_CERTIFICATES> = ArrayVec::new();
    for (i, cert_buf) in store.certs.iter().enumerate().rev() {
        let der = &cert_buf[..store.cert_len[i]];
        let len = der.len();
        println!("------------------------------------------------------------------");
        println!("Certificate {}, DER ({} bytes)", i, len);

        let cert = Cert::from_der(&der, len).expect("DER parse failed");
        let _ = cert.print_min_info();
        let _ = cert.print_basic_info();
        let _ = cert.print_extension_info();
        let _ = cert.print_validation_info();

        let (_, _cert) = X509Certificate::from_der(der).expect("DER parse failed");
        certs.push(_cert.clone());

        if i == 3 {
            let issuer_cert = certs.get(0).unwrap();
            let res = cert.print_verification_info(issuer_cert);
        } else {
            let issuer_cert = certs.get(2 - i).unwrap();
            let res = cert.print_verification_info(issuer_cert);
        }

        println!();
    }

    Ok(())
}
