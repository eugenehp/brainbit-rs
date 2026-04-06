//! # brainbit
//!
//! Rust library for **BrainBit** EEG headband devices via the
//! [NeuroSDK2](https://sdk.brainbit.com/) C library, loaded at runtime.
//!
//! Supports **BrainBit** (original 4-channel), **BrainBit 2**,
//! **BrainBit Pro**, **BrainBit Flex 4/8**, and related devices from
//! [BrainBit LLC](https://brainbit.com/).
//!
//! ## Cross-platform
//!
//! Works on **Windows**, **Linux**, and **macOS**.  The `neurosdk2` shared
//! library (`neurosdk2.dll` / `libneurosdk2.so` / `libneurosdk2.dylib`) is
//! loaded at runtime via `libloading` — no build-time C dependencies.
//!
//! Download the native library for your platform:
//! - **Windows**: <https://github.com/BrainbitLLC/neurosdk2-cpp>
//! - **Linux**: <https://github.com/BrainbitLLC/linux_neurosdk2>
//! - **macOS**: <https://github.com/BrainbitLLC/apple_neurosdk2>
//!
//! ## Quick start
//!
//! ```rust,ignore
//! use brainbit::prelude::*;
//! use std::time::Duration;
//!
//! // 1. Scan for devices
//! let scanner = Scanner::new(&[SensorFamily::LEBrainBit])?;
//! scanner.start()?;
//! std::thread::sleep(Duration::from_secs(5));
//! scanner.stop()?;
//!
//! let devices = scanner.devices()?;
//! if devices.is_empty() {
//!     eprintln!("No BrainBit device found!");
//!     return Ok(());
//! }
//!
//! // 2. Connect
//! let mut device = BrainBitDevice::connect(&scanner, &devices[0])?;
//! println!("Connected to: {}", device.name()?);
//! println!("Battery: {}%", device.battery_level()?);
//! println!("Firmware: {:?}", device.firmware_version()?);
//!
//! // 3. Stream EEG for 4 seconds
//! let samples = device.capture_signal(BRAINBIT_SAMPLING_RATE as usize * 4)?;
//! for s in &samples[..5] {
//!     println!("#{}: O1={:.6}V O2={:.6}V T3={:.6}V T4={:.6}V",
//!         s.pack_num, s.channels[0], s.channels[1], s.channels[2], s.channels[3]);
//! }
//! ```
//!
//! ## Module overview
//!
//! | Module | Purpose |
//! |---|---|
//! | [`prelude`] | One-line glob import of the most commonly needed types |
//! | [`ffi`] | Cross-platform FFI bindings for NeuroSDK2 (runtime-loaded) |
//! | [`types`] | C-compatible FFI types, enums, and structures |
//! | [`scanner`] | BLE device scanner |
//! | [`device`] | High-level device API: signal streaming, resistance, battery |
//! | [`error`] | Error types |

pub mod ffi;
pub mod types;
pub mod error;
pub mod verify;
pub mod sandbox;
pub mod scanner;
pub mod device;

/// Convenience re-exports for downstream crates.
///
/// ```rust,ignore
/// use brainbit::prelude::*;
///
/// let scanner = Scanner::new(&[SensorFamily::LEBrainBit])?;
/// ```
pub mod prelude {
    // ── Error ─────────────────────────────────────────────────────────────
    pub use crate::error::BrainBitError;

    // ── Types ─────────────────────────────────────────────────────────────
    pub use crate::types::{
        SensorFamily, SensorFeature, SensorCommand, SensorParameter,
        SensorParamAccess, SensorState, SensorSamplingFrequency,
        SensorGain, SensorDataOffset, SensorFilter, SensorFirmwareMode,
        SensorVersion, SensorInfo, ParameterInfo,
        BrainBitSignalData, BrainBitResistData,
        EEGChannelId, EEGChannelType, EEGChannelInfo,
        BrainBit2ChannelMode, BrainBit2AmplifierParam, GenCurrent,
        MEMSData,
    };

    // ── Scanner ───────────────────────────────────────────────────────────
    pub use crate::scanner::Scanner;

    // ── Device ────────────────────────────────────────────────────────────
    pub use crate::device::{
        BrainBitDevice, EegSample, ResistanceSample,
        BRAINBIT_CHANNEL_NAMES, BRAINBIT_SAMPLING_RATE, BRAINBIT_NUM_CHANNELS,
    };

    // ── Verification ──────────────────────────────────────────────────────
    pub use crate::verify::{verify_library, find_and_verify_library};

    // ── Sandboxing ───────────────────────────────────────────────────────
    pub use crate::sandbox::{block_internet, is_sandboxed};
}
