use std::env;
use tropic01::Tropic01;
use tropic01::BankId;
use tropic01_example_usb::port::UsbDevice;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    env_logger::init();
    
    println!("=============================================================");
    println!("==== TROPIC01 show chip ID and firmware versions example ====");
    println!("=============================================================");

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

    // TODO: rng seed ?
    let usb_device = UsbDevice::new(&port_name, baud_rate)?;
    let mut tropic = Tropic01::new(usb_device);

    println!("Rebooting into APPLICATION mode to check FW versions");
    // TODO: handle error
    tropic.startup_req(tropic01::StartupReq::Reboot)?;

    println!("Reading RISC-V FW version");
    let riscv_fw_ver = tropic
        .get_info_riscv_fw_ver()
        .map_err(|e| {
            eprintln!("Failed to get RISC-V FW version: {:?}", e);
            anyhow::anyhow!("Failed to get RISC-V FW version: {:?}", e)
        })?;
    
    println!(
        "Chip is executing RISC-V application FW version: {:02X}.{:02X}.{:02X}    (+ .{:02X})",
        riscv_fw_ver[3], riscv_fw_ver[2], riscv_fw_ver[1], riscv_fw_ver[0]
    );
    
    println!("Reading SPECT FW version");
    let spect_fw_ver = tropic
        .get_info_spect_fw_ver()
        .map_err(|e| {
            eprintln!("Failed to get SPECT FW version: {:?}", e);
            anyhow::anyhow!("Failed to get SPECT FW version: {:?}", e)
        })?;
    
    println!(
        "Chip is executing SPECT application FW version: {:02X}.{:02X}.{:02X}    (+ .{:02X})",
        spect_fw_ver[3], spect_fw_ver[2], spect_fw_ver[1], spect_fw_ver[0]
    );

    println!("    -------------------------------------------------------------------------------------------------------------");
    println!("Rebooting into MAINTENANCE mode to check bootloader version and fw bank headers");
    // TODO: handle error
    tropic.startup_req(tropic01::StartupReq::MaintenanceReboot)?;
    //LT_LOG_ERROR("Failed to reboot into MAINTENANCE mode, ret=%s", lt_ret_verbose(ret));
        
    println!("Reading RISC-V FW version (during maintenance chip actually returns bootloader version):");
    let riscv_fw_ver = tropic
        .get_info_riscv_fw_ver()
        .map_err(|e| {
            eprintln!("Failed to get RISC-V FW version: {:?}", e);
            anyhow::anyhow!("Failed to get RISC-V FW version: {:?}", e)
        })?;

    println!("Bootloader version: {:02X}.{:02X}.{:02X}    (+ .{:02X})",
        riscv_fw_ver[3] & 0x7f,
        riscv_fw_ver[2],
        riscv_fw_ver[1],
        riscv_fw_ver[0],
    );
    println!("    -------------------------------------------------------------------------------------------------------------");

    println!("Reading and printing headers of all 4 FW banks:");

    println!("Bank Id Firmware 1: {:?}", BankId::RiscvFw1);
    println!("    Reading header from Application's firmware bank 1:\r\n");

    //ret = lt_print_fw_header(h, FirmwareBankId, printf);
    let riscv_fw_bank_info = tropic
        .get_info_fw_bank(BankId::RiscvFw1 as u8)
        .map_err(|e| {
            eprintln!("Failed to get RISC-V FW bank 1 info: {:?}", e);
            anyhow::anyhow!("Failed to get RISC-V FW bank 1 info: {:?}", e)
        })?;

    println!("riscv_fw_bank_info: {:?}", riscv_fw_bank_info);
    Ok(())
}

