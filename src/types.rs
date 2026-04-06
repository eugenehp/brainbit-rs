//! FFI types matching `cmn_type.h` from the NeuroSDK2 C API.
//!
//! These are `#[repr(C)]` structs and enums that are ABI-compatible with
//! the `neurosdk2` shared library on Windows, Linux, and macOS.

#![allow(non_camel_case_types)]

use std::ffi::c_void;
use std::fmt;

// ── Constants ────────────────────────────────────────────────────────────────

pub const ERR_MSG_LEN: usize = 512;
pub const SENSOR_NAME_LEN: usize = 256;
pub const SENSOR_ADR_LEN: usize = 128;
pub const SENSOR_SN_LEN: usize = 128;
pub const SENSOR_CHANNEL_NAME_LEN: usize = 8;
pub const NEURO_EEG_MAX_CH_COUNT: usize = 24;
pub const BRAINBIT2_MAX_CH_COUNT: usize = 8;

// ── Opaque handles ───────────────────────────────────────────────────────────

/// Opaque scanner handle (pointer to C struct).
pub type SensorScanner = c_void;
/// Opaque sensor/device handle (pointer to C struct).
pub type Sensor = c_void;

// Callback listener handles (all opaque pointers).
pub type SensorsListenerHandle = *mut c_void;
pub type BattPowerListenerHandle = *mut c_void;
pub type BattVoltageListenerHandle = *mut c_void;
pub type SensorStateListenerHandle = *mut c_void;
pub type BrainBitSignalDataListenerHandle = *mut c_void;
pub type BrainBitResistDataListenerHandle = *mut c_void;
pub type BrainBit2SignalDataListenerHandle = *mut c_void;
pub type BrainBit2ResistDataListenerHandle = *mut c_void;
pub type MEMSDataListenerHandle = *mut c_void;
pub type CallibriSignalDataListenerHandle = *mut c_void;
pub type CallibriRespirationDataListenerHandle = *mut c_void;
pub type CallibriElectrodeStateListenerHandle = *mut c_void;
pub type CallibriEnvelopeDataListenerHandle = *mut c_void;
pub type QuaternionDataListenerHandle = *mut c_void;
pub type FPGDataListenerHandle = *mut c_void;
pub type HeadphonesSignalDataListenerHandle = *mut c_void;
pub type HeadphonesResistDataListenerHandle = *mut c_void;
pub type Headphones2SignalDataListenerHandle = *mut c_void;
pub type Headphones2ResistDataListenerHandle = *mut c_void;
pub type AmpModeListenerHandle = *mut c_void;
pub type HeadbandSignalDataListenerHandle = *mut c_void;
pub type HeadbandResistDataListenerHandle = *mut c_void;
pub type NeuroEEGSignalDataListenerHandle = *mut c_void;
pub type NeuroEEGResistDataListenerHandle = *mut c_void;
pub type NeuroEEGSignalResistDataListenerHandle = *mut c_void;
pub type NeuroEEGSignalRawDataListenerHandle = *mut c_void;
pub type NeuroEEGFileStreamDataListenerHandle = *mut c_void;
pub type NeuroEEGSignalProcessParam = *mut c_void;
pub type StimulModeListenerHandle = *mut c_void;
pub type PhotoStimulSyncStateListenerHandle = *mut c_void;

// ── OpStatus ─────────────────────────────────────────────────────────────────

/// Status returned by most SDK functions.
#[repr(C)]
#[derive(Clone)]
pub struct OpStatus {
    pub success: u8,
    pub error: u32,
    pub error_msg: [u8; ERR_MSG_LEN],
}

impl Default for OpStatus {
    fn default() -> Self {
        Self {
            success: 0,
            error: 0,
            error_msg: [0u8; ERR_MSG_LEN],
        }
    }
}

impl OpStatus {
    /// Returns `true` if the operation succeeded.
    pub fn is_ok(&self) -> bool {
        self.success != 0
    }

    /// Extract the error message as a Rust string.
    pub fn message(&self) -> String {
        let nul = self.error_msg.iter().position(|&b| b == 0).unwrap_or(ERR_MSG_LEN);
        String::from_utf8_lossy(&self.error_msg[..nul]).into_owned()
    }
}

