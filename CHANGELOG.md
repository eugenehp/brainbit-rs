# Changelog

All notable changes to this project will be documented in this file.

## [0.0.2] - 2026-04-08

### Fixed
- Windows build fix: add missing `ctrlc` dependency for `sandbox.rs` Ctrl+C handler (`ctrlc::set_handler`).
- Restores successful compilation for downstream crates that depend on `brainbit` on Windows.

## [0.0.1] - 2026-04-06

### Added
- Initial release
- Runtime-loaded FFI bindings to NeuroSDK2 C library (175/175 functions)
- Full `#[repr(C)]` type parity with `cmn_type.h`
- BLE scanner with callback support
- High-level `BrainBitDevice` API (signal, resistance, battery, firmware)
- Support for BrainBit, BrainBit 2, BrainBit Pro, Flex 4/8, Callibri, Headphones, Headband, NeuroEEG
- SHA-256 integrity verification of native libraries (`verify.rs`)
- OS-level network sandboxing (`sandbox.rs`) — seccomp on Linux, Seatbelt on macOS, Firewall on Windows
- Pinned-commit download script with checksum verification (`sdk/download.sh`)
- CLI binary for scan + connect + stream
- Real-time ratatui TUI with 4-channel EEG charts
- Examples: scan, stream, resist, sandbox_test
- Cross-platform: Windows, Linux, macOS
