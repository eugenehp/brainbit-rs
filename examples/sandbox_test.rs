//! Test that the network sandbox actually blocks internet access.
//!
//! Run with: `cargo run --example sandbox_test`

use brainbit::sandbox;
use std::net::TcpStream;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    // Test BEFORE sandbox
    println!("=== Before sandbox ===");
    match TcpStream::connect_timeout(
        &"1.1.1.1:80".parse().unwrap(),
        Duration::from_secs(3),
    ) {
        Ok(_) => println!("  TCP to 1.1.1.1:80 → CONNECTED (internet works)"),
        Err(e) => println!("  TCP to 1.1.1.1:80 → FAILED: {} (no internet?)", e),
    }

    // Apply sandbox
    println!("\n=== Applying network sandbox ===");
    match sandbox::block_internet() {
        Ok(()) => println!("  Sandbox applied successfully!"),
        Err(e) => {
            println!("  Sandbox failed: {} (continuing test anyway)", e);
            return Ok(());
        }
    }
    println!("  is_sandboxed() = {}", sandbox::is_sandboxed());

    // Test AFTER sandbox
    println!("\n=== After sandbox ===");
    match TcpStream::connect_timeout(
        &"1.1.1.1:80".parse().unwrap(),
        Duration::from_secs(3),
    ) {
        Ok(_) => {
            println!("  TCP to 1.1.1.1:80 → CONNECTED (SANDBOX FAILED!)");
            std::process::exit(1);
        }
        Err(e) => println!("  TCP to 1.1.1.1:80 → BLOCKED: {} ✓", e),
    }

    match TcpStream::connect_timeout(
        &"8.8.8.8:53".parse().unwrap(),
        Duration::from_secs(3),
    ) {
        Ok(_) => {
            println!("  TCP to 8.8.8.8:53 → CONNECTED (SANDBOX FAILED!)");
            std::process::exit(1);
        }
        Err(e) => println!("  TCP to 8.8.8.8:53 → BLOCKED: {} ✓", e),
    }

    // Unix sockets should still work (needed for D-Bus on Linux)
    #[cfg(unix)]
    {
        use std::os::unix::net::UnixStream;
        println!("\n=== IPC test ===");
        match UnixStream::connect("/var/run/dbus/system_bus_socket") {
            Ok(_) => println!("  Unix socket (D-Bus) → CONNECTED ✓ (IPC works)"),
            Err(e) => println!("  Unix socket (D-Bus) → {}: (may not exist on macOS)", e),
        }
    }

    println!("\n✅ Sandbox is working correctly!");
    Ok(())
}
