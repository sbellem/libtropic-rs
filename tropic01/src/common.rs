//! Pure-Rust representation and no-std-friendly tests for lt_header_boot_v2_t.
//!
//! This file intentionally does NOT contain `#![no_std]` so it can be included from a
//! crate that already applies that at the crate root (put `#![no_std]` in `lib.rs`).
//!
//! The tests below avoid `format!` / `std` and instead write `Display` output into a
//! small stack-backed buffer that implements `core::fmt::Write`, so the tests work
//! even when the crate is `no_std`.

use core::convert::TryFrom;
use core::fmt::{self, Write};

// @brief Maximal size of returned fw header
/// Expected size of the v1 header in bytes.
pub const L2_GET_INFO_FW_HEADER_SIZE_BOOT_V1: usize = 20;
/// Expected size of the v2 header in bytes.
pub const L2_GET_INFO_FW_HEADER_SIZE_BOOT_V2: usize = 52;
pub const L2_GET_INFO_FW_HEADER_SIZE_BOOT_V2_EMPTY_BANK: usize = 0;

// /// Maximal size of returned fw header */
//pub const L2_GET_INFO_FW_HEADER_SIZE TR01_L2_GET_INFO_FW_HEADER_SIZE_BOOT_V2

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LtHeaderBootV2 {
    /// 1 == RISCV FW, 2 == SPECT FW
    pub type_: u16,
    pub padding: u8,
    /// header version
    pub header_version: u8,
    /// fw version
    pub ver: u32,
    /// fw size in bytes
    pub size: u32,
    /// git hash (first 4 bytes stored as u32 in header)
    pub git_hash: u32,
    /// SHA256 hash (32 bytes)
    pub hash: [u8; 32],
    /// pair version
    pub pair_version: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseHeaderV2Error {
    TooShort { got: usize, expected: usize },
}

impl core::fmt::Display for ParseHeaderV2Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseHeaderV2Error::TooShort { got, expected } => {
                write!(f, "slice too short: got {} bytes, expected {}", got, expected)
            }
        }
    }
}

impl TryFrom<&[u8]> for LtHeaderBootV2 {
    type Error = ParseHeaderV2Error;

    /// Parse a byte slice into LtHeaderBootV2.
    /// Expects at least L2_GET_INFO_FW_HEADER_SIZE_BOOT_V2 bytes.
    fn try_from(slice: &[u8]) -> Result<Self, Self::Error> {
        if slice.len() < L2_GET_INFO_FW_HEADER_SIZE_BOOT_V2 {
            return Err(ParseHeaderV2Error::TooShort {
                got: slice.len(),
                expected: L2_GET_INFO_FW_HEADER_SIZE_BOOT_V2,
            });
        }

        // Parse as little-endian (adjust to from_be_bytes if needed).
        let type_ = u16::from_le_bytes([slice[0], slice[1]]);
        let padding = slice[2];
        let header_version = slice[3];
        let ver = u32::from_le_bytes([slice[4], slice[5], slice[6], slice[7]]);
        let size = u32::from_le_bytes([slice[8], slice[9], slice[10], slice[11]]);
        let git_hash = u32::from_le_bytes([slice[12], slice[13], slice[14], slice[15]]);

        let mut hash = [0u8; 32];
        hash.copy_from_slice(&slice[16..48]);

        let pair_version = u32::from_le_bytes([slice[48], slice[49], slice[50], slice[51]]);

        Ok(LtHeaderBootV2 {
            type_,
            padding,
            header_version,
            ver,
            size,
            git_hash,
            hash,
            pair_version,
        })
    }
}

