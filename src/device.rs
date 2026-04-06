//! High-level device abstraction for BrainBit sensors.
//!
//! A [`BrainBitDevice`] wraps an opaque SDK sensor pointer and provides
//! safe Rust methods for querying device info, reading EEG signal data,
//! reading resistance, and controlling the device.

use std::ptr;
use std::sync::{Arc, Mutex};

use crate::error::BrainBitError;
use crate::ffi::sdk_lib;
use crate::scanner::{check_status, Scanner};
use crate::types::*;

// ── BrainBit EEG channel names ──────────────────────────────────────────────

/// Standard BrainBit (original) 4-channel EEG electrode names.
pub const BRAINBIT_CHANNEL_NAMES: [&str; 4] = ["O1", "O2", "T3", "T4"];

/// BrainBit sampling rate (Hz).
pub const BRAINBIT_SAMPLING_RATE: u32 = 250;

/// Number of channels on original BrainBit.
pub const BRAINBIT_NUM_CHANNELS: usize = 4;

// ── Convenience types ────────────────────────────────────────────────────────

/// A single 4-channel EEG sample (original BrainBit).
#[derive(Debug, Clone, Copy)]
pub struct EegSample {
    /// Packet sequence number.
    pub pack_num: u32,
    /// Marker byte (for synchronisation / event tagging).
    pub marker: u8,
    /// Channel values in **Volts**: [O1, O2, T3, T4].
    pub channels: [f64; BRAINBIT_NUM_CHANNELS],
}

impl From<&BrainBitSignalData> for EegSample {
    fn from(d: &BrainBitSignalData) -> Self {
        EegSample {
            pack_num: d.pack_num,
            marker: d.marker,
            channels: [d.o1, d.o2, d.t3, d.t4],
        }
    }
}

/// 4-channel resistance reading (original BrainBit), in Ohms.
#[derive(Debug, Clone, Copy)]
pub struct ResistanceSample {
    pub o1: f64,
    pub o2: f64,
    pub t3: f64,
    pub t4: f64,
}

impl From<&BrainBitResistData> for ResistanceSample {
    fn from(d: &BrainBitResistData) -> Self {
        ResistanceSample {
            o1: d.o1,
            o2: d.o2,
            t3: d.t3,
            t4: d.t4,
        }
    }
}

// ── Device ───────────────────────────────────────────────────────────────────

/// A connected BrainBit EEG device.
///
/// Created via [`Scanner`] → [`BrainBitDevice::connect`].
///
/// ```rust,ignore
/// use brainbit::prelude::*;
///
/// let scanner = Scanner::new(&[SensorFamily::LEBrainBit])?;
/// scanner.start()?;
/// std::thread::sleep(std::time::Duration::from_secs(5));
/// scanner.stop()?;
///
/// let devices = scanner.devices()?;
/// let device = BrainBitDevice::connect(&scanner, &devices[0])?;
///
/// println!("Name: {}", device.name()?);
/// println!("Battery: {}%", device.battery_level()?);
/// println!("Version: {:?}", device.firmware_version()?);
/// ```
pub struct BrainBitDevice {
    ptr: *mut Sensor,
    info: SensorInfo,
    /// Active signal callback handle (if any).
    signal_handle: Option<BrainBitSignalDataListenerHandle>,
    /// Active resist callback handle (if any).
    resist_handle: Option<BrainBitResistDataListenerHandle>,
    /// Active BB2 signal callback handle (if any).
    signal_handle_bb2: Option<BrainBit2SignalDataListenerHandle>,
    /// Active BB2 resist callback handle (if any).
    resist_handle_bb2: Option<BrainBit2ResistDataListenerHandle>,
    /// Active battery callback handle.
    battery_handle: Option<BattPowerListenerHandle>,
    /// Active connection state callback handle.
    state_handle: Option<SensorStateListenerHandle>,
    /// Active MEMS callback handle.
    mems_handle: Option<MEMSDataListenerHandle>,
    /// Raw user_data pointers that need cleanup.
    user_data_ptrs: Vec<*mut std::ffi::c_void>,
}

