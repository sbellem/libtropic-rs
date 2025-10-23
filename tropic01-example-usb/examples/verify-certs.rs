/// Example to verify the secure element's provisioned certificate chain
use std::env;
use std::fs::File;
use std::io::Write;

use tropic01::Tropic01;
use tropic01_example_usb::cert::Cert;
use tropic01_example_usb::port::UsbDevice;

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

    let usb_device = UsbDevice::new(&port_name, baud_rate)?;
    let mut tropic = Tropic01::new(usb_device);

    /// XXX
    let res = tropic.get_info_cert_store()?;
    //println!("Cert store: {:?}", res);
    println!("Cert store sizes: {:?}\n", res.cert_len);
    //println!("Cert store cert 0: {:?}", res.certs[0]);
    //println!("Cert store cert 1: {:?}", res.certs[1]);
    //println!("Cert store cert 2: {:?}", res.certs[2]);
    //println!("Cert store cert 3: {:?}", res.certs[3]);

    /// XXX

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

    Ok(())
}
