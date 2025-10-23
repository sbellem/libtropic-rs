use hex;
use pem::Pem;
use std::fmt;
use x509_parser::prelude::{FromDer, X509Certificate};

/// Certificate struct with DER, PEM, hex, and parsed metadata.
#[derive(Debug, Clone)]
pub struct Cert {
    pub der: Vec<u8>,
    pub len: usize,
    pub parsed: X509Certificate<'static>,
}

impl Cert {
    /// Construct a Cert from DER bytes and actual length.
    pub fn from_der(der: &[u8], len: usize) -> Result<Self, &'static str> {
        let owned = der[..len].to_vec();
        // Lifetime hack: leak the owned data so X509Certificate can live 'static
        let leaked = Box::leak(owned.clone().into_boxed_slice());
        let (_, parsed) = X509Certificate::from_der(leaked).map_err(|_| "Failed to parse DER")?;
        Ok(Cert { der: owned, len, parsed })
    }

    /// Returns PEM-encoded certificate as a String using the pem crate.
    pub fn to_pem(&self) -> String {
        let pem = Pem::new(
            "CERTIFICATE",
            self.der[..self.len].to_vec()
        );
        pem::encode(&pem)
    }

    /// Returns hex representation as a String.
    pub fn to_hex(&self) -> String {
        hex::encode(&self.der[..self.len])
    }

    /// Returns the serial number as a lowercase hex string.
    pub fn serial_hex(&self) -> String {
        self.parsed.serial.to_str_radix(16)
    }

    /// Returns the subject as a String.
    pub fn subject(&self) -> String {
        format!("{}", self.parsed.subject())
    }
}

/// Convenience implement Display for PEM view
impl fmt::Display for Cert {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_pem())
    }
}

// Example usage (uncomment for your main.rs)
/*
fn main() -> std::io::Result<()> {
    // Suppose you have DER bytes and their actual length from SPI:
    let der: Vec<u8> = /* ... */;
    let len: usize = /* ... */;

    let cert = Cert::from_der(&der, len).expect("DER parse failed");
    println!("Serial: {}", cert.serial_hex());
    println!("Subject: {}", cert.subject());
    println!("PEM:\n{}", cert.to_pem());
    println!("Hex:\n{}", cert.to_hex());

    // Write to files
    std::fs::write(format!("cert_sn_{}.pem", cert.serial_hex()), cert.to_pem())?;
    std::fs::write(format!("cert_sn_{}.hex", cert.serial_hex()), cert.to_hex())?;
    std::fs::write(format!("cert_sn_{}.der", cert.serial_hex()), &cert.der)?;

    Ok(())
}
*/
