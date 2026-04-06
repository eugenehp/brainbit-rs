//! Example: measure electrode resistance (impedance check).
//!
//! Run with: `cargo run --example resist`

use brainbit::prelude::*;
use std::sync::{Arc, Mutex};
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let scanner = Scanner::new(&[SensorFamily::LEBrainBit])?;
    scanner.start()?;
    std::thread::sleep(Duration::from_secs(5));
    scanner.stop()?;

    let devices = scanner.devices()?;
    if devices.is_empty() {
        eprintln!("No device found.");
        return Ok(());
    }

    let mut device = BrainBitDevice::connect(&scanner, &devices[0])?;
    println!("Connected to: {}", device.name()?);

    let last = Arc::new(Mutex::new(None::<ResistanceSample>));
    let last2 = last.clone();

    device.on_resist(move |sample| {
        *last2.lock().unwrap() = Some(sample);
    })?;

    device.start_resist()?;
    println!("Measuring resistance for 5 seconds...\n");

    for _ in 0..10 {
        std::thread::sleep(Duration::from_millis(500));
        if let Some(r) = last.lock().unwrap().as_ref() {
            println!(
                "O1={:.0}Ω  O2={:.0}Ω  T3={:.0}Ω  T4={:.0}Ω",
                r.o1, r.o2, r.t3, r.t4,
            );
        }
    }

    device.stop_resist()?;
    device.remove_resist_callback();
    device.disconnect()?;

    println!("\nDone.");
    Ok(())
}
