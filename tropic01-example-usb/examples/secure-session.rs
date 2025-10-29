/// Example to verify start a secure session

use rand_core::OsRng;
use std::env;

use tropic01::Tropic01;
use tropic01::X25519Dalek;
use tropic01::keys::SH0PRIV;
use tropic01::keys::SH0PUB;
use tropic01_example_usb::port::UsbDevice;

use x25519_dalek::PublicKey;
use x25519_dalek::StaticSecret;


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

    let csprng = OsRng;
    let ehpriv = StaticSecret::random_from_rng(csprng);
    let ehpub = PublicKey::from(&ehpriv);
    let shpub = SH0PUB.into();
    let shpriv = SH0PRIV.into();
    tropic.session_start(&X25519Dalek, shpub, shpriv, ehpub, ehpriv, 0)?;
    
    let res = tropic.get_random_value(6)?;
    println!("random value get: {res:x?}");
    
    let ping_data = b"";
    let res = tropic.ping(ping_data)?;
    // Test empty data loopback
    assert_eq!(res, ping_data);
    
    let ping_data = [6; 4096];
    let res = tropic.ping(&ping_data)?;
    // Test long data loopback
    assert_eq!(res, ping_data);

    Ok(())
}