// The sensor pointer is only used through the SDK's thread-safe API.
unsafe impl Send for BrainBitDevice {}

impl BrainBitDevice {
    /// Connect to a discovered device.
    pub fn connect(scanner: &Scanner, info: &SensorInfo) -> Result<Self, BrainBitError> {
        let ptr = scanner.create_sensor_raw(info)?;
        Ok(BrainBitDevice {
            ptr,
            info: info.clone(),
            signal_handle: None,
            resist_handle: None,
            signal_handle_bb2: None,
            resist_handle_bb2: None,
            battery_handle: None,
            state_handle: None,
            mems_handle: None,
            user_data_ptrs: Vec::new(),
        })
    }

    /// The original `SensorInfo` from discovery.
    pub fn sensor_info(&self) -> &SensorInfo {
        &self.info
    }

    /// Device family.
    pub fn family(&self) -> SensorFamily {
        let lib = sdk_lib().expect("SDK loaded");
        unsafe { (lib.fn_get_family)(self.ptr) }
    }

    // ── Properties ───────────────────────────────────────────────────────

    /// Read the device name.
    pub fn name(&self) -> Result<String, BrainBitError> {
        let lib = sdk_lib()?;
        let mut buf = [0u8; SENSOR_NAME_LEN];
        let mut status = OpStatus::default();
        unsafe {
            (lib.fn_read_name)(self.ptr, buf.as_mut_ptr(), SENSOR_NAME_LEN as i32, &mut status);
        }
        check_status(&status)?;
        let nul = buf.iter().position(|&b| b == 0).unwrap_or(SENSOR_NAME_LEN);
        Ok(String::from_utf8_lossy(&buf[..nul]).into_owned())
    }

    /// Write a new device name.
    pub fn set_name(&self, name: &str) -> Result<(), BrainBitError> {
        let lib = sdk_lib()?;
        let mut buf = [0u8; SENSOR_NAME_LEN];
        let bytes = name.as_bytes();
        let len = bytes.len().min(SENSOR_NAME_LEN - 1);
        buf[..len].copy_from_slice(&bytes[..len]);
        let mut status = OpStatus::default();
        unsafe {
            (lib.fn_write_name)(self.ptr, buf.as_mut_ptr(), len as i32, &mut status);
        }
        check_status(&status)
    }

    /// Read the BLE address.
    pub fn address(&self) -> Result<String, BrainBitError> {
        let lib = sdk_lib()?;
        let mut buf = [0u8; SENSOR_ADR_LEN];
        let mut status = OpStatus::default();
        unsafe {
            (lib.fn_read_address)(self.ptr, buf.as_mut_ptr(), SENSOR_ADR_LEN as i32, &mut status);
        }
        check_status(&status)?;
        let nul = buf.iter().position(|&b| b == 0).unwrap_or(SENSOR_ADR_LEN);
        Ok(String::from_utf8_lossy(&buf[..nul]).into_owned())
    }

    /// Read the serial number.
    pub fn serial_number(&self) -> Result<String, BrainBitError> {
        let lib = sdk_lib()?;
        let mut buf = [0u8; SENSOR_SN_LEN];
        let mut status = OpStatus::default();
        unsafe {
            (lib.fn_read_serial_number)(self.ptr, buf.as_mut_ptr(), SENSOR_SN_LEN as i32, &mut status);
        }
        check_status(&status)?;
        let nul = buf.iter().position(|&b| b == 0).unwrap_or(SENSOR_SN_LEN);
        Ok(String::from_utf8_lossy(&buf[..nul]).into_owned())
    }

