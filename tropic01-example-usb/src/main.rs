#![allow(deprecated)] // Suppress aes-gcm warnings in tropic01

use std::env;

use ed25519_dalek::Signature;
use ed25519_dalek::VerifyingKey;
use rand_core::OsRng;
use sha2::Digest as _;
use tropic01::EccCurve;
use tropic01::Error;
use tropic01::Tropic01;
use tropic01::X25519Dalek;
use tropic01::keys::SH0PRIV_PROD;
use tropic01::keys::SH0PUB_PROD;
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
    let shpub = SH0PUB_PROD.into();
    let shpriv = SH0PRIV_PROD.into();
    tropic01.session_start(&X25519Dalek, shpub, shpriv, ehpub, ehpriv, 0)?;

    let res = tropic01.get_random_value(6)?;
    println!("random value get: {res:x?}");

    println!("{:-<79}", "");
    let ping_data = b"";
    println!("ping: {:?}", ping_data);
    let pong = tropic01.ping(ping_data)?;
    println!("pong: {:?}", pong);
    assert_eq!(pong, ping_data);

    println!("ping");
    let ping_data = [6; 4096];
    println!("ping: {:?}", ping_data);
    let pong = tropic01.ping(&ping_data)?;
    println!("pong: {:?}", pong);
    assert_eq!(pong, ping_data);

    println!("{:-<79}", "");
    let key_slot = 0.into();
    println!("ecc key read call ...");
    match tropic01.ecc_key_read(key_slot) {
        Ok(res) => {
            println!("key read response: {res:x?}");
        },
        Err(err) => {
            if matches!(err, Error::InvalidKey) {
                println!("no ECC key in slot {}, skipping ECC operation", key_slot);
            } else {
                log::error!("ecc_key_read failed: {:?}", err);
            }
        },
    }

    println!("{:-<79}", "");
    println!("ecc key erase call ...");
    match tropic01.ecc_key_erase(key_slot) {
        Ok(res) => {
            println!("ecc key erase done: {res:x?}");
        },
        Err(err) => {
            if matches!(err, Error::ChipBusy) {
                log::error!("chip is busy, not sure why {:?}", err);
            } else {
                log::error!("ecc_key_erase error: {:?}", err);
            }
        },
    }

    println!("{:-<79}", "");
    let key_slot = 0.into();
    println!("ecc key read call ...");
    match tropic01.ecc_key_read(key_slot) {
        Ok(res) => {
            println!("key read response: {res:x?}");
        },
        Err(err) => {
            if matches!(err, Error::InvalidKey) {
                println!("no ECC key in slot {}, skipping ECC operation", key_slot);
            } else {
                log::error!("ecc_key_read failed: {:?}", err);
            }
        },
    }

    println!("{:-<79}", "");
    println!("ecc key gen call ...");
    match tropic01.ecc_key_generate(key_slot, EccCurve::Ed25519) {
        Ok(res) => {
            println!("ecc key gen done: {res:x?}");
        },
        Err(err) => {
            if matches!(err, Error::ChipBusy) {
                log::error!("chip is busy, not sure why {:?}", err);
            } else {
                log::error!("ecc_key_generate error: {:?}", err);
            }
        },
    }

    let res = tropic01.ecc_key_read(key_slot)?;
    println!("key read response: {res:x?}");

    println!("pub key: {:?}", res.pub_key());
    let public_key: &[u8; 32] = res.pub_key().try_into()?;
    let public_key = VerifyingKey::from_bytes(public_key).expect("public key to be valid");
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
    //if shpub.as_bytes() == &SH0PUB_PROD {
    //    assert!(matches!(
    //        tropic01.ecc_key_generate(3.into(), EccCurve::P256),
    //        Err(Error::Unauthorized)
    //    ));
    //}

    // Signature of raw message
    let msg = "hello tropic".repeat(341);
    let msg = msg.as_bytes();
    let signature = tropic01.eddsa_sign(key_slot, msg)?;
    println!("signature of long raw msg: {signature:x?}");
    public_key
        .verify_strict(msg, &Signature::from_bytes(signature))
        .expect("signature to be verified");
    Ok(())
}