impl fmt::Debug for OpStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OpStatus")
            .field("success", &self.success)
            .field("error", &self.error)
            .field("error_msg", &self.message())
            .finish()
    }
}

// ── Enums ────────────────────────────────────────────────────────────────────

/// Device family used for scanner filtering and identification.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SensorFamily {
    Unknown = 0,
    LECallibri = 1,
    LEKolibri = 2,
    LEBrainBit = 3,
    LEBrainBitBlack = 4,
    LEHeadPhones = 5,
    LEHeadPhones2 = 6,
    LEHeadband = 11,
    LENeuroEEG = 14,
    LEBrainBit2 = 18,
    LEBrainBitPro = 19,
    LEBrainBitFlex = 20,
    LEPhotoStim = 21,
}

/// Sensor feature capabilities.
#[repr(i8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensorFeature {
    Signal = 0,
    MEMS = 1,
    CurrentStimulator = 2,
    Respiration = 3,
    Resist = 4,
    FPG = 5,
    Envelope = 6,
    PhotoStimulator = 7,
    AcousticStimulator = 8,
    FlashCard = 9,
    LedChannels = 10,
    SignalWithResist = 11,
}

/// Firmware mode.
#[repr(i8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensorFirmwareMode {
    Bootloader = 0,
    Application = 1,
}

/// Commands that can be sent to a sensor.
#[repr(i8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensorCommand {
    StartSignal = 0,
    StopSignal = 1,
    StartResist = 2,
    StopResist = 3,
    StartMEMS = 4,
    StopMEMS = 5,
    StartRespiration = 6,
    StopRespiration = 7,
    StartCurrentStimulation = 8,
    StopCurrentStimulation = 9,
    EnableMotionAssistant = 10,
    DisableMotionAssistant = 11,
    FindMe = 12,
    StartAngle = 13,
    StopAngle = 14,
    CalibrateMEMS = 15,
    ResetQuaternion = 16,
    StartEnvelope = 17,
    StopEnvelope = 18,
    ResetMotionCounter = 19,
    CalibrateStimulation = 20,
    Idle = 21,
    PowerDown = 22,
    StartFPG = 23,
    StopFPG = 24,
    StartSignalAndResist = 25,
    StopSignalAndResist = 26,
    StartPhotoStimulation = 27,
    StopPhotoStimulation = 28,
    StartAcousticStimulation = 29,
    StopAcousticStimulation = 30,
    FileSystemEnable = 31,
    FileSystemDisable = 32,
    FileSystemStreamClose = 33,
    StartCalibrateSignal = 34,
    StopCalibrateSignal = 35,
    PhotoStimEnable = 36,
    PhotoStimDisable = 37,
}

/// Sensor parameter identifiers.
#[repr(i8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensorParameter {
    Name = 0,
    State = 1,
    Address = 2,
    SerialNumber = 3,
    HardwareFilterState = 4,
    FirmwareMode = 5,
    SamplingFrequency = 6,
    Gain = 7,
    Offset = 8,
    ExternalSwitchState = 9,
    ADCInputState = 10,
    AccelerometerSens = 11,
    GyroscopeSens = 12,
    StimulatorAndMAState = 13,
    StimulatorParamPack = 14,
    MotionAssistantParamPack = 15,
    FirmwareVersion = 16,
    MEMSCalibrationStatus = 17,
    MotionCounterParamPack = 18,
    MotionCounter = 19,
    BattPower = 20,
    SensorFamilyParam = 21,
    SensorMode = 22,
    IrAmplitude = 23,
    RedAmplitude = 24,
    EnvelopeAvgWndSz = 25,
    EnvelopeDecimation = 26,
    SamplingFrequencyResist = 27,
    SamplingFrequencyMEMS = 28,
    SamplingFrequencyFPG = 29,
    Amplifier = 30,
    SensorChannels = 31,
    SamplingFrequencyResp = 32,
    SurveyId = 33,
    FileSystemStatus = 34,
    FileSystemDiskInfo = 35,
    ReferentsShort = 36,
    ReferentsGround = 37,
    SamplingFrequencyEnvelope = 38,
    ChannelConfiguration = 39,
    ElectrodeState = 40,
    ChannelResistConfiguration = 41,
    BattVoltage = 42,
    PhotoStimTimeDefer = 43,
    PhotoStimSyncState = 44,
    SensorPhotoStim = 45,
    StimMode = 46,
    LedChannels = 47,
    LedState = 48,
}

