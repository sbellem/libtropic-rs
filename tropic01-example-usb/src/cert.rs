use hex;
use pem::Pem;
use std::fmt;
use std::io;
use utils::x509::*;
use x509_parser::prelude::{FromDer, X509Certificate};
use x509_parser::validate::{
    Validator, VecLogger, X509CertificateValidator, X509StructureValidator,
};
use x509_parser::x509::SubjectPublicKeyInfo;

pub const NUM_CERTIFICATES: usize = 4;

const VALIDATE_ERRORS_FATAL: bool = false;

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
        Ok(Cert {
            der: owned,
            len,
            parsed,
        })
    }

    /// Returns PEM-encoded certificate as a String using the pem crate.
    pub fn to_pem(&self) -> String {
        let pem = Pem::new("CERTIFICATE", self.der[..self.len].to_vec());
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

    /// Returns the public key of the subject
    pub fn public_key(&self) -> SubjectPublicKeyInfo<'_> {
        self.parsed.public_key().clone()
    }

    /// Show certificate minimal information
    ///
    /// credits:
    ///     repo: https://github.com/rusticata/x509-parser
    ///     commit: b7dcc9397b596cf9fa3df65115c3f405f1748b2a
    ///     file: examples/print-cert.rs
    pub fn print_min_info(&self) -> io::Result<()> {
        let x509 = self.parsed.clone();

        let version = x509.version();
        if version.0 < 3 {
            println!("  Version: {version}");
        } else {
            println!("  Version: INVALID({})", version.0);
        }
        println!("  Serial: {}", x509.tbs_certificate.raw_serial_as_string());
        println!("  Subject: {}", x509.subject());
        println!("  Issuer: {}", x509.issuer());
        Ok(())
    }

    /// Show certificate information
    ///
    /// credits:
    ///     repo: https://github.com/rusticata/x509-parser
    ///     commit: b7dcc9397b596cf9fa3df65115c3f405f1748b2a
    ///     file: examples/print-cert.rs
    pub fn print_basic_info(&self) -> io::Result<()> {
        let x509 = self.parsed.clone();

        let version = x509.version();
        if version.0 < 3 {
            println!("  Version: {version}");
        } else {
            println!("  Version: INVALID({})", version.0);
        }
        println!("  Serial: {}", x509.tbs_certificate.raw_serial_as_string());
        println!("  Subject: {}", x509.subject());
        println!("  Issuer: {}", x509.issuer());
        println!("  Validity:");
        println!("    NotBefore: {}", x509.validity().not_before);
        println!("    NotAfter:  {}", x509.validity().not_after);
        println!("    is_valid:  {}", x509.validity().is_valid());
        println!("  Subject Public Key Info:");
        print_x509_ski(x509.public_key());
        print_x509_signature_algorithm(&x509.signature_algorithm, 4);
        println!("  Signature Value:");
        for l in format_number_to_hex_with_colon(x509.signature_value.as_raw_slice(), 16) {
            println!("      {l}");
        }

        Ok(())
    }

    pub fn print_extension_info(&self) -> io::Result<()> {
        let x509 = self.parsed.clone();
        println!("  Extensions:");
        for ext in x509.extensions() {
            print_x509_extension(&ext.oid, ext);
        }
        Ok(())
    }

    pub fn print_validation_info(&self) -> io::Result<()> {
        let x509 = self.parsed.clone();
        print!("Structure validation status: ");
        //#[cfg(feature = "validate")]
        let mut logger = VecLogger::default();
        // structure validation status
        let ok = X509StructureValidator
            .chain(X509CertificateValidator)
            .validate(&x509, &mut logger);
        if ok {
            println!("Ok");
        } else {
            println!("FAIL");
        }
        for warning in logger.warnings() {
            println!("  [W] {warning}");
        }
        for error in logger.errors() {
            println!("  [E] {error}");
        }
        println!();
        if VALIDATE_ERRORS_FATAL && !logger.errors().is_empty() {
            return Err(io::Error::new(io::ErrorKind::Other, "validation failed"));
        }

        Ok(())
    }

    pub fn print_verification_info(&self, issuer_cert: &X509Certificate<'_>) -> io::Result<()> {
        let x509 = self.parsed.clone();
        print!("Signature verification: ");
        match x509.verify_signature(Some(&issuer_cert.tbs_certificate.subject_pki)) {
            Ok(_) => {
                println!("OK");
                //println!("Signature verification succeeded");
            },
            Err(e) => {
                println!("FAIL");
                println!("  [E] {:?}", e);
                //println!("Signature verification failed");
            },
        }

        Ok(())
    }
}

/// Convenience implement Display for PEM view
impl fmt::Display for Cert {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_pem())
    }
}
