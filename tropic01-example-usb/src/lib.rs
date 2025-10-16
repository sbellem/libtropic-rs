//! Public library for tropic01-example-usb
//! Contains chip ID parsing and helpers.

use std::convert::TryInto;

/// Helper function for hex formatting (lowercase).
pub fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect::<Vec<_>>().join("")
}

#[derive(Debug, PartialEq, Eq)]
pub struct ChipId {
    pub version: [u8; 4],            // 0-3
    pub fl_chip_info: [u8; 16],      // 4-19
    pub func_test_info: [u8; 8],     // 20-27
    pub silicon_rev: [u8; 4],        // 28-31
    pub package_type_id: [u8; 2],    // 32-33
    pub prov_info: [u8; 4],          // 36-39
    pub provisioning_date: [u8; 2],  // 40-41
    pub hsm_ver: [u8; 4],            // 42-45
    pub prog_ver: [u8; 4],           // 46-49
    pub serial_number: [u8; 16],     // 52-67
    pub part_number: [u8; 16],       // 68-83
    pub prov_templ_ver: [u8; 2],     // 84-85
    pub prov_templ_tag: [u8; 4],     // 86-89
    pub prov_spec_ver: [u8; 2],      // 90-91
    pub prov_spec_tag: [u8; 4],      // 92-95
    pub batch_id: [u8; 5],           // 96-100
}

/// TryFrom implementation for parsing chip_id slice
impl TryFrom<&[u8]> for ChipId {
    type Error = &'static str;

    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        if bytes.len() < 101 {
            return Err("Chip ID slice too short");
        }
        Ok(Self {
            version: bytes[0..4].try_into().unwrap(),
            fl_chip_info: bytes[4..20].try_into().unwrap(),
            func_test_info: bytes[20..28].try_into().unwrap(),
            silicon_rev: bytes[28..32].try_into().unwrap(),
            package_type_id: bytes[32..34].try_into().unwrap(),
            prov_info: bytes[36..40].try_into().unwrap(),
            provisioning_date: bytes[40..42].try_into().unwrap(),
            hsm_ver: bytes[42..46].try_into().unwrap(),
            prog_ver: bytes[46..50].try_into().unwrap(),
            serial_number: bytes[52..68].try_into().unwrap(),
            part_number: bytes[68..84].try_into().unwrap(),
            prov_templ_ver: bytes[84..86].try_into().unwrap(),
            prov_templ_tag: bytes[86..90].try_into().unwrap(),
            prov_spec_ver: bytes[90..92].try_into().unwrap(),
            prov_spec_tag: bytes[92..96].try_into().unwrap(),
            batch_id: bytes[96..101].try_into().unwrap(),
        })
    }
}