/// Parameter access mode.
#[repr(i8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensorParamAccess {
    Read = 0,
    ReadWrite = 1,
    ReadNotify = 2,
    Write = 3,
}

/// Connection state.
#[repr(i8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensorState {
    InRange = 0,
    OutOfRange = 1,
}

/// Sampling frequency presets.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensorSamplingFrequency {
    Hz10 = 0,
    Hz20 = 1,
    Hz100 = 2,
    Hz125 = 3,
    Hz250 = 4,
    Hz500 = 5,
    Hz1000 = 6,
    Hz2000 = 7,
    Hz4000 = 8,
    Hz8000 = 9,
    Hz10000 = 10,
    Hz12000 = 11,
    Hz16000 = 12,
    Hz24000 = 13,
    Hz32000 = 14,
    Hz48000 = 15,
    Hz64000 = 16,
    Unsupported = 0xFF,
}

impl SensorSamplingFrequency {
    /// Convert to the actual Hz value.
    pub fn to_hz(self) -> Option<u32> {
        match self {
            Self::Hz10 => Some(10),
            Self::Hz20 => Some(20),
            Self::Hz100 => Some(100),
            Self::Hz125 => Some(125),
            Self::Hz250 => Some(250),
            Self::Hz500 => Some(500),
            Self::Hz1000 => Some(1000),
            Self::Hz2000 => Some(2000),
            Self::Hz4000 => Some(4000),
            Self::Hz8000 => Some(8000),
            Self::Hz10000 => Some(10000),
            Self::Hz12000 => Some(12000),
            Self::Hz16000 => Some(16000),
            Self::Hz24000 => Some(24000),
            Self::Hz32000 => Some(32000),
            Self::Hz48000 => Some(48000),
            Self::Hz64000 => Some(64000),
            Self::Unsupported => None,
        }
    }
}

/// Signal gain settings.
#[repr(i8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensorGain {
    Gain1 = 0,
    Gain2 = 1,
    Gain3 = 2,
    Gain4 = 3,
    Gain6 = 4,
    Gain8 = 5,
    Gain12 = 6,
    Gain24 = 7,
    Gain5 = 8,
    Gain2x = 9,
    Gain4x = 10,
    Unsupported = 11,
}

/// Data offset settings.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensorDataOffset {
    Offset0 = 0x00,
    Offset1 = 0x01,
    Offset2 = 0x02,
    Offset3 = 0x03,
    Offset4 = 0x04,
    Offset5 = 0x05,
    Offset6 = 0x06,
    Offset7 = 0x07,
    Offset8 = 0x08,
    Unsupported = 0xFF,
}

/// Hardware filter presets.
#[repr(u16)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensorFilter {
    HPFBwhLvl1CutoffFreq1Hz = 0,
    HPFBwhLvl1CutoffFreq5Hz = 1,
    BSFBwhLvl2CutoffFreq45_55Hz = 2,
    BSFBwhLvl2CutoffFreq55_65Hz = 3,
    HPFBwhLvl2CutoffFreq10Hz = 4,
    LPFBwhLvl2CutoffFreq400Hz = 5,
    HPFBwhLvl2CutoffFreq80Hz = 6,
    Unknown = 0xFF,
}

/// EEG channel identifiers (10-20 system).
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EEGChannelId {
    Unknown = 0,
    O1 = 1,
    P3 = 2,
    C3 = 3,
    F3 = 4,
    Fp1 = 5,
    T5 = 6,
    T3 = 7,
    F7 = 8,
    F8 = 9,
    T4 = 10,
    T6 = 11,
    Fp2 = 12,
    F4 = 13,
    C4 = 14,
    P4 = 15,
    O2 = 16,
    D1 = 17,
    D2 = 18,
    OZ = 19,
    PZ = 20,
    CZ = 21,
    FZ = 22,
    FpZ = 23,
    D3 = 24,
    Ref = 25,
    A1 = 26,
    A2 = 27,
    Gnd1 = 28,
    Gnd2 = 29,
}