    /// Read the connection state.
    pub fn state(&self) -> Result<SensorState, BrainBitError> {
        let lib = sdk_lib()?;
        let mut state = SensorState::OutOfRange;
        let mut status = OpStatus::default();
        unsafe { (lib.fn_read_state)(self.ptr, &mut state, &mut status) };
        check_status(&status)?;
        Ok(state)
    }

    /// Read battery power level (0–100 %).
    pub fn battery_level(&self) -> Result<i32, BrainBitError> {
        let lib = sdk_lib()?;
        let mut level: i32 = 0;
        let mut status = OpStatus::default();
        unsafe { (lib.fn_read_batt_power)(self.ptr, &mut level, &mut status) };
        check_status(&status)?;
        Ok(level)
    }

    /// Read battery voltage in mV.
    pub fn battery_voltage(&self) -> Result<i32, BrainBitError> {
        let lib = sdk_lib()?;
        let mut voltage: i32 = 0;
        let mut status = OpStatus::default();
        unsafe { (lib.fn_read_batt_voltage)(self.ptr, &mut voltage, &mut status) };
        check_status(&status)?;
        Ok(voltage)
    }

    /// Read the sampling frequency.
    pub fn sampling_frequency(&self) -> Result<SensorSamplingFrequency, BrainBitError> {
        let lib = sdk_lib()?;
        let mut freq = SensorSamplingFrequency::Unsupported;
        let mut status = OpStatus::default();
        unsafe { (lib.fn_read_sampling_frequency)(self.ptr, &mut freq, &mut status) };
        check_status(&status)?;
        Ok(freq)
    }

    /// Read the current gain setting.
    pub fn gain(&self) -> Result<SensorGain, BrainBitError> {
        let lib = sdk_lib()?;
        let mut gain = SensorGain::Unsupported;
        let mut status = OpStatus::default();
        unsafe { (lib.fn_read_gain)(self.ptr, &mut gain, &mut status) };
        check_status(&status)?;
        Ok(gain)
    }

    /// Write a new gain setting.
    pub fn set_gain(&self, gain: SensorGain) -> Result<(), BrainBitError> {
        let lib = sdk_lib()?;
        let mut status = OpStatus::default();
        unsafe { (lib.fn_write_gain)(self.ptr, gain, &mut status) };
        check_status(&status)
    }

    /// Read firmware mode (Bootloader or Application).
    pub fn firmware_mode(&self) -> Result<SensorFirmwareMode, BrainBitError> {
        let lib = sdk_lib()?;
        let mut mode = SensorFirmwareMode::Application;
        let mut status = OpStatus::default();
        unsafe { (lib.fn_read_firmware_mode)(self.ptr, &mut mode, &mut status) };
        check_status(&status)?;
        Ok(mode)
    }

    /// Read firmware and hardware version.
    pub fn firmware_version(&self) -> Result<SensorVersion, BrainBitError> {
        let lib = sdk_lib()?;
        let mut version = SensorVersion {
            fw_major: 0, fw_minor: 0, fw_patch: 0,
            hw_major: 0, hw_minor: 0, hw_patch: 0,
            ext_major: 0,
        };
        let mut status = OpStatus::default();
        unsafe { (lib.fn_read_version)(self.ptr, &mut version, &mut status) };
        check_status(&status)?;
        Ok(version)
    }

    /// Check if this is a Flex (v1) device (FwMajor >= 100).
    pub fn is_flex_v1(&self) -> Result<bool, BrainBitError> {
        let v = self.firmware_version()?;
        Ok(v.fw_major >= 100)
    }

    // ── Capabilities ─────────────────────────────────────────────────────

    /// Number of EEG channels.
    pub fn channel_count(&self) -> i32 {
        let lib = sdk_lib().expect("SDK loaded");
        unsafe { (lib.fn_get_channels_count)(self.ptr) }
    }

