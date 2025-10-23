use hal::SerialTransport;
use tropic01::Tropic01;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    env_logger::init();
    
    let args: Vec<String> = env::args().collect();
    let port_name = args.get(1).cloned().unwrap_or_else(|| "/dev/ttyACM0".to_string());

    let baud_rate = args
        .get(2)
        .and_then(|s| s.parse().ok())
        .filter(|&r| [4800, 9600, 19200, 38400, 115200].contains(&r))
        .unwrap_or(115200);

    println!("Opening TS1302 dongle on {} @ {} baud", port_name, baud_rate);

    let transport = SerialTransport::new(&port_name, baud_rate)?;
    let mut tropic01 = Tropic01::new(transport);



    let res = tropic01.get_info_chip_id()?;
    println!("ChipId: {res:x?}");
    let chip_id = res.to_vec();

    Ok(())
}