/// EEG channel type.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EEGChannelType {
    SingleA1 = 0,
    SingleA2 = 1,
    Differential = 2,
    Ref = 3,
}

/// BrainBit2 channel mode.
#[repr(i8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrainBit2ChannelMode {
    Short = 0,
    Normal = 1,
}

/// Generator current.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GenCurrent {
    GenCurr0nA = 0,
    GenCurr6nA = 1,
    GenCurr12nA = 2,
    GenCurr18nA = 3,
    GenCurr24nA = 4,
    GenCurr6uA = 5,
    GenCurr24uA = 6,
    Unsupported = 0xFF,
}

/// Accelerometer sensitivity.
#[repr(i8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensorAccelerometerSensitivity {
    Sens2g = 0,
    Sens4g = 1,
    Sens8g = 2,
    Sens16g = 3,
    Unsupported = 4,
}

/// Gyroscope sensitivity.
#[repr(i8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensorGyroscopeSensitivity {
    Sens250Grad = 0,
    Sens500Grad = 1,
    Sens1000Grad = 2,
    Sens2000Grad = 3,
    Unsupported = 4,
}

// ── Structs ──────────────────────────────────────────────────────────────────

/// Firmware/hardware version information.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SensorVersion {
    pub fw_major: u32,
    pub fw_minor: u32,
    pub fw_patch: u32,
    pub hw_major: u32,
    pub hw_minor: u32,
    pub hw_patch: u32,
    pub ext_major: u32,
}

/// Discovered device information (from scanner).
#[repr(C)]
#[derive(Clone)]
pub struct SensorInfo {
    pub sens_family: SensorFamily,
    pub sens_model: u8,
    pub name: [u8; SENSOR_NAME_LEN],
    pub address: [u8; SENSOR_ADR_LEN],
    pub serial_number: [u8; SENSOR_SN_LEN],
    pub pairing_required: u8,
    pub rssi: i16,
}

impl Default for SensorInfo {
    fn default() -> Self {
        Self {
            sens_family: SensorFamily::Unknown,
            sens_model: 0,
            name: [0u8; SENSOR_NAME_LEN],
            address: [0u8; SENSOR_ADR_LEN],
            serial_number: [0u8; SENSOR_SN_LEN],
            pairing_required: 0,
            rssi: 0,
        }
    }
}

impl SensorInfo {
    fn buf_to_string(buf: &[u8]) -> String {
        let nul = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
        String::from_utf8_lossy(&buf[..nul]).into_owned()
    }

    /// Device name as a Rust string.
    pub fn name_str(&self) -> String {
        Self::buf_to_string(&self.name)
    }

    /// Device BLE address as a Rust string.
    pub fn address_str(&self) -> String {
        Self::buf_to_string(&self.address)
    }

    /// Device serial number as a Rust string.
    pub fn serial_number_str(&self) -> String {
        Self::buf_to_string(&self.serial_number)
    }
}

impl fmt::Debug for SensorInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SensorInfo")
            .field("family", &self.sens_family)
            .field("model", &self.sens_model)
            .field("name", &self.name_str())
            .field("address", &self.address_str())
            .field("serial_number", &self.serial_number_str())
            .field("pairing_required", &self.pairing_required)
            .field("rssi", &self.rssi)
            .finish()
    }
}

/// Parameter information.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ParameterInfo {
    pub param: SensorParameter,
    pub param_access: SensorParamAccess,
}

/// BrainBit (original) 4-channel EEG signal sample.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct BrainBitSignalData {
    pub pack_num: u32,
    pub marker: u8,
    pub o1: f64,
    pub o2: f64,
    pub t3: f64,
    pub t4: f64,
}

/// BrainBit (original) 4-channel resistance data.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct BrainBitResistData {
    pub o1: f64,
    pub o2: f64,
    pub t3: f64,
    pub t4: f64,
}

/// Multi-channel signal data (BrainBit2, NeuroEEG, etc.).
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SignalChannelsData {
    pub pack_num: u32,
    pub marker: u8,
    pub sz_samples: u32,
    pub samples: *mut f64,
}

/// Multi-channel resistance data with referents.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ResistRefChannelsData {
    pub pack_num: u32,
    pub sz_samples: u32,
    pub sz_referents: u32,
    pub samples: *mut f64,
    pub referents: *mut f64,
}