    /// Get supported features.
    pub fn features(&self) -> Result<Vec<SensorFeature>, BrainBitError> {
        let lib = sdk_lib()?;
        let count = unsafe { (lib.fn_get_features_count)(self.ptr) };
        if count <= 0 {
            return Ok(Vec::new());
        }
        let mut features = vec![SensorFeature::Signal; count as usize];
        let mut sz = count;
        let mut status = OpStatus::default();
        unsafe { (lib.fn_get_features)(self.ptr, features.as_mut_ptr(), &mut sz, &mut status) };
        check_status(&status)?;
        features.truncate(sz.max(0) as usize);
        Ok(features)
    }

    /// Check if a specific feature is supported.
    pub fn supports_feature(&self, feature: SensorFeature) -> bool {
        let lib = sdk_lib().expect("SDK loaded");
        unsafe { (lib.fn_is_supported_feature)(self.ptr, feature) != 0 }
    }

    /// Get supported commands.
    pub fn commands(&self) -> Result<Vec<SensorCommand>, BrainBitError> {
        let lib = sdk_lib()?;
        let count = unsafe { (lib.fn_get_commands_count)(self.ptr) };
        if count <= 0 {
            return Ok(Vec::new());
        }
        let mut commands = vec![SensorCommand::StartSignal; count as usize];
        let mut sz = count;
        let mut status = OpStatus::default();
        unsafe { (lib.fn_get_commands)(self.ptr, commands.as_mut_ptr(), &mut sz, &mut status) };
        check_status(&status)?;
        commands.truncate(sz.max(0) as usize);
        Ok(commands)
    }

    /// Check if a specific command is supported.
    pub fn supports_command(&self, command: SensorCommand) -> bool {
        let lib = sdk_lib().expect("SDK loaded");
        unsafe { (lib.fn_is_supported_command)(self.ptr, command) != 0 }
    }

    /// Get supported parameters.
    pub fn parameters(&self) -> Result<Vec<ParameterInfo>, BrainBitError> {
        let lib = sdk_lib()?;
        let count = unsafe { (lib.fn_get_parameters_count)(self.ptr) };
        if count <= 0 {
            return Ok(Vec::new());
        }
        let mut params = vec![
            ParameterInfo {
                param: SensorParameter::Name,
                param_access: SensorParamAccess::Read,
            };
            count as usize
        ];
        let mut sz = count;
        let mut status = OpStatus::default();
        unsafe { (lib.fn_get_parameters)(self.ptr, params.as_mut_ptr(), &mut sz, &mut status) };
        check_status(&status)?;
        params.truncate(sz.max(0) as usize);
        Ok(params)
    }

    /// Check if a specific parameter is supported.
    pub fn supports_parameter(&self, param: SensorParameter) -> bool {
        let lib = sdk_lib().expect("SDK loaded");
        unsafe { (lib.fn_is_supported_parameter)(self.ptr, param) != 0 }
    }

    // ── Commands ─────────────────────────────────────────────────────────

    /// Execute a command on the device.
    pub fn exec_command(&self, command: SensorCommand) -> Result<(), BrainBitError> {
        let lib = sdk_lib()?;
        let mut status = OpStatus::default();
        unsafe { (lib.fn_exec_command)(self.ptr, command, &mut status) };
        check_status(&status)
    }

    /// Start EEG signal streaming.
    pub fn start_signal(&self) -> Result<(), BrainBitError> {
        self.exec_command(SensorCommand::StartSignal)
    }

    /// Stop EEG signal streaming.
    pub fn stop_signal(&self) -> Result<(), BrainBitError> {
        self.exec_command(SensorCommand::StopSignal)
    }

    /// Start resistance measurement.
    pub fn start_resist(&self) -> Result<(), BrainBitError> {
        self.exec_command(SensorCommand::StartResist)
    }

    /// Stop resistance measurement.
    pub fn stop_resist(&self) -> Result<(), BrainBitError> {
        self.exec_command(SensorCommand::StopResist)
    }

    // ── Connection ───────────────────────────────────────────────────────

