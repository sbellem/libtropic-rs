use std::env;

/// Example to verify start a secure session
use rand_core::OsRng;
use tropic01::MCounterIndex;
use tropic01::Tropic01;
use tropic01::X25519Dalek;
use tropic01::keys::SH0PRIV_PROD;
use tropic01::keys::SH0PUB_PROD;
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
    let shpub = SH0PUB_PROD.into();
    let shpriv = SH0PRIV_PROD.into();
    tropic.session_start(&X25519Dalek, shpub, shpriv, ehpub, ehpriv, 0)?;

    println!("Started secure session with sh0pub key ...");

    println!("read monotonic counter from index 0");
    let res = tropic.mcounter_get(MCounterIndex::Index0)?;
    println!("counter value at index 0: {:?}", res);

    let c = 0;
    println!("init monotonic counter to {c}");
    let _ = tropic.mcounter_init(MCounterIndex::Index0, c)?;
    let res = tropic.mcounter_get(MCounterIndex::Index0)?;
    println!("counter is now: {:?}", res);

    let c = 3;
    println!("init monotonic counter to {c}");
    let _ = tropic.mcounter_init(MCounterIndex::Index0, c)?;
    let res = tropic.mcounter_get(MCounterIndex::Index0)?;
    println!("counter is now: {:?}", res);

    println!("decrement counter ...");
    let _ = tropic.mcounter_update(MCounterIndex::Index0)?;
    let res = tropic.mcounter_get(MCounterIndex::Index0)?;
    println!("counter is now: {:?}", res);

    let _ = tropic.mcounter_init(MCounterIndex::Index0, 0)?;

    Ok(())
}