/// EEG channel information.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct EEGChannelInfo {
    pub id: EEGChannelId,
    pub ch_type: EEGChannelType,
    pub name: [u8; SENSOR_CHANNEL_NAME_LEN],
    pub num: u8,
}

impl EEGChannelInfo {
    /// Channel name as a Rust string.
    pub fn name_str(&self) -> String {
        let nul = self.name.iter().position(|&b| b == 0).unwrap_or(SENSOR_CHANNEL_NAME_LEN);
        String::from_utf8_lossy(&self.name[..nul]).into_owned()
    }
}

impl fmt::Debug for EEGChannelInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("EEGChannelInfo")
            .field("id", &self.id)
            .field("ch_type", &self.ch_type)
            .field("name", &self.name_str())
            .field("num", &self.num)
            .finish()
    }
}

/// BrainBit2 amplifier parameters.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct BrainBit2AmplifierParam {
    pub ch_signal_mode: [BrainBit2ChannelMode; BRAINBIT2_MAX_CH_COUNT],
    pub ch_resist_use: [u8; BRAINBIT2_MAX_CH_COUNT],
    pub ch_gain: [SensorGain; BRAINBIT2_MAX_CH_COUNT],
    pub current: GenCurrent,
}

/// MEMS (accelerometer + gyroscope) data sample.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MEMSData {
    pub pack_num: u32,
    pub accelerometer_x: f64,
    pub accelerometer_y: f64,
    pub accelerometer_z: f64,
    pub gyroscope_x: f64,
    pub gyroscope_y: f64,
    pub gyroscope_z: f64,
}

// ── Callibri types ───────────────────────────────────────────────────────────

/// Callibri device colour.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallibriColorType {
    Red = 0,
    Yellow = 1,
    Blue = 2,
    White = 3,
    Unknown = 4,
}

/// Callibri electrode state.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallibriElectrodeState {
    Normal = 0,
    HighResistance = 1,
    Detached = 2,
}

/// External switch input.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensorExternalSwitchInput {
    ElectrodesRespUSB = 0,
    Electrodes = 1,
    USB = 2,
    RespUSB = 3,
    Short = 4,
    Unknown = 0xFF,
}

/// ADC input mode.
#[repr(i8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensorADCInput {
    Electrodes = 0,
    Short = 1,
    Test = 2,
    Resistance = 3,
}

/// Callibri stimulator state.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallibriStimulatorState {
    NoParams = 0,
    Disabled = 1,
    Enabled = 2,
    Unsupported = 0xFF,
}

/// Callibri stimulator + motion assistant state.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CallibriStimulatorMAState {
    pub stimulator_state: CallibriStimulatorState,
    pub ma_state: CallibriStimulatorState,
}

/// Callibri stimulation parameters.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CallibriStimulationParams {
    /// Stimulus amplitude in mA (1..100).
    pub current: u8,
    /// Duration of the stimulating pulse in µs (20..460).
    pub pulse_width: u16,
    /// Frequency of stimulation impulses in Hz (1..200).
    pub frequency: u8,
    /// Maximum stimulation time in ms (0..65535, 0 = infinite).
    pub stimulus_duration: u16,
}

/// Callibri motion assistant limb.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CallibriMotionAssistantLimb {
    RightLeg = 0,
    LeftLeg = 1,
    RightArm = 2,
    LeftArm = 3,
    Unsupported = 0xFF,
}

/// Callibri motion assistant parameters.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CallibriMotionAssistantParams {
    pub gyro_start: u8,
    pub gyro_stop: u8,
    pub limb: CallibriMotionAssistantLimb,
    /// Multiple of 10.
    pub min_pause_ms: u8,
}

/// Callibri motion counter parameters.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CallibriMotionCounterParam {
    /// Insensitivity threshold in mg (0..500).
    pub insense_threshold_mg: u16,
    /// Algorithm insensitivity threshold in samples (0..500).
    pub insense_threshold_sample: u16,
}

/// Callibri signal data.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CallibriSignalData {
    pub pack_num: u32,
    pub samples: *mut f64,
    pub sz_samples: u32,
}

/// Callibri respiration data.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CallibriRespirationData {
    pub pack_num: u32,
    pub samples: *mut f64,
    pub sz_samples: u32,
}