    /// Reconnect to the device.
    pub fn reconnect(&self) -> Result<(), BrainBitError> {
        let lib = sdk_lib()?;
        let mut status = OpStatus::default();
        unsafe { (lib.fn_connect_sensor)(self.ptr, &mut status) };
        check_status(&status)
    }

    /// Disconnect from the device.
    pub fn disconnect(&self) -> Result<(), BrainBitError> {
        let lib = sdk_lib()?;
        let mut status = OpStatus::default();
        unsafe { (lib.fn_disconnect_sensor)(self.ptr, &mut status) };
        check_status(&status)
    }

    // ── BrainBit (original) signal callback ──────────────────────────────

    /// Subscribe to EEG signal data (original BrainBit, 4 channels).
    ///
    /// The callback receives a slice of `EegSample`s at ~250 Hz.
    /// You must call [`start_signal`](Self::start_signal) after subscribing.
    pub fn on_signal<F>(&mut self, callback: F) -> Result<(), BrainBitError>
    where
        F: FnMut(&[EegSample]) + Send + 'static,
    {
        self.remove_signal_callback();

        let lib = sdk_lib()?;
        let mut status = OpStatus::default();
        let boxed: Box<Box<dyn FnMut(&[EegSample]) + Send>> = Box::new(Box::new(callback));
        let user_data = Box::into_raw(boxed) as *mut std::ffi::c_void;
        self.user_data_ptrs.push(user_data);

        let mut handle: BrainBitSignalDataListenerHandle = ptr::null_mut();
        unsafe {
            (lib.fn_add_signal_cb_bb)(
                self.ptr,
                bb_signal_trampoline,
                &mut handle,
                user_data,
                &mut status,
            );
        }
        check_status(&status)?;
        self.signal_handle = Some(handle);
        Ok(())
    }

    /// Remove the signal callback.
    pub fn remove_signal_callback(&mut self) {
        if let Some(handle) = self.signal_handle.take() {
            if let Ok(lib) = sdk_lib() {
                unsafe { (lib.fn_remove_signal_cb_bb)(handle) };
            }
        }
    }

    // ── BrainBit (original) resist callback ──────────────────────────────

    /// Subscribe to resistance data (original BrainBit, 4 channels).
    ///
    /// You must call [`start_resist`](Self::start_resist) after subscribing.
    pub fn on_resist<F>(&mut self, callback: F) -> Result<(), BrainBitError>
    where
        F: FnMut(ResistanceSample) + Send + 'static,
    {
        self.remove_resist_callback();

        let lib = sdk_lib()?;
        let mut status = OpStatus::default();
        let boxed: Box<Box<dyn FnMut(ResistanceSample) + Send>> = Box::new(Box::new(callback));
        let user_data = Box::into_raw(boxed) as *mut std::ffi::c_void;
        self.user_data_ptrs.push(user_data);

        let mut handle: BrainBitResistDataListenerHandle = ptr::null_mut();
        unsafe {
            (lib.fn_add_resist_cb_bb)(
                self.ptr,
                bb_resist_trampoline,
                &mut handle,
                user_data,
                &mut status,
            );
        }
        check_status(&status)?;
        self.resist_handle = Some(handle);
        Ok(())
    }

    /// Remove the resistance callback.
    pub fn remove_resist_callback(&mut self) {
        if let Some(handle) = self.resist_handle.take() {
            if let Ok(lib) = sdk_lib() {
                unsafe { (lib.fn_remove_resist_cb_bb)(handle) };
            }
        }
    }

    // ── Battery callback ─────────────────────────────────────────────────

