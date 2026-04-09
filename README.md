# brainbit

A Rust library and terminal UI for streaming real-time EEG data from
**BrainBit** headband devices via the [NeuroSDK2](https://sdk.brainbit.com/) C library.

Rust FFI wrapper with 100% API parity — all 175 functions from `sdk_api.h`
bound, all types from `cmn_type.h` mapped to `#[repr(C)]` Rust equivalents,
SHA-256 integrity verification, and OS-level network sandboxing.

## Installation

```shell
cargo add brainbit
```

---

## Supported hardware

| Device | BLE Name | Scanner Family | Channels | Hz | How to distinguish |
|---|---|---|---|---|---|
| BrainBit (original) | `BrainBit` | `LEBrainBit` | 4 (O1, O2, T3, T4) | 250 | `FwMajor < 100` |
| Flex (v1) | `BrainBit` | `LEBrainBit` | 4 (relocatable) | 250 | `FwMajor >= 100` |
| BrainBit 2 | `BrainBit` | `LEBrainBit2` | up to 8 | 250 | — |
| BrainBit Pro | `BB Pro` | `LEBrainBitPro` | up to 8 | 250 | — |
| Flex 4 | `Flex` | `LEBrainBitFlex` | 4 | 250 | `SensModel == 2` |
| Flex 8 | `Flex Pro` | `LEBrainBitFlex` | 8 | 250 | `SensModel == 3` |
| Callibri | — | `LECallibri` | 1 (EEG/EMG/ECG/EDA) | configurable | — |
| Headphones | — | `LEHeadPhones` | 7 | 250 | — |
| Headphones 2 | — | `LEHeadPhones2` | 4 | 250 | — |
| Headband | — | `LEHeadband` | 4 | 250 | — |
| NeuroEEG | — | `LENeuroEEG` | up to 24 | configurable | — |

All devices communicate over BLE. The NeuroSDK2 library handles the
full Bluetooth stack internally — no external BLE library needed.

---

## Cross-platform

Works on **Windows**, **Linux**, and **macOS**. The `neurosdk2` shared library
(`neurosdk2.dll` / `libneurosdk2.so` / `libneurosdk2.dylib`) is loaded at
runtime via `libloading` — no build-time C dependencies, no bindgen, no
system headers.

---

## Prerequisites

| Requirement | Notes |
|---|---|
| Rust ≥ 1.75 | `rustup update stable` |
| NeuroSDK2 native library | Run `./sdk/download.sh` (auto-downloads + verifies) |
| BLE adapter | Built-in or USB Bluetooth adapter |

### Native library sources

| Platform | Repository | File |
|---|---|---|
| **Windows** | [neurosdk2-cpp](https://github.com/BrainbitLLC/neurosdk2-cpp) | `neurosdk2-x64.dll` / `neurosdk2-x32.dll` |
| **Linux** | [linux_neurosdk2](https://github.com/BrainbitLLC/linux_neurosdk2) | `libneurosdk2.so` |
| **macOS** | [apple_neurosdk2](https://github.com/BrainbitLLC/apple_neurosdk2) | `libneurosdk2.dylib` (universal x86_64 + arm64) |

---

## Features

### Library

Use `brainbit` as a library in your own project:

```toml
[dependencies]
# Full build (includes the ratatui TUI feature):
brainbit = "0.0.2"

# Library only — skips ratatui / crossterm compilation:
brainbit = { version = "0.0.2", default-features = false }
```

```rust
use brainbit::prelude::*;
use std::time::Duration;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Optional: block internet access for the process
    block_internet()?;

    // Scan for devices
    let scanner = Scanner::new(&[SensorFamily::LEBrainBit])?;
    scanner.start()?;
    std::thread::sleep(Duration::from_secs(5));
    scanner.stop()?;

    let devices = scanner.devices()?;
    let mut device = BrainBitDevice::connect(&scanner, &devices[0])?;

    println!("Connected: {}", device.name()?);
    println!("Battery:   {}%", device.battery_level()?);

    // Stream 4 seconds of EEG
    let samples = device.capture_signal(BRAINBIT_SAMPLING_RATE as usize * 4)?;
    for s in &samples[..5] {
        println!(
            "#{}: O1={:.6}V O2={:.6}V T3={:.6}V T4={:.6}V",
            s.pack_num, s.channels[0], s.channels[1], s.channels[2], s.channels[3]
        );
    }

    device.disconnect()?;
    Ok(())
}
```

### CLI

```bash
cargo run --bin brainbit --no-default-features
RUST_LOG=debug cargo run --bin brainbit --no-default-features
```

### TUI

Real-time 4-channel EEG waveform display in the terminal:

```bash
cargo run --bin brainbit-tui
```

---

## Security

Three layers of defense for the closed-source native library:

### 1. SHA-256 integrity verification

```bash
# At download time (automatic)
./sdk/download.sh

# At runtime (opt-in)
BRAINBIT_VERIFY_SDK=1 cargo run

# Programmatic
verify_library("/path/to/libneurosdk2.dylib")?;
```

### 2. Pinned Git commits

Downloads from exact commit SHAs, not `main`. Update `sdk/download.sh`
to upgrade.

### 3. OS-level network sandbox

```rust
use brainbit::prelude::*;

block_internet()?; // irrevocable — process can never reach the internet
// BLE still works (uses IPC, not sockets)
```

| Platform | Mechanism | Blocks | Allows |
|---|---|---|---|
| Linux | seccomp-bpf | `AF_INET`/`AF_INET6` sockets | `AF_UNIX` (D-Bus), `AF_BLUETOOTH` |
| macOS | Seatbelt sandbox | `network-outbound` (remote) | XPC (CoreBluetooth), IPC |
| Windows | Windows Firewall rule | Outbound TCP/UDP | WinRT Bluetooth APIs |

---

## Project layout

```
brainbit-rs/
├── Cargo.toml
├── README.md
├── CHANGELOG.md
├── LICENSE
└── src/
    ├── lib.rs            # Crate root: modules + prelude
    ├── main.rs           # Headless CLI binary
    ├── bin/
    │   └── tui.rs        # Full-screen TUI binary (ratatui)
    ├── ffi.rs            # Cross-platform NeuroSDK2 FFI (runtime-loaded, 175 functions)
    ├── types.rs          # #[repr(C)] FFI types matching cmn_type.h
    ├── scanner.rs        # BLE device scanner
    ├── device.rs         # High-level device API (signal, resist, battery)
    ├── error.rs          # BrainBitError
    ├── verify.rs         # SHA-256 integrity verification
    └── sandbox.rs        # OS-level network sandboxing
├── sdk/
│   ├── download.sh       # Download + verify native libraries
│   └── checksums.sha256  # Pinned SHA-256 hashes
├── examples/
│   ├── scan.rs           # Device discovery
│   ├── stream.rs         # Signal streaming with callback
│   ├── resist.rs         # Electrode resistance measurement
│   └── sandbox_test.rs   # Network sandbox verification
└── tests/
    └── types_tests.rs    # FFI type layouts, enum values, ABI checks (29 tests)
```

---

## Examples

```bash
# Scan for devices
cargo run --example scan

# Stream EEG data with callback
cargo run --example stream

# Measure electrode resistance
cargo run --example resist

# Verify network sandbox works
cargo run --example sandbox_test
```

---

## Dependencies

| Crate | Purpose |
|---|---|
| [libloading](https://crates.io/crates/libloading) | Runtime DLL/so/dylib loading for NeuroSDK2 |
| [thiserror](https://crates.io/crates/thiserror) | Error type derivation |
| [log](https://crates.io/crates/log) | Logging facade |
| [env_logger](https://crates.io/crates/env_logger) | Log output for binaries |
| [libc](https://crates.io/crates/libc) | seccomp-bpf syscalls (Linux sandbox) |
| [ratatui](https://ratatui.rs) | Terminal UI (optional, `tui` feature) |
| [crossterm](https://github.com/crossterm-rs/crossterm) | Terminal backend (optional, `tui` feature) |

---

## Running tests

```bash
cargo test
```

29 unit tests cover FFI type layouts, enum discriminant values, ABI struct
sizes, string extraction, SHA-256 correctness, and sampling frequency
conversion — all run without hardware.

---

## License

[MIT](./LICENSE)