/// Callibri envelope data.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct CallibriEnvelopeData {
    pub pack_num: u32,
    pub sample: f64,
}

/// Callibri signal type.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalTypeCallibri {
    EEG = 0,
    EMG = 1,
    ECG = 2,
    EDA = 3,
    StrainGaugeBreathing = 4,
    ImpedanceBreathing = 5,
    TenzoBreathing = 6,
    Unknown = 7,
}

/// Quaternion data (MEMS orientation).
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct QuaternionData {
    pub pack_num: u32,
    pub w: f32,
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

// ── FPG types ────────────────────────────────────────────────────────────────

/// IR amplitude for FPG.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IrAmplitude {
    Amp0 = 0,
    Amp14 = 1,
    Amp28 = 2,
    Amp42 = 3,
    Amp56 = 4,
    Amp70 = 5,
    Amp84 = 6,
    Amp100 = 7,
    Unsupported = 0xFF,
}

/// Red amplitude for FPG.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedAmplitude {
    Amp0 = 0,
    Amp14 = 1,
    Amp28 = 2,
    Amp42 = 3,
    Amp56 = 4,
    Amp70 = 5,
    Amp84 = 6,
    Amp100 = 7,
    Unsupported = 0xFF,
}

/// FPG (photoplethysmography) data sample.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct FPGData {
    pub pack_num: u32,
    pub ir_amplitude: f64,
    pub red_amplitude: f64,
}

// ── Headphones types ─────────────────────────────────────────────────────────

/// Headphones (v1) 7-channel signal data.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct HeadphonesSignalData {
    pub pack_num: u32,
    pub marker: u8,
    pub ch1: f64,
    pub ch2: f64,
    pub ch3: f64,
    pub ch4: f64,
    pub ch5: f64,
    pub ch6: f64,
    pub ch7: f64,
}

/// Headphones (v1) 7-channel resistance data.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct HeadphonesResistData {
    pub pack_num: u32,
    pub ch1: f64,
    pub ch2: f64,
    pub ch3: f64,
    pub ch4: f64,
    pub ch5: f64,
    pub ch6: f64,
    pub ch7: f64,
}

/// Headphones (v1) amplifier parameters.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct HeadphonesAmplifierParam {
    pub ch_signal_use: [u8; 7],
    pub ch_resist_use: [u8; 7],
    pub ch_gain: [SensorGain; 7],
    pub current: GenCurrent,
}

/// Headphones2 4-channel signal data.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Headphones2SignalData {
    pub pack_num: u32,
    pub marker: u8,
    pub ch1: f64,
    pub ch2: f64,
    pub ch3: f64,
    pub ch4: f64,
}

/// Headphones2 4-channel resistance data.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Headphones2ResistData {
    pub pack_num: u32,
    pub ch1: f64,
    pub ch2: f64,
    pub ch3: f64,
    pub ch4: f64,
}

/// Headphones2 amplifier parameters.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Headphones2AmplifierParam {
    pub ch_signal_use: [u8; 4],
    pub ch_resist_use: [u8; 4],
    pub ch_gain: [SensorGain; 4],
    pub current: GenCurrent,
}

// ── Headband types ───────────────────────────────────────────────────────────

/// Headband 4-channel signal data.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct HeadbandSignalData {
    pub pack_num: u32,
    pub marker: u8,
    pub o1: f64,
    pub o2: f64,
    pub t3: f64,
    pub t4: f64,
}

/// Headband 4-channel resistance data.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct HeadbandResistData {
    pub pack_num: u32,
    pub o1: f64,
    pub o2: f64,
    pub t3: f64,
    pub t4: f64,
}

// ── AmpMode ──────────────────────────────────────────────────────────────────

/// Amplifier mode.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensorAmpMode {
    Invalid = 0,
    PowerDown = 1,
    Idle = 2,
    Signal = 3,
    Resist = 4,
    SignalResist = 5,
    Envelope = 6,
}

// ── SmartBand types ──────────────────────────────────────────────────────────

pub const SMART_BAND_MAX_CH_COUNT: usize = 4;

