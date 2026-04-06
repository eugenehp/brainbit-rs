//! CLI tool: scan for BrainBit devices, connect, and stream EEG data.

use brainbit::prelude::*;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    println!("BrainBit EEG — Rust CLI");
    println!("=======================\n");

    // Scan for all BrainBit-family devices
    let families = [
        SensorFamily::LEBrainBit,
        SensorFamily::LEBrainBit2,
        SensorFamily::LEBrainBitPro,
        SensorFamily::LEBrainBitFlex,
    ];

    println!("Scanning for BrainBit devices (5 seconds)...");
    let scanner = Scanner::new(&families)?;
    scanner.start()?;
    std::thread::sleep(Duration::from_secs(5));
    scanner.stop()?;

    let devices = scanner.devices()?;
    if devices.is_empty() {
        eprintln!("No BrainBit device found. Make sure the device is powered on.");
        return Ok(());
    }

    println!("\nFound {} device(s):", devices.len());
    for (i, d) in devices.iter().enumerate() {
        println!(
            "  [{}] {} (family={:?}, model={}, addr={}, rssi={})",
            i,
            d.name_str(),
            d.sens_family,
            d.sens_model,
            d.address_str(),
            d.rssi,
        );
    }

    // Connect to the first device
    println!("\nConnecting to {}...", devices[0].name_str());
    let mut device = BrainBitDevice::connect(&scanner, &devices[0])?;

    println!("  Name:      {}", device.name()?);
    println!("  Address:   {}", device.address()?);
    println!("  Serial:    {}", device.serial_number()?);
    println!("  Family:    {:?}", device.family());
    println!("  Battery:   {}%", device.battery_level()?);
    println!("  Gain:      {:?}", device.gain()?);
    println!("  Frequency: {:?}", device.sampling_frequency()?);

    let version = device.firmware_version()?;
    println!(
        "  Firmware:  {}.{}.{}",
        version.fw_major, version.fw_minor, version.fw_patch
    );
    println!(
        "  Hardware:  {}.{}.{}",
        version.hw_major, version.hw_minor, version.hw_patch
    );

    if device.is_flex_v1()? {
        println!("  Device type: Flex (v1)");
    } else {
        println!("  Device type: BrainBit (original)");
    }

    // List features
    let features = device.features()?;
    println!("\n  Features: {:?}", features);

    // Stream 4 seconds of EEG
    let n = BRAINBIT_SAMPLING_RATE as usize * 4;
    println!("\nCapturing {} samples (~4 seconds)...", n);
    let samples = device.capture_signal(n)?;

    println!("\nFirst 10 samples:");
    println!("{:>8} {:>4} {:>14} {:>14} {:>14} {:>14}", "Pack#", "Mkr", "O1 (V)", "O2 (V)", "T3 (V)", "T4 (V)");
    for s in samples.iter().take(10) {
        println!(
            "{:>8} {:>4} {:>14.9} {:>14.9} {:>14.9} {:>14.9}",
            s.pack_num, s.marker,
            s.channels[0], s.channels[1], s.channels[2], s.channels[3],
        );
    }

    println!("\nLast 10 samples:");
    for s in samples.iter().rev().take(10).collect::<Vec<_>>().into_iter().rev() {
        println!(
            "{:>8} {:>4} {:>14.9} {:>14.9} {:>14.9} {:>14.9}",
            s.pack_num, s.marker,
            s.channels[0], s.channels[1], s.channels[2], s.channels[3],
        );
    }

    println!("\nDone. Disconnecting...");
    device.disconnect()?;
    Ok(())
}
