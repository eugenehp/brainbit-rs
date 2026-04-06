//! Process-level network sandboxing for the NeuroSDK2 library.
//!
//! The `neurosdk2` native library is closed-source. Although static analysis
//! shows it only uses CoreBluetooth (macOS) and D-Bus/BlueZ (Linux) with no
//! network imports, this module provides defense-in-depth by blocking internet
//! access at the OS level.
//!
//! # How it works
//!
//! | Platform | Mechanism | What's blocked | What's allowed |
//! |---|---|---|---|
//! | **Linux** | `seccomp-bpf` | `AF_INET`/`AF_INET6` socket creation | `AF_UNIX` (D-Bus), `AF_BLUETOOTH` |
//! | **macOS** | `sandbox_init` (Seatbelt) | `network-outbound` to remote | IPC, Bluetooth (XPC) |
//! | **Windows** | Windows Firewall rule | TCP/UDP outbound for this exe | Bluetooth (WinRT API) |
//!
//! # Usage
//!
//! ```rust,ignore
//! use brainbit::sandbox;
//!
//! // Call BEFORE loading the SDK library
//! sandbox::block_internet()?;
//!
//! // Now load and use the SDK — it cannot make internet connections
//! let scanner = Scanner::new(&[SensorFamily::LEBrainBit])?;
//! ```
//!
//! # Important
//!
//! - This is **process-wide** — your own code also loses internet access
//! - BLE/Bluetooth continues to work (it uses IPC, not internet sockets)
//! - On Linux, `seccomp` filters are **irrevocable** once applied
//! - Call this **before** `sdk_lib()` / `Scanner::new()` for best protection

use crate::error::BrainBitError;

/// Block all internet (IPv4/IPv6) access for the current process.
///
/// BLE/Bluetooth and local IPC (Unix sockets, XPC, D-Bus) remain functional.
/// This is irrevocable on Linux (seccomp) and macOS (sandbox).
///
/// Returns `Ok(())` on success or if sandboxing is not available.
#[cfg(target_os = "linux")]
pub fn block_internet() -> Result<(), BrainBitError> {
    linux_sandbox::apply_seccomp_filter()
}

#[cfg(target_os = "macos")]
pub fn block_internet() -> Result<(), BrainBitError> {
    macos_sandbox::apply_sandbox_profile()
}