    /// Subscribe to battery level changes.
    pub fn on_battery<F>(&mut self, callback: F) -> Result<(), BrainBitError>
    where
        F: FnMut(i32) + Send + 'static,
    {
        self.remove_battery_callback();

        let lib = sdk_lib()?;
        let mut status = OpStatus::default();
        let boxed: Box<Box<dyn FnMut(i32) + Send>> = Box::new(Box::new(callback));
        let user_data = Box::into_raw(boxed) as *mut std::ffi::c_void;
        self.user_data_ptrs.push(user_data);

        let mut handle: BattPowerListenerHandle = ptr::null_mut();
        unsafe {
            (lib.fn_add_battery_cb)(
                self.ptr,
                battery_trampoline,
                &mut handle,
                user_data,
                &mut status,
            );
        }
        check_status(&status)?;
        self.battery_handle = Some(handle);
        Ok(())
    }

    /// Remove the battery callback.
    pub fn remove_battery_callback(&mut self) {
        if let Some(handle) = self.battery_handle.take() {
            if let Ok(lib) = sdk_lib() {
                unsafe { (lib.fn_remove_battery_cb)(handle) };
            }
        }
    }

    // ── Connection state callback ────────────────────────────────────────

    /// Subscribe to connection state changes.
    pub fn on_connection_state<F>(&mut self, callback: F) -> Result<(), BrainBitError>
    where
        F: FnMut(SensorState) + Send + 'static,
    {
        self.remove_connection_state_callback();

        let lib = sdk_lib()?;
        let mut status = OpStatus::default();
        let boxed: Box<Box<dyn FnMut(SensorState) + Send>> = Box::new(Box::new(callback));
        let user_data = Box::into_raw(boxed) as *mut std::ffi::c_void;
        self.user_data_ptrs.push(user_data);

        let mut handle: SensorStateListenerHandle = ptr::null_mut();
        unsafe {
            (lib.fn_add_connection_state_cb)(
                self.ptr,
                connection_state_trampoline,
                &mut handle,
                user_data,
                &mut status,
            );
        }
        check_status(&status)?;
        self.state_handle = Some(handle);
        Ok(())
    }

    /// Remove the connection state callback.
    pub fn remove_connection_state_callback(&mut self) {
        if let Some(handle) = self.state_handle.take() {
            if let Ok(lib) = sdk_lib() {
                unsafe { (lib.fn_remove_connection_state_cb)(handle) };
            }
        }
    }

    // ── BrainBit2 specific ───────────────────────────────────────────────

    /// Read the supported EEG channels (BrainBit2 / Flex / Pro).
    pub fn supported_channels_bb2(&self) -> Result<Vec<EEGChannelInfo>, BrainBitError> {
        let lib = sdk_lib()?;
        let mut channels = vec![
            EEGChannelInfo {
                id: EEGChannelId::Unknown,
                ch_type: EEGChannelType::SingleA1,
                name: [0u8; SENSOR_CHANNEL_NAME_LEN],
                num: 0,
            };
            BRAINBIT2_MAX_CH_COUNT
        ];
        let mut sz = BRAINBIT2_MAX_CH_COUNT as i32;
        let mut status = OpStatus::default();
        unsafe {
            (lib.fn_read_supported_channels_bb2)(
                self.ptr,
                channels.as_mut_ptr(),
                &mut sz,
                &mut status,
            );
        }
        check_status(&status)?;
        channels.truncate(sz.max(0) as usize);
        Ok(channels)
    }

    /// Read BrainBit2 amplifier parameters.
    pub fn amplifier_param_bb2(&self) -> Result<BrainBit2AmplifierParam, BrainBitError> {
        let lib = sdk_lib()?;
        let mut param = unsafe { std::mem::zeroed::<BrainBit2AmplifierParam>() };
        let mut status = OpStatus::default();
        unsafe { (lib.fn_read_amplifier_param_bb2)(self.ptr, &mut param, &mut status) };
        check_status(&status)?;
        Ok(param)
    }

    /// Write BrainBit2 amplifier parameters.
    pub fn set_amplifier_param_bb2(&self, param: BrainBit2AmplifierParam) -> Result<(), BrainBitError> {
        let lib = sdk_lib()?;
        let mut status = OpStatus::default();
        unsafe { (lib.fn_write_amplifier_param_bb2)(self.ptr, param, &mut status) };
        check_status(&status)
    }