/// SmartBand amplifier parameters.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SmartBandAmplifierParam {
    pub ch_signal_use: [u8; SMART_BAND_MAX_CH_COUNT],
    pub ch_resist_use: [u8; SMART_BAND_MAX_CH_COUNT],
    pub ch_gain: [SensorGain; SMART_BAND_MAX_CH_COUNT],
    pub current: GenCurrent,
}

// ── NeuroEEG types ───────────────────────────────────────────────────────────

/// EEG channel mode (NeuroEEG).
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EEGChannelMode {
    Off = 0,
    Shorted = 1,
    SignalResist = 2,
    Signal = 3,
    Test = 4,
}

/// EEG reference mode (NeuroEEG).
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EEGRefMode {
    HeadTop = 1,
    A1A2 = 2,
}

/// NeuroEEG amplifier parameters.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NeuroEEGAmplifierParam {
    pub referent_resist_measure_allow: u8,
    pub frequency: SensorSamplingFrequency,
    pub referent_mode: EEGRefMode,
    pub channel_mode: [EEGChannelMode; NEURO_EEG_MAX_CH_COUNT],
    pub channel_gain: [SensorGain; NEURO_EEG_MAX_CH_COUNT],
    pub respiration_on: u8,
}

/// NeuroEEG resistance data (with A1, A2, Bias).
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct ResistChannelsData {
    pub pack_num: u32,
    pub a1: f64,
    pub a2: f64,
    pub bias: f64,
    pub sz_values: u32,
    pub values: *mut f64,
}

// ── NeuroEEG filesystem types ────────────────────────────────────────────────

/// NeuroEEG filesystem status.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensorFSStatus {
    OK = 0,
    NoInit = 1,
    NoDisk = 2,
    Protect = 3,
}

/// NeuroEEG filesystem I/O status.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensorFSIOStatus {
    NoError = 0,
    IOError = 1,
    Timeout = 2,
}

/// NeuroEEG filesystem stream status.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensorFSStreamStatus {
    Closed = 0,
    Write = 1,
    Read = 2,
}

/// NeuroEEG filesystem composite status.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NeuroEEGFSStatus {
    pub status: SensorFSStatus,
    pub io_status: SensorFSIOStatus,
    pub stream_status: SensorFSStreamStatus,
    pub autosave_signal: u8,
}

pub const FILE_NAME_MAX_LEN: usize = 64;

/// File information on the NeuroEEG filesystem.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct SensorFileInfo {
    pub file_name: [u8; FILE_NAME_MAX_LEN],
    pub file_size: u32,
    pub modified_year: u16,
    pub modified_month: u8,
    pub modified_day_of_month: u8,
    pub modified_hour: u8,
    pub modified_min: u8,
    pub modified_sec: u8,
    pub attribute: u8,
}

impl fmt::Debug for SensorFileInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let nul = self.file_name.iter().position(|&b| b == 0).unwrap_or(FILE_NAME_MAX_LEN);
        f.debug_struct("SensorFileInfo")
            .field("file_name", &String::from_utf8_lossy(&self.file_name[..nul]))
            .field("file_size", &self.file_size)
            .finish()
    }
}

/// File data (for read/write operations).
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SensorFileData {
    pub offset_start: u32,
    pub data_amount: u32,
    pub sz_data: u32,
    pub data: *mut u8,
}

/// Disk information.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SensorDiskInfo {
    pub total_size: u64,
    pub free_size: u64,
}

// ── PhotoStim types ──────────────────────────────────────────────────────────

/// Stimulation phase parameters.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct StimulPhase {
    /// Stimulation frequency.
    pub frequency: f64,
    /// Stimulus power 0..100 %.
    pub power: f64,
    /// Duration of a single stimulation pulse.
    pub pulse: f64,
    /// Stimulation phase duration.
    pub stimul_duration: f64,
    /// Duration of pause after the stimulation phase.
    pub pause: f64,
    /// Filling frequency of the signal for acoustic stimulation.
    pub filling_frequency: f64,
}

/// Stimulation mode.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensorStimulMode {
    Invalid = 0,
    Stopped = 1,
    PendingSync = 2,
    Synchronized = 3,
    StimProgramRunning = 4,
    Error = 5,
}

/// Stimulation sync state.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SensorStimulSyncState {
    Normal = 0,
    TimeOut = 1,
}