#[cfg(target_os = "windows")]
pub fn block_internet() -> Result<(), BrainBitError> {
    windows_sandbox::apply_firewall_rule()
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
pub fn block_internet() -> Result<(), BrainBitError> {
    log::warn!("Network sandboxing not implemented for this platform");
    Ok(())
}

/// Check if sandboxing is active / supported on this platform.
#[cfg(target_os = "linux")]
pub fn is_sandboxed() -> bool {
    linux_sandbox::is_seccomp_active()
}

#[cfg(target_os = "macos")]
pub fn is_sandboxed() -> bool {
    macos_sandbox::is_sandboxed()
}

#[cfg(not(any(target_os = "linux", target_os = "macos")))]
pub fn is_sandboxed() -> bool {
    false
}

// ── Linux: seccomp-bpf ���───────���─────────────────────────────────────────────

#[cfg(target_os = "linux")]
mod linux_sandbox {
    use crate::error::BrainBitError;
    use std::sync::atomic::{AtomicBool, Ordering};

    static SECCOMP_APPLIED: AtomicBool = AtomicBool::new(false);

    // Syscall numbers (x86_64)
    const SYS_SOCKET: u32 = 41;

    // Address families
    const AF_INET: u32 = 2;
    const AF_INET6: u32 = 10;

    // seccomp constants
    const SECCOMP_SET_MODE_FILTER: libc::c_ulong = 1;
    const SECCOMP_RET_ALLOW: u32 = 0x7fff_0000;
    const SECCOMP_RET_ERRNO: u32 = 0x0005_0000;
    const EPERM: u32 = 1;

    // BPF instruction encoding
    const BPF_LD_W_ABS: u16 = 0x20;  // BPF_LD | BPF_W | BPF_ABS
    const BPF_JMP_JEQ_K: u16 = 0x15; // BPF_JMP | BPF_JEQ | BPF_K
    const BPF_RET_K: u16 = 0x06;     // BPF_RET | BPF_K

    // seccomp_data field offsets
    const OFFSET_NR: u32 = 0;       // syscall number
    const OFFSET_ARG0: u32 = 16;    // first argument

    #[repr(C)]
    struct SockFprog {
        len: u16,
        filter: *const SockFilter,
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct SockFilter {
        code: u16,
        jt: u8,
        jf: u8,
        k: u32,
    }

    /// Apply a seccomp-bpf filter that blocks socket(AF_INET/AF_INET6)
    /// while allowing everything else (AF_UNIX, AF_BLUETOOTH, etc.).
    ///
    /// BPF program:
    /// ```text
    /// [0] load syscall number
    /// [1] if != SYS_socket → ALLOW
    /// [2] load arg0 (address family)
    /// [3] if == AF_INET → ERRNO(EPERM)
    /// [4] if == AF_INET6 → ERRNO(EPERM)
    /// [5] ALLOW
    /// [6] ERRNO(EPERM)
    /// ```
    pub fn apply_seccomp_filter() -> Result<(), BrainBitError> {
        if SECCOMP_APPLIED.load(Ordering::Relaxed) {
            return Ok(());
        }

        // Required to install seccomp without CAP_SYS_ADMIN
        let ret = unsafe { libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0) };
        if ret != 0 {
            return Err(BrainBitError::NotSupported(
                "prctl(PR_SET_NO_NEW_PRIVS) failed".into(),
            ));
        }

        let filter: [SockFilter; 7] = [
            // [0] Load syscall number
            SockFilter { code: BPF_LD_W_ABS, jt: 0, jf: 0, k: OFFSET_NR },
            // [1] If != SYS_socket → jump to [5] (ALLOW)
            SockFilter { code: BPF_JMP_JEQ_K, jt: 0, jf: 3, k: SYS_SOCKET },
            // [2] Load first argument (address family)
            SockFilter { code: BPF_LD_W_ABS, jt: 0, jf: 0, k: OFFSET_ARG0 },
            // [3] If AF_INET → jump to [6] (BLOCK)
            SockFilter { code: BPF_JMP_JEQ_K, jt: 2, jf: 0, k: AF_INET },
            // [4] If AF_INET6 → jump to [6] (BLOCK), else → [5] (ALLOW)
            SockFilter { code: BPF_JMP_JEQ_K, jt: 1, jf: 0, k: AF_INET6 },
            // [5] ALLOW
            SockFilter { code: BPF_RET_K, jt: 0, jf: 0, k: SECCOMP_RET_ALLOW },
            // [6] BLOCK with EPERM
            SockFilter { code: BPF_RET_K, jt: 0, jf: 0, k: SECCOMP_RET_ERRNO | EPERM },
        ];

        let prog = SockFprog {
            len: filter.len() as u16,
            filter: filter.as_ptr(),
        };

        let ret = unsafe {
            libc::syscall(
                libc::SYS_seccomp,
                SECCOMP_SET_MODE_FILTER,
                0u64,
                &prog as *const _,
            )
        };

        if ret != 0 {
            let err = std::io::Error::last_os_error();
            return Err(BrainBitError::NotSupported(format!(
                "seccomp(SET_MODE_FILTER) failed: {}", err
            )));
        }

        SECCOMP_APPLIED.store(true, Ordering::Relaxed);
        log::info!("seccomp: AF_INET/AF_INET6 socket creation blocked");
        Ok(())
    }

    pub fn is_seccomp_active() -> bool {
        SECCOMP_APPLIED.load(Ordering::Relaxed)
    }
}

// ── macOS: Seatbelt sandbox ──��──────────────────────────────────────────────

#[cfg(target_os = "macos")]
mod macos_sandbox {
    use crate::error::BrainBitError;
    use std::ffi::{CStr, CString};
    use std::os::raw::c_char;
    use std::ptr;
    use std::sync::atomic::{AtomicBool, Ordering};

    static SANDBOX_APPLIED: AtomicBool = AtomicBool::new(false);

    extern "C" {
        fn sandbox_init(
            profile: *const c_char,
            flags: u64,
            errorbuf: *mut *mut c_char,
        ) -> i32;
        fn sandbox_free_error(errorbuf: *mut c_char);
    }

    /// Apply a macOS sandbox that blocks outbound network to remote IPs.
    ///
    /// CoreBluetooth works through XPC (IPC), not network sockets, so BLE
    /// is unaffected. The profile:
    /// - Allows everything by default (`(allow default)`)
    /// - Denies `network-outbound` to remote IP addresses
    /// - Allows localhost connections (if needed)
    pub fn apply_sandbox_profile() -> Result<(), BrainBitError> {
        if SANDBOX_APPLIED.load(Ordering::Relaxed) {
            return Ok(());
        }

        // Seatbelt profile: allow everything except remote network
        let profile = CString::new(concat!(
            "(version 1)\n",
            "(allow default)\n",
            "(deny network-outbound (remote ip))\n",
        ))
        .map_err(|_| BrainBitError::NotSupported("Invalid sandbox profile".into()))?;

        let mut errorbuf: *mut c_char = ptr::null_mut();
        let ret = unsafe { sandbox_init(profile.as_ptr(), 0, &mut errorbuf) };

        if ret != 0 {
            let msg = if !errorbuf.is_null() {
                let s = unsafe { CStr::from_ptr(errorbuf) }
                    .to_string_lossy()
                    .into_owned();
                unsafe { sandbox_free_error(errorbuf) };
                s
            } else {
                "unknown error".into()
            };
            return Err(BrainBitError::NotSupported(format!(
                "sandbox_init failed: {}", msg
            )));
        }

        SANDBOX_APPLIED.store(true, Ordering::Relaxed);
        log::info!("macOS sandbox: remote network-outbound denied");
        Ok(())
    }

    pub fn is_sandboxed() -> bool {
        SANDBOX_APPLIED.load(Ordering::Relaxed)
    }
}

// ── Windows: Firewall rule ──────────────────────────────────────────────────

#[cfg(target_os = "windows")]
mod windows_sandbox {
    use crate::error::BrainBitError;

    /// Block outbound network for this executable via Windows Firewall.
    ///
    /// Requires **Administrator privileges**. Creates a `netsh advfirewall`
    /// rule named `BrainBitSDK_Block_<PID>`. BLE uses WinRT APIs which go
    /// through the Bluetooth driver stack, not the network stack.
    pub fn apply_firewall_rule() -> Result<(), BrainBitError> {
        let exe = std::env::current_exe().map_err(|e| {
            BrainBitError::NotSupported(format!("Cannot get exe path: {}", e))
        })?;

        let rule_name = format!("BrainBitSDK_Block_{}", std::process::id());

        let output = std::process::Command::new("netsh")
            .args([
                "advfirewall", "firewall", "add", "rule",
                &format!("name={}", rule_name),
                "dir=out", "action=block",
                &format!("program={}", exe.to_string_lossy()),
                "enable=yes", "profile=any",
            ])
            .output()
            .map_err(|e| {
                BrainBitError::NotSupported(format!(
                    "netsh failed (need Administrator?): {}", e
                ))
            })?;

        if !output.status.success() {
            return Err(BrainBitError::NotSupported(format!(
                "Firewall rule failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        log::info!("Windows Firewall: outbound blocked (rule '{}')", rule_name);

        // Cleanup on exit: spawn a thread that removes the rule on Ctrl+C
        let name = rule_name.clone();
        std::thread::spawn(move || {
            // Wait for ctrl+c signal
            let (tx, rx) = std::sync::mpsc::channel();
            let _ = ctrlc::set_handler(move || { let _ = tx.send(()); });
            let _ = rx.recv();
            let _ = std::process::Command::new("netsh")
                .args(["advfirewall", "firewall", "delete", "rule",
                       &format!("name={}", name)])
                .output();
            std::process::exit(0);
        });

        Ok(())
    }
}