    // ── Convenience: blocking signal capture ─────────────────────────────

    /// Capture `n_samples` of EEG data (original BrainBit) and return them.
    ///
    /// This is a blocking convenience method. For real-time streaming,
    /// use [`on_signal`](Self::on_signal) instead.
    pub fn capture_signal(&mut self, n_samples: usize) -> Result<Vec<EegSample>, BrainBitError> {
        let collected = Arc::new(Mutex::new(Vec::with_capacity(n_samples)));
        let collected2 = collected.clone();
        let target = n_samples;

        self.on_signal(move |samples| {
            let mut buf = collected2.lock().unwrap();
            if buf.len() < target {
                buf.extend_from_slice(samples);
            }
        })?;

        self.start_signal()?;

        loop {
            std::thread::sleep(std::time::Duration::from_millis(10));
            let len = collected.lock().unwrap().len();
            if len >= n_samples {
                break;
            }
        }

        self.stop_signal()?;
        self.remove_signal_callback();

        let mut result = collected.lock().unwrap().clone();
        result.truncate(n_samples);
        Ok(result)
    }
}

impl Drop for BrainBitDevice {
    fn drop(&mut self) {
        // Remove all callbacks first
        self.remove_signal_callback();
        self.remove_resist_callback();
        self.remove_battery_callback();
        self.remove_connection_state_callback();

        if let Some(handle) = self.signal_handle_bb2.take() {
            if let Ok(lib) = sdk_lib() {
                unsafe { (lib.fn_remove_signal_cb_bb2)(handle) };
            }
        }
        if let Some(handle) = self.resist_handle_bb2.take() {
            if let Ok(lib) = sdk_lib() {
                unsafe { (lib.fn_remove_resist_cb_bb2)(handle) };
            }
        }
        if let Some(handle) = self.mems_handle.take() {
            if let Ok(lib) = sdk_lib() {
                unsafe { (lib.fn_remove_mems_cb)(handle) };
            }
        }

        // Free the sensor
        if let Ok(lib) = sdk_lib() {
            unsafe { (lib.fn_free_sensor)(self.ptr) };
        }
    }
}

// ── Trampolines ──────────────────────────────────────────────────────────────

unsafe extern "C" fn bb_signal_trampoline(
    _sensor: *mut Sensor,
    data: *mut BrainBitSignalData,
    count: i32,
    user_data: *mut std::ffi::c_void,
) {
    if user_data.is_null() || data.is_null() || count <= 0 {
        return;
    }
    let closure = &mut *(user_data as *mut Box<dyn FnMut(&[EegSample]) + Send>);
    let raw_slice = std::slice::from_raw_parts(data, count as usize);
    let samples: Vec<EegSample> = raw_slice.iter().map(EegSample::from).collect();
    closure(&samples);
}

unsafe extern "C" fn bb_resist_trampoline(
    _sensor: *mut Sensor,
    data: BrainBitResistData,
    user_data: *mut std::ffi::c_void,
) {
    if user_data.is_null() {
        return;
    }
    let closure = &mut *(user_data as *mut Box<dyn FnMut(ResistanceSample) + Send>);
    closure(ResistanceSample::from(&data));
}

unsafe extern "C" fn battery_trampoline(
    _sensor: *mut Sensor,
    level: i32,
    user_data: *mut std::ffi::c_void,
) {
    if user_data.is_null() {
        return;
    }
    let closure = &mut *(user_data as *mut Box<dyn FnMut(i32) + Send>);
    closure(level);
}

unsafe extern "C" fn connection_state_trampoline(
    _sensor: *mut Sensor,
    state: SensorState,
    user_data: *mut std::ffi::c_void,
) {
    if user_data.is_null() {
        return;
    }
    let closure = &mut *(user_data as *mut Box<dyn FnMut(SensorState) + Send>);
    closure(state);
}
