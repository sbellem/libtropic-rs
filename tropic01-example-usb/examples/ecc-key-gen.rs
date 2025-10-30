/// Example to verify start a secure session

use rand_core::OsRng;
use std::env;
//use std::{thread, time};

use tropic01::EccCurve;
use tropic01::Error;
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
    
    let key_slot = 0.into();

    match tropic.ecc_key_read(key_slot) {
        Ok(res) => {
            println!("key read response: {res:x?}");
        }
        Err(err) => {
            if matches!(err, Error::InvalidKey) {
                log::warn!("no ECC key in slot {}, skipping ECC operation", key_slot);
            } else {
                log::error!("ecc_key_read failed: {:?}", err);
            }
        }
    }

    println!("ecc key gen call ...");
    match tropic.ecc_key_generate(key_slot, EccCurve::P256) {
        Ok(res) => {
            println!("ecc key gen done: {res:x?}");
        }
        Err(err) => {
            if matches!(err, Error::ChipBusy) {
                log::error!("chip is busy, not sure why {:?}", err);
            } else {
                log::error!("ecc_key_generate error: {:?}", err);
            }
        }
    }

    //let ten_millis = time::Duration::from_millis(1000);
    //let now = time::Instant::now();
    //thread::sleep(ten_millis);

    //let res = tropic.ecc_key_read(key_slot)?;
    //println!("key read response: {res:x?}");

    //let public_key =
     //   VerifyingKey::from_bytes(res.pub_key().try_into()?).expect("public key to be valid");

    Ok(())
}