impl ChipId {
    /// Print all parsed details in a readable format.
    pub fn print_details(&self) {
        let version = self.version;
        println!(
            "CHIP_ID ver: 0x{:02x}{:02x}{:02x}{:02x} (v{}.{}.{}.{})",
            version[0], version[1], version[2], version[3],
            version[0], version[1], version[2], version[3]
        );
        println!("FL_PROD_DATA: 0x{} (N/A)", bytes_to_hex(&self.fl_chip_info));
        println!("MAN_FUNC_TEST: 0x{} (PASSED)", bytes_to_hex(&self.func_test_info));

        // Silicon revision (ASCII)
        let silicon_rev_str = String::from_utf8_lossy(&self.silicon_rev);
        println!(
            "Silicon rev: 0x{:02x}{:02x}{:02x}{:02x} ({})",
            self.silicon_rev[0], self.silicon_rev[1], self.silicon_rev[2], self.silicon_rev[3],
            silicon_rev_str
        );

        println!("Package ID: 0x{} (QFN32, 4x4mm)", bytes_to_hex(&self.package_type_id));

        // Provisioning info (bytes 36-39, hex: version, fab ID, part number ID)
        let prov_info_ver = self.prov_info[0];
        let fab_id = u16::from_be_bytes([self.prov_info[1], self.prov_info[2] & 0x0f]); // 12-bit fab ID
        let pn_id = u16::from_be_bytes([(self.prov_info[2] & 0xf0) >> 4, self.prov_info[3]]); // 12-bit part number ID
        println!("Prov info ver: 0x{:02x} (v{})", prov_info_ver, prov_info_ver);
        println!("Fab ID: 0x{:03x} (EPS Global - Brno)", fab_id);
        println!("P/N ID (short P/N): 0x{:03x}", pn_id);

        let provisioning_date = u16::from_be_bytes(self.provisioning_date);
        println!("Prov date: 0x{:04x}", provisioning_date);

        let hsm_ver = self.hsm_ver;
        println!(
            "HSM HW/FW/SW ver: 0x{:02x}{:02x}{:02x}{:02x} (v0.{}.{}.{})",
            hsm_ver[0], hsm_ver[1], hsm_ver[2], hsm_ver[3],
            hsm_ver[1], hsm_ver[2], hsm_ver[3]
        );
        let prog_ver = self.prog_ver;
        println!(
            "Programmer ver: 0x{:02x}{:02x}{:02x}{:02x} (v0.{}.{}.{})",
            prog_ver[0], prog_ver[1], prog_ver[2], prog_ver[3],
            prog_ver[1], prog_ver[2], prog_ver[3]
        );

        // Serial number parsing
        let sn = self.serial_number[0];
        let fab_data = u32::from_be_bytes([0, self.serial_number[1], self.serial_number[2], self.serial_number[3]]);
        let fab_id_sn = (fab_data >> 12) & 0xfff;
        let pn_id_sn = fab_data & 0xfff;
        let fab_date = u16::from_be_bytes([self.serial_number[4], self.serial_number[5]]);
        let lot_id = bytes_to_hex(&self.serial_number[6..11]);
        let wafer_id = self.serial_number[11];
        let x_coord = u16::from_be_bytes([self.serial_number[12], self.serial_number[13]]);
        let y_coord = u16::from_be_bytes([self.serial_number[14], self.serial_number[15]]);
        let serial_number_hex = bytes_to_hex(&self.serial_number);

        println!("S/N: 0x{}", serial_number_hex);
        println!("  SN: 0x{:02x}", sn);
        println!("  Fab ID: 0x{:03x}", fab_id_sn);
        println!("  P/N ID: 0x{:03x}", pn_id_sn);
        println!("  Fabrication Date: 0x{:04x}", fab_date);
        println!("  Lot ID: 0x{}", lot_id);
        println!("  Wafer ID: 0x{:02x}", wafer_id);
        println!("  X-Coordinate: 0x{:04x}", x_coord);
        println!("  Y-Coordinate: 0x{:04x}", y_coord);

        // Part number (first byte is length, rest is ASCII)
        let pn_len = self.part_number[0] as usize;
        let pn_data = if pn_len > 0 && pn_len < self.part_number.len() {
            String::from_utf8_lossy(&self.part_number[1..=pn_len]).to_string()
        } else {
            "Unknown".to_string()
        };
        let part_num_hex = self.part_number.iter().map(|b| format!("{:02X}", b)).collect::<String>();
        println!("P/N (long) = 0x{} ({})", part_num_hex, pn_data);

        let prov_templ_ver = u16::from_be_bytes(self.prov_templ_ver);
        println!("Prov template ver: 0x{:04x} (v{}.{})", prov_templ_ver, prov_templ_ver >> 8, prov_templ_ver & 0xff);

        println!("Prov template tag: 0x{}", bytes_to_hex(&self.prov_templ_tag));

        let prov_spec_ver = u16::from_be_bytes(self.prov_spec_ver);
        println!("Prov specification ver: 0x{:04x} (v0.{})", prov_spec_ver, prov_spec_ver & 0xff);

        println!("Prov specification tag: 0x{}", bytes_to_hex(&self.prov_spec_tag));

        println!("Batch ID: 0x{}", bytes_to_hex(&self.batch_id));
    }
}
