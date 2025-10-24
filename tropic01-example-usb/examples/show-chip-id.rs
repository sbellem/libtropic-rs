use std::env;
use tropic01::Tropic01;
use tropic01_example_usb::chipid::ChipId;
use tropic01_example_usb::port::UsbDevice;

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
    let res = tropic.get_info_chip_id()?;
    let chip_id = res.to_vec();
    let chip_info = ChipId::try_from(&chip_id[..]).map_err(|e| {
        println!("Failed to parse chip ID: {}", e);
        anyhow::anyhow!("Chip ID parsing error: {}", e)
    })?;

    chip_info.print_details();

    Ok(())
}
