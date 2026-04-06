//! Example: connect to a BrainBit and stream EEG signal data with a callback.
//!
//! Run with: `cargo run --example stream`

use brainbit::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    // Scan
    let scanner = Scanner::new(&[SensorFamily::LEBrainBit])?;
    scanner.start()?;
    std::thread::sleep(Duration::from_secs(5));
    scanner.stop()?;

    let devices = scanner.devices()?;
    if devices.is_empty() {
        eprintln!("No device found.");
        return Ok(());
    }

    // Connect
    let mut device = BrainBitDevice::connect(&scanner, &devices[0])?;
    println!("Connected to: {}", device.name()?);

    // Stream with callback
    let count = Arc::new(AtomicUsize::new(0));
    let count2 = count.clone();

    device.on_signal(move |samples| {
        for s in samples {
            let n = count2.fetch_add(1, Ordering::Relaxed);
            if n % 250 == 0 {
                println!(
                    "[{:>6}] O1={:>10.6}µV  O2={:>10.6}µV  T3={:>10.6}µV  T4={:>10.6}µV",
                    s.pack_num,
                    s.channels[0] * 1e6,
                    s.channels[1] * 1e6,
                    s.channels[2] * 1e6,
                    s.channels[3] * 1e6,
                );
            }
        }
    })?;

    device.start_signal()?;
    println!("Streaming for 10 seconds (printing every 250th sample)...\n");
    std::thread::sleep(Duration::from_secs(10));
    device.stop_signal()?;
    device.remove_signal_callback();

    let total = count.load(Ordering::Relaxed);
    println!("\nReceived {} samples in 10 seconds ({:.1} Hz)", total, total as f64 / 10.0);

    device.disconnect()?;
    Ok(())
}