impl fmt::Display for LtHeaderBootV2 {
    /// Format matching the original C printing (uppercase hex, zero-padded widths).
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "      Type:               {:04X}", self.type_)?;
        writeln!(f, "      Padding:            {:02X}", self.padding)?;
        writeln!(f, "      FW header version:  {:02X}", self.header_version)?;
        writeln!(f, "      Version:            {:08X}", self.ver)?;
        writeln!(f, "      Size:               {:08X}", self.size)?;
        writeln!(f, "      Git hash:           {:08X}", self.git_hash)?;

        write!(f, "      Hash:          ")?;
        for b in &self.hash {
            write!(f, "{:02X}", b)?;
        }
        writeln!(f)?;
        writeln!(f, "      Pair version:  {:08X}", self.pair_version)?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use core::fmt::Write;
    use core::str;

    // Small stack-backed buffer for tests that implements core::fmt::Write.
    // Avoids using `format!` / std allocation so tests are compatible with no_std crates.
    struct TestBuf {
        buf: [u8; 512], // increased from 256 to 512 to avoid overflow for the v2 header output
        pos: usize,
    }

    impl TestBuf {
        fn new() -> Self {
            Self { buf: [0u8; 512], pos: 0 }
        }

        fn as_str(&self) -> &str {
            // For tests we expect ASCII hex output; panic on invalid UTF-8 which indicates a bug.
            core::str::from_utf8(&self.buf[..self.pos]).expect("buffer contains valid utf8")
        }
    }

    impl core::fmt::Write for TestBuf {
        fn write_str(&mut self, s: &str) -> fmt::Result {
            let bytes = s.as_bytes();
            if bytes.len() > self.buf.len() - self.pos {
                return Err(fmt::Error);
            }
            let end = self.pos + bytes.len();
            self.buf[self.pos..end].copy_from_slice(bytes);
            self.pos = end;
            Ok(())
        }
    }

    #[test]
    fn test_header_boot_v2_parse_and_display() {
        // little-endian sample bytes constructed to test parsing
        let mut buf = [0u8; L2_GET_INFO_FW_HEADER_SIZE_BOOT_V2];
        // type = 0x0201 (bytes 0..2 little-endian)
        buf[0] = 0x01;
        buf[1] = 0x02;
        // padding = 0xAA
        buf[2] = 0xAA;
        // header_version = 0x01
        buf[3] = 0x01;
        // ver = 0x11223344 (little-endian)
        buf[4..8].copy_from_slice(&0x11223344u32.to_le_bytes());
        // size = 0x55667788
        buf[8..12].copy_from_slice(&0x55667788u32.to_le_bytes());
        // git_hash = 0x99AABBCC
        buf[12..16].copy_from_slice(&0x99AABBCCu32.to_le_bytes());
        // hash: 0..31
        for i in 0..32 {
            buf[16 + i] = i as u8;
        }
        // pair_version = 0xDEADBEEF
        buf[48..52].copy_from_slice(&0xDEADBEEFu32.to_le_bytes());

        let header = LtHeaderBootV2::try_from(&buf[..]).expect("parse v2");
        assert_eq!(header.type_, 0x0201);
        assert_eq!(header.padding, 0xAA);
        assert_eq!(header.header_version, 0x01);
        assert_eq!(header.ver, 0x11223344);
        assert_eq!(header.size, 0x55667788);
        assert_eq!(header.git_hash, 0x99AABBCC);
        assert_eq!(header.hash[0], 0);
        assert_eq!(header.hash[31], 31);
        assert_eq!(header.pair_version, 0xDEADBEEF);

        // Write Display output to stack-backed TestBuf instead of using format!
        let mut tb = TestBuf::new();
        write!(&mut tb, "{}", header).expect("write should succeed");
        let s = tb.as_str();

        assert!(s.contains("Type:               0201"));
        assert!(s.contains("Padding:            AA"));
        assert!(s.contains("FW header version:  01"));
        assert!(s.contains("Version:            11223344"));
        assert!(s.contains("Size:               55667788"));
        assert!(s.contains("Git hash:           99AABBCC"));
        assert!(s.contains("Pair version:  DEADBEEF"));
        // hash begins with 000102...
        assert!(s.contains("Hash:          00010203"));
    }
}
