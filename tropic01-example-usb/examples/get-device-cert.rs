/// Example to verify the secure element's provisioned certificate chain
use std::env;

use tropic01::Tropic01;
use tropic01_example_usb::port::UsbDevice;
use utils::x509::PARSE_ERRORS_FATAL;
use utils::x509::print_x509_info;
use x509_parser::certificate::X509Certificate;
use x509_parser::prelude::FromDer; // from asn1_rs

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
    let device_cert_bytes = store.certs.first().expect("cert bytes");

    let x509 = match X509Certificate::from_der(device_cert_bytes) {
        Ok((_, x509)) => x509,
        Err(e) => {
            let msg = format!("Error while parsing cert bytes: {e}");
            if PARSE_ERRORS_FATAL {
                return Err(anyhow::anyhow!(msg));
            } else {
                log::error!("{}", msg);
                return Ok(());
            }
        },
    };
    print_x509_info(&x509)?;

    // TODO: Verify certificate signature

    Ok(())
}
