//! BLE device scanner for discovering BrainBit devices.
//!
//! Wraps the NeuroSDK2 scanner API. Typical usage:
//!
//! ```rust,ignore
//! use brainbit::prelude::*;
//!
//! let mut scanner = Scanner::new(&[SensorFamily::LEBrainBit])?;
//! scanner.start()?;
//! std::thread::sleep(std::time::Duration::from_secs(5));
//! scanner.stop()?;
//! let devices = scanner.devices()?;
//! println!("Found {} device(s)", devices.len());
//! ```

use std::ptr;

use crate::error::BrainBitError;
use crate::ffi::sdk_lib;
use crate::types::*;

/// Check an `OpStatus` and convert to `Result`.
pub(crate) fn check_status(status: &OpStatus) -> Result<(), BrainBitError> {
    if status.is_ok() {
        Ok(())
    } else {
        Err(BrainBitError::SdkError {
            code: status.error,
            message: status.message(),
        })
    }
}

/// BLE scanner that discovers BrainBit (and related) devices.
pub struct Scanner {
    ptr: *mut SensorScanner,
}

// The scanner pointer is only used through the SDK's thread-safe API.
unsafe impl Send for Scanner {}

impl Scanner {
    /// Create a new scanner filtering for the given device families.
    ///
    /// Common families:
    /// - `SensorFamily::LEBrainBit` — original BrainBit, Flex v1
    /// - `SensorFamily::LEBrainBit2` — BrainBit 2
    /// - `SensorFamily::LEBrainBitPro` — BrainBit Pro
    /// - `SensorFamily::LEBrainBitFlex` — BrainBit Flex 4/8
    pub fn new(families: &[SensorFamily]) -> Result<Self, BrainBitError> {
        let lib = sdk_lib()?;
        let mut status = OpStatus::default();
        let ptr = unsafe {
            (lib.fn_create_scanner)(
                families.as_ptr(),
                families.len() as i32,
                &mut status,
            )
        };
        check_status(&status)?;
        if ptr.is_null() {
            return Err(BrainBitError::NullPointer);
        }
        Ok(Scanner { ptr })
    }

    /// Start scanning for devices.
    pub fn start(&self) -> Result<(), BrainBitError> {
        let lib = sdk_lib()?;
        let mut status = OpStatus::default();
        unsafe { (lib.fn_start_scanner)(self.ptr, &mut status, 1) };
        check_status(&status)
    }

    /// Stop scanning for devices.
    pub fn stop(&self) -> Result<(), BrainBitError> {
        let lib = sdk_lib()?;
        let mut status = OpStatus::default();
        unsafe { (lib.fn_stop_scanner)(self.ptr, &mut status) };
        check_status(&status)
    }

    /// Get the list of discovered devices so far.
    pub fn devices(&self) -> Result<Vec<SensorInfo>, BrainBitError> {
        let lib = sdk_lib()?;
        let mut status = OpStatus::default();
        let mut count: i32 = 32;
        let mut sensors: Vec<SensorInfo> = vec![SensorInfo::default(); count as usize];
        unsafe {
            (lib.fn_sensors_scanner)(
                self.ptr,
                sensors.as_mut_ptr(),
                &mut count,
                &mut status,
            );
        }
        check_status(&status)?;
        sensors.truncate(count.max(0) as usize);
        Ok(sensors)
    }

    /// Register a callback that fires whenever a new device is discovered.
    ///
    /// Returns a handle that must be kept alive. Drop it (or call
    /// [`Scanner::remove_callback`]) to unsubscribe.
    pub fn on_device_found<F>(
        &self,
        callback: F,
    ) -> Result<ScannerCallbackHandle, BrainBitError>
    where
        F: FnMut(&[SensorInfo]) + Send + 'static,
    {
        let lib = sdk_lib()?;
        let mut status = OpStatus::default();
        let boxed: Box<Box<dyn FnMut(&[SensorInfo]) + Send>> = Box::new(Box::new(callback));
        let user_data = Box::into_raw(boxed) as *mut std::ffi::c_void;

        let mut handle: SensorsListenerHandle = ptr::null_mut();
        unsafe {
            (lib.fn_add_sensors_callback)(
                self.ptr,
                scanner_trampoline,
                &mut handle,
                user_data,
                &mut status,
            );
        }
        check_status(&status)?;
        Ok(ScannerCallbackHandle { handle, user_data })
    }

    /// Remove a previously registered callback.
    pub fn remove_callback(&self, cb: ScannerCallbackHandle) {
        if let Ok(lib) = sdk_lib() {
            unsafe {
                (lib.fn_remove_sensors_callback)(cb.handle);
                // Reclaim the closure memory
                let _ = Box::from_raw(cb.user_data as *mut Box<dyn FnMut(&[SensorInfo]) + Send>);
            }
        }
    }

    /// Create a `Sensor` (device) from discovered `SensorInfo`.
    ///
    /// This initiates a BLE connection. On success the device is connected
    /// and ready for use.
    pub(crate) fn create_sensor_raw(&self, info: &SensorInfo) -> Result<*mut Sensor, BrainBitError> {
        let lib = sdk_lib()?;
        let mut status = OpStatus::default();
        let ptr = unsafe { (lib.fn_create_sensor)(self.ptr, info.clone(), &mut status) };
        check_status(&status)?;
        if ptr.is_null() {
            return Err(BrainBitError::NullPointer);
        }
        Ok(ptr)
    }
}

impl Drop for Scanner {
    fn drop(&mut self) {
        if let Ok(lib) = sdk_lib() {
            unsafe { (lib.fn_free_scanner)(self.ptr) };
        }
    }
}

/// Handle for a scanner callback. Dropping it does NOT automatically unregister;
/// call [`Scanner::remove_callback`] explicitly.
pub struct ScannerCallbackHandle {
    handle: SensorsListenerHandle,
    user_data: *mut std::ffi::c_void,
}

unsafe impl Send for ScannerCallbackHandle {}

/// C-compatible trampoline that invokes the Rust closure.
unsafe extern "C" fn scanner_trampoline(
    _scanner: *mut SensorScanner,
    sensors: *mut SensorInfo,
    count: i32,
    user_data: *mut std::ffi::c_void,
) {
    if user_data.is_null() || sensors.is_null() || count <= 0 {
        return;
    }
    let closure = &mut *(user_data as *mut Box<dyn FnMut(&[SensorInfo]) + Send>);
    let slice = std::slice::from_raw_parts(sensors, count as usize);
    closure(slice);
}
