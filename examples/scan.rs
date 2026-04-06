//! Example: scan for BrainBit devices and print their info.
//!
//! Run with: `cargo run --example scan`

use brainbit::prelude::*;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let families = [
        SensorFamily::LEBrainBit,
        SensorFamily::LEBrainBit2,
        SensorFamily::LEBrainBitPro,
        SensorFamily::LEBrainBitFlex,
    ];

    println!("Scanning for BrainBit devices...");
    let scanner = Scanner::new(&families)?;
    scanner.start()?;
    std::thread::sleep(Duration::from_secs(5));
    scanner.stop()?;

    let devices = scanner.devices()?;
    println!("Found {} device(s):\n", devices.len());

    for d in &devices {
        println!("  Name:     {}", d.name_str());
        println!("  Family:   {:?}", d.sens_family);
        println!("  Model:    {}", d.sens_model);
        println!("  Address:  {}", d.address_str());
        println!("  Serial:   {}", d.serial_number_str());
        println!("  RSSI:     {} dBm", d.rssi);
        println!("  Pairing:  {}", if d.pairing_required != 0 { "yes" } else { "no" });
        println!();
    }

    Ok(())
}
