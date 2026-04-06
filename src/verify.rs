//! Runtime integrity verification for the NeuroSDK2 native library.
//!
//! Computes SHA-256 of the loaded shared library and compares it against
//! known-good hashes pinned at compile time. This guards against loading
//! a tampered or incompatible binary.

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::error::BrainBitError;

/// Known-good SHA-256 hashes for official NeuroSDK2 binaries.
///
/// Update these when upgrading to a new SDK version. The hashes correspond
/// to specific Git commits in the BrainbitLLC repositories:
///
/// - `apple_neurosdk2`  commit `c0497ead740b` (2025-04-30)
/// - `neurosdk2-cpp`    commit `c10abc74fb61` (2025-04-09)
/// - `linux_neurosdk2`  commit `9f09ad459078` (2025-03-31)
fn known_hashes() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    m.insert(
        "libneurosdk2.dylib",
        "047b9d533cbd35bb0fcc360dcb87de8d66bf7167924ea36bafb1837950b51d58",
    );
    m.insert(
        "neurosdk2-x64.dll",
        "0f11fa32795512f0e8a9bb100c10a4ea169f305ced750de1263534842466569e",
    );
    m.insert(
        "neurosdk2-x32.dll",
        "2f65aee8def262d5da988721fcb1386f35e795f63159bf1163d8d8ec388b9eeb",
    );
    // On Windows the library is typically named just "neurosdk2.dll" when
    // placed alongside the binary. Accept both the arch-specific and
    // generic names.
    m.insert(
        "neurosdk2.dll",
        "0f11fa32795512f0e8a9bb100c10a4ea169f305ced750de1263534842466569e",
    );
    m.insert(
        "libneurosdk2.so",
        "f57092f6ab1c1f2e7dc28ea773acb6813dcef5fa86f043ce8a5292fbee493d88",
    );
    m
}

/// Simple SHA-256 implementation (no external dependency).
/// Uses the FIPS 180-4 algorithm directly.
fn sha256(data: &[u8]) -> [u8; 32] {
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5,
        0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
        0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3,
        0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
        0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc,
        0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
        0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7,
        0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
        0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13,
        0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
        0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3,
        0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
        0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5,
        0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208,
        0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
    ];

    let bit_len = (data.len() as u64) * 8;

    // Pad message
    let mut msg = data.to_vec();
    msg.push(0x80);
    while (msg.len() % 64) != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bit_len.to_be_bytes());

    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
        0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
    ];

    for chunk in msg.chunks(64) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([
                chunk[4 * i],
                chunk[4 * i + 1],
                chunk[4 * i + 2],
                chunk[4 * i + 3],
            ]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16]
                .wrapping_add(s0)
                .wrapping_add(w[i - 7])
                .wrapping_add(s1);
        }

        let (mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh) =
            (h[0], h[1], h[2], h[3], h[4], h[5], h[6], h[7]);

        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = hh
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);

            hh = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }

        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(hh);
    }

    let mut result = [0u8; 32];
    for (i, &val) in h.iter().enumerate() {
        result[4 * i..4 * i + 4].copy_from_slice(&val.to_be_bytes());
    }
    result
}

fn bytes_to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
}

/// Verify the integrity of a NeuroSDK2 library file.
///
/// Returns `Ok(())` if the SHA-256 hash matches a known-good value,
/// or `Err` with details if verification fails.
///
/// # Example
///
/// ```rust,ignore
/// use brainbit::verify::verify_library;
///
/// verify_library("/usr/local/lib/libneurosdk2.dylib")?;
/// println!("Library integrity verified!");
/// ```
pub fn verify_library<P: AsRef<Path>>(path: P) -> Result<(), BrainBitError> {
    let path = path.as_ref();
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");

    let known = known_hashes();
    let expected = known.get(file_name).ok_or_else(|| {
        BrainBitError::LibraryNotAvailable {
            reason: format!(
                "No known checksum for '{}'. Expected one of: {}",
                file_name,
                known.keys().cloned().collect::<Vec<_>>().join(", ")
            ),
        }
    })?;

    let data = fs::read(path).map_err(|e| BrainBitError::LibraryNotAvailable {
        reason: format!("Cannot read '{}': {}", path.display(), e),
    })?;

    let actual = bytes_to_hex(&sha256(&data));

    if actual != *expected {
        return Err(BrainBitError::LibraryNotAvailable {
            reason: format!(
                "INTEGRITY CHECK FAILED for '{}'!\n\
                 Expected SHA-256: {}\n\
                 Actual SHA-256:   {}\n\
                 \n\
                 The library may have been tampered with or is an unrecognised version.\n\
                 Download the official library from:\n\
                 - macOS:   https://github.com/BrainbitLLC/apple_neurosdk2\n\
                 - Linux:   https://github.com/BrainbitLLC/linux_neurosdk2\n\
                 - Windows: https://github.com/BrainbitLLC/neurosdk2-cpp",
                file_name, expected, actual
            ),
        });
    }

    log::info!("Library integrity verified: {} (SHA-256: {})", file_name, &actual[..16]);
    Ok(())
}

/// Find and verify the NeuroSDK2 library for the current platform.
///
/// Searches common locations:
/// 1. `./sdk/lib/{platform}/` (project-local, from `download.sh`)
/// 2. Same directory as the running executable
/// 3. System library paths
///
/// Returns the verified path on success.
pub fn find_and_verify_library() -> Result<std::path::PathBuf, BrainBitError> {
    let (lib_name, platform_dir) = if cfg!(target_os = "macos") {
        ("libneurosdk2.dylib", "macos")
    } else if cfg!(target_os = "linux") {
        ("libneurosdk2.so", "linux")
    } else if cfg!(target_os = "windows") {
        if cfg!(target_pointer_width = "64") {
            ("neurosdk2-x64.dll", "windows")
        } else {
            ("neurosdk2-x32.dll", "windows")
        }
    } else {
        return Err(BrainBitError::LibraryNotAvailable {
            reason: "Unsupported platform".into(),
        });
    };

    // Search locations in priority order
    let mut candidates: Vec<std::path::PathBuf> = Vec::new();

    // 1. Project-local sdk/lib/
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            // When run from target/debug/, look up to project root
            for ancestor in exe_dir.ancestors().take(5) {
                let sdk_path = ancestor.join("sdk").join("lib").join(platform_dir).join(lib_name);
                candidates.push(sdk_path);
            }
            // 2. Same directory as executable
            candidates.push(exe_dir.join(lib_name));
        }
    }

    // 3. Current working directory
    candidates.push(std::path::PathBuf::from(lib_name));
    candidates.push(
        std::path::PathBuf::from("sdk")
            .join("lib")
            .join(platform_dir)
            .join(lib_name),
    );

    for path in &candidates {
        if path.exists() {
            verify_library(path)?;
            return Ok(path.clone());
        }
    }

    Err(BrainBitError::LibraryNotAvailable {
        reason: format!(
            "NeuroSDK2 library '{}' not found. Run: ./sdk/download.sh\n\
             Searched:\n{}",
            lib_name,
            candidates
                .iter()
                .map(|p| format!("  - {}", p.display()))
                .collect::<Vec<_>>()
                .join("\n")
        ),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sha256_empty() {
        let hash = bytes_to_hex(&sha256(b""));
        assert_eq!(
            hash,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn test_sha256_hello() {
        let hash = bytes_to_hex(&sha256(b"hello"));
        assert_eq!(
            hash,
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
    }

    #[test]
    fn test_sha256_known_vector() {
        // "abc" => ba7816bf...
        let hash = bytes_to_hex(&sha256(b"abc"));
        assert_eq!(
            hash,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }
}
