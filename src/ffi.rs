//! Runtime-loaded FFI bindings to the NeuroSDK2 C library.
//!
//! Loads `neurosdk2.dll` / `libneurosdk2.so` / `libneurosdk2.dylib` at runtime
//! via `libloading`.  No build-time C dependencies are required.
//!
//! **Every** exported function from `sdk_api.h` is bound here for full parity
//! with the C SDK.

use std::ffi::c_void;
use std::sync::OnceLock;

use crate::error::BrainBitError;
use crate::types::*;

// ── Type aliases for function pointer signatures ─────────────────────────────

// Scanner
type FnCreateScanner = unsafe extern "C" fn(*const SensorFamily, i32, *mut OpStatus) -> *mut SensorScanner;
type FnFreeScanner = unsafe extern "C" fn(*mut SensorScanner);
type FnStartScanner = unsafe extern "C" fn(*mut SensorScanner, *mut OpStatus, i32) -> u8;
type FnStopScanner = unsafe extern "C" fn(*mut SensorScanner, *mut OpStatus) -> u8;
type FnSensorsScanner = unsafe extern "C" fn(*mut SensorScanner, *mut SensorInfo, *mut i32, *mut OpStatus) -> u8;
type FnAddSensorsCallbackScanner = unsafe extern "C" fn(
    *mut SensorScanner,
    unsafe extern "C" fn(*mut SensorScanner, *mut SensorInfo, i32, *mut c_void),
    *mut SensorsListenerHandle,
    *mut c_void,
    *mut OpStatus,
) -> u8;
type FnRemoveSensorsCallbackScanner = unsafe extern "C" fn(SensorsListenerHandle);

// Sensor lifecycle
type FnCreateSensor = unsafe extern "C" fn(*mut SensorScanner, SensorInfo, *mut OpStatus) -> *mut Sensor;
type FnFreeSensor = unsafe extern "C" fn(*mut Sensor);
type FnConnectSensor = unsafe extern "C" fn(*mut Sensor, *mut OpStatus) -> u8;
type FnDisconnectSensor = unsafe extern "C" fn(*mut Sensor, *mut OpStatus) -> u8;

// Features / commands / parameters
type FnGetCountI32 = unsafe extern "C" fn(*mut Sensor) -> i32;
type FnGetFeatures = unsafe extern "C" fn(*mut Sensor, *mut SensorFeature, *mut i32, *mut OpStatus) -> u8;
type FnIsSupportedFeature = unsafe extern "C" fn(*mut Sensor, SensorFeature) -> i8;
type FnGetCommands = unsafe extern "C" fn(*mut Sensor, *mut SensorCommand, *mut i32, *mut OpStatus) -> u8;
type FnIsSupportedCommand = unsafe extern "C" fn(*mut Sensor, SensorCommand) -> i8;
type FnGetParameters = unsafe extern "C" fn(*mut Sensor, *mut ParameterInfo, *mut i32, *mut OpStatus) -> u8;
type FnIsSupportedParameter = unsafe extern "C" fn(*mut Sensor, SensorParameter) -> i8;
type FnExecCommand = unsafe extern "C" fn(*mut Sensor, SensorCommand, *mut OpStatus) -> u8;

// Read/write simple properties
type FnGetFamily = unsafe extern "C" fn(*mut Sensor) -> SensorFamily;
type FnReadString = unsafe extern "C" fn(*mut Sensor, *mut u8, i32, *mut OpStatus) -> u8;
type FnWriteString = unsafe extern "C" fn(*mut Sensor, *mut u8, i32, *mut OpStatus) -> u8;
type FnReadState = unsafe extern "C" fn(*mut Sensor, *mut SensorState, *mut OpStatus) -> u8;
type FnReadI32 = unsafe extern "C" fn(*mut Sensor, *mut i32, *mut OpStatus) -> u8;
type FnReadU32 = unsafe extern "C" fn(*mut Sensor, *mut u32, *mut OpStatus) -> u8;
type FnReadU8 = unsafe extern "C" fn(*mut Sensor, *mut u8, *mut OpStatus) -> u8;
type FnReadSamplingFreq = unsafe extern "C" fn(*mut Sensor, *mut SensorSamplingFrequency, *mut OpStatus) -> u8;
type FnReadGain = unsafe extern "C" fn(*mut Sensor, *mut SensorGain, *mut OpStatus) -> u8;
type FnWriteGain = unsafe extern "C" fn(*mut Sensor, SensorGain, *mut OpStatus) -> u8;
type FnReadDataOffset = unsafe extern "C" fn(*mut Sensor, *mut SensorDataOffset, *mut OpStatus) -> u8;
type FnWriteDataOffset = unsafe extern "C" fn(*mut Sensor, SensorDataOffset, *mut OpStatus) -> u8;
type FnReadFirmwareMode = unsafe extern "C" fn(*mut Sensor, *mut SensorFirmwareMode, *mut OpStatus) -> u8;
type FnWriteFirmwareMode = unsafe extern "C" fn(*mut Sensor, SensorFirmwareMode, *mut OpStatus) -> u8;
type FnReadVersion = unsafe extern "C" fn(*mut Sensor, *mut SensorVersion, *mut OpStatus) -> u8;
type FnWriteSamplingFreq = unsafe extern "C" fn(*mut Sensor, SensorSamplingFrequency, *mut OpStatus) -> u8;

// Hardware filters
type FnReadFilters = unsafe extern "C" fn(*mut Sensor, *mut SensorFilter, *mut i32, *mut OpStatus) -> u8;
type FnWriteFilters = unsafe extern "C" fn(*mut Sensor, *mut SensorFilter, i32, *mut OpStatus) -> u8;
type FnGetSupportedFilters = unsafe extern "C" fn(*mut Sensor, *mut SensorFilter, *mut i32, *mut OpStatus) -> u8;
type FnIsSupportedFilter = unsafe extern "C" fn(*mut Sensor, SensorFilter) -> i8;

// External switch / ADC input
type FnReadExtSwitch = unsafe extern "C" fn(*mut Sensor, *mut SensorExternalSwitchInput, *mut OpStatus) -> u8;
type FnWriteExtSwitch = unsafe extern "C" fn(*mut Sensor, SensorExternalSwitchInput, *mut OpStatus) -> u8;
type FnReadADCInput = unsafe extern "C" fn(*mut Sensor, *mut SensorADCInput, *mut OpStatus) -> u8;
type FnWriteADCInput = unsafe extern "C" fn(*mut Sensor, SensorADCInput, *mut OpStatus) -> u8;

// Accelerometer / Gyroscope
type FnReadAccSens = unsafe extern "C" fn(*mut Sensor, *mut SensorAccelerometerSensitivity, *mut OpStatus) -> u8;
type FnWriteAccSens = unsafe extern "C" fn(*mut Sensor, SensorAccelerometerSensitivity, *mut OpStatus) -> u8;
type FnReadGyroSens = unsafe extern "C" fn(*mut Sensor, *mut SensorGyroscopeSensitivity, *mut OpStatus) -> u8;
type FnWriteGyroSens = unsafe extern "C" fn(*mut Sensor, SensorGyroscopeSensitivity, *mut OpStatus) -> u8;

// Callibri-specific
type FnReadColor = unsafe extern "C" fn(*mut Sensor, *mut CallibriColorType, *mut OpStatus) -> u8;
type FnReadElectrodeState = unsafe extern "C" fn(*mut Sensor, *mut CallibriElectrodeState, *mut OpStatus) -> u8;
type FnReadColorInfo = unsafe extern "C" fn(SensorInfo, *mut CallibriColorType);
type FnReadStimMAState = unsafe extern "C" fn(*mut Sensor, *mut CallibriStimulatorMAState, *mut OpStatus) -> u8;
type FnReadStimParam = unsafe extern "C" fn(*mut Sensor, *mut CallibriStimulationParams, *mut OpStatus) -> u8;
type FnWriteStimParam = unsafe extern "C" fn(*mut Sensor, CallibriStimulationParams, *mut OpStatus) -> u8;
type FnReadMAParam = unsafe extern "C" fn(*mut Sensor, *mut CallibriMotionAssistantParams, *mut OpStatus) -> u8;
type FnWriteMAParam = unsafe extern "C" fn(*mut Sensor, CallibriMotionAssistantParams, *mut OpStatus) -> u8;
type FnReadMotionCounterParam = unsafe extern "C" fn(*mut Sensor, *mut CallibriMotionCounterParam, *mut OpStatus) -> u8;
type FnWriteMotionCounterParam = unsafe extern "C" fn(*mut Sensor, CallibriMotionCounterParam, *mut OpStatus) -> u8;
type FnSetSignalSettingsCallibri = unsafe extern "C" fn(*mut Sensor, SignalTypeCallibri, *mut OpStatus) -> u8;
type FnGetSignalSettingsCallibri = unsafe extern "C" fn(*mut Sensor, *mut SignalTypeCallibri, *mut OpStatus) -> u8;

// Callibri callbacks
type FnAddCallibriSignalCb = unsafe extern "C" fn(
    *mut Sensor,
    unsafe extern "C" fn(*mut Sensor, *mut CallibriSignalData, i32, *mut c_void),
    *mut CallibriSignalDataListenerHandle,
    *mut c_void, *mut OpStatus,
) -> u8;
type FnRemoveCallibriSignalCb = unsafe extern "C" fn(CallibriSignalDataListenerHandle);
type FnAddCallibriRespCb = unsafe extern "C" fn(
    *mut Sensor,
    unsafe extern "C" fn(*mut Sensor, *mut CallibriRespirationData, i32, *mut c_void),
    *mut CallibriRespirationDataListenerHandle,
    *mut c_void, *mut OpStatus,
) -> u8;
type FnRemoveCallibriRespCb = unsafe extern "C" fn(CallibriRespirationDataListenerHandle);
type FnAddCallibriElStateCb = unsafe extern "C" fn(
    *mut Sensor,
    unsafe extern "C" fn(*mut Sensor, CallibriElectrodeState, *mut c_void),
    *mut CallibriElectrodeStateListenerHandle,
    *mut c_void, *mut OpStatus,
) -> u8;
type FnRemoveCallibriElStateCb = unsafe extern "C" fn(CallibriElectrodeStateListenerHandle);
type FnAddCallibriEnvCb = unsafe extern "C" fn(
    *mut Sensor,
    unsafe extern "C" fn(*mut Sensor, *mut CallibriEnvelopeData, i32, *mut c_void),
    *mut CallibriEnvelopeDataListenerHandle,
    *mut c_void, *mut OpStatus,
) -> u8;
type FnRemoveCallibriEnvCb = unsafe extern "C" fn(CallibriEnvelopeDataListenerHandle);

// Quaternion
type FnAddQuaternionCb = unsafe extern "C" fn(
    *mut Sensor,
    unsafe extern "C" fn(*mut Sensor, *mut QuaternionData, i32, *mut c_void),
    *mut QuaternionDataListenerHandle,
    *mut c_void, *mut OpStatus,
) -> u8;
type FnRemoveQuaternionCb = unsafe extern "C" fn(QuaternionDataListenerHandle);

// FPG
type FnReadIrAmp = unsafe extern "C" fn(*mut Sensor, *mut IrAmplitude, *mut OpStatus) -> u8;
type FnWriteIrAmp = unsafe extern "C" fn(*mut Sensor, IrAmplitude, *mut OpStatus) -> u8;
type FnReadRedAmp = unsafe extern "C" fn(*mut Sensor, *mut RedAmplitude, *mut OpStatus) -> u8;
type FnWriteRedAmp = unsafe extern "C" fn(*mut Sensor, RedAmplitude, *mut OpStatus) -> u8;
type FnAddFPGCb = unsafe extern "C" fn(
    *mut Sensor,
    unsafe extern "C" fn(*mut Sensor, *mut FPGData, i32, *mut c_void),
    *mut FPGDataListenerHandle,
    *mut c_void, *mut OpStatus,
) -> u8;
type FnRemoveFPGCb = unsafe extern "C" fn(FPGDataListenerHandle);

// Headphones
type FnReadAmpHeadphones = unsafe extern "C" fn(*mut Sensor, *mut HeadphonesAmplifierParam, *mut OpStatus) -> u8;
type FnWriteAmpHeadphones = unsafe extern "C" fn(*mut Sensor, HeadphonesAmplifierParam, *mut OpStatus) -> u8;
type FnAddHpSignalCb = unsafe extern "C" fn(
    *mut Sensor,
    unsafe extern "C" fn(*mut Sensor, *mut HeadphonesSignalData, i32, *mut c_void),
    *mut HeadphonesSignalDataListenerHandle,
    *mut c_void, *mut OpStatus,
) -> u8;
type FnRemoveHpSignalCb = unsafe extern "C" fn(HeadphonesSignalDataListenerHandle);
type FnAddHpResistCb = unsafe extern "C" fn(
    *mut Sensor,
    unsafe extern "C" fn(*mut Sensor, *mut HeadphonesResistData, i32, *mut c_void),
    *mut HeadphonesResistDataListenerHandle,
    *mut c_void, *mut OpStatus,
) -> u8;
type FnRemoveHpResistCb = unsafe extern "C" fn(HeadphonesResistDataListenerHandle);

// Headphones2
type FnReadAmpHeadphones2 = unsafe extern "C" fn(*mut Sensor, *mut Headphones2AmplifierParam, *mut OpStatus) -> u8;
type FnWriteAmpHeadphones2 = unsafe extern "C" fn(*mut Sensor, Headphones2AmplifierParam, *mut OpStatus) -> u8;
type FnAddHp2SignalCb = unsafe extern "C" fn(
    *mut Sensor,
    unsafe extern "C" fn(*mut Sensor, *mut Headphones2SignalData, i32, *mut c_void),
    *mut Headphones2SignalDataListenerHandle,
    *mut c_void, *mut OpStatus,
) -> u8;
type FnRemoveHp2SignalCb = unsafe extern "C" fn(Headphones2SignalDataListenerHandle);
type FnAddHp2ResistCb = unsafe extern "C" fn(
    *mut Sensor,
    unsafe extern "C" fn(*mut Sensor, *mut Headphones2ResistData, i32, *mut c_void),
    *mut Headphones2ResistDataListenerHandle,
    *mut c_void, *mut OpStatus,
) -> u8;
type FnRemoveHp2ResistCb = unsafe extern "C" fn(Headphones2ResistDataListenerHandle);

// AmpMode
type FnReadAmpMode = unsafe extern "C" fn(*mut Sensor, *mut SensorAmpMode, *mut OpStatus) -> u8;
type FnAddAmpModeCb = unsafe extern "C" fn(
    *mut Sensor,
    unsafe extern "C" fn(*mut Sensor, SensorAmpMode, *mut c_void),
    *mut AmpModeListenerHandle,
    *mut c_void, *mut OpStatus,
) -> u8;
type FnRemoveAmpModeCb = unsafe extern "C" fn(AmpModeListenerHandle);

// Headband
type FnAddHeadbandSignalCb = unsafe extern "C" fn(
    *mut Sensor,
    unsafe extern "C" fn(*mut Sensor, *mut HeadbandSignalData, i32, *mut c_void),
    *mut HeadbandSignalDataListenerHandle,
    *mut c_void, *mut OpStatus,
) -> u8;
type FnRemoveHeadbandSignalCb = unsafe extern "C" fn(HeadbandSignalDataListenerHandle);
type FnAddHeadbandResistCb = unsafe extern "C" fn(
    *mut Sensor,
    unsafe extern "C" fn(*mut Sensor, HeadbandResistData, *mut c_void),
    *mut HeadbandResistDataListenerHandle,
    *mut c_void, *mut OpStatus,
) -> u8;
type FnRemoveHeadbandResistCb = unsafe extern "C" fn(HeadbandResistDataListenerHandle);

// BrainBit signal/resist callbacks
type FnAddBBSignalCb = unsafe extern "C" fn(
    *mut Sensor,
    unsafe extern "C" fn(*mut Sensor, *mut BrainBitSignalData, i32, *mut c_void),
    *mut BrainBitSignalDataListenerHandle,
    *mut c_void, *mut OpStatus,
) -> u8;
type FnRemoveBBSignalCb = unsafe extern "C" fn(BrainBitSignalDataListenerHandle);
type FnAddBBResistCb = unsafe extern "C" fn(
    *mut Sensor,
    unsafe extern "C" fn(*mut Sensor, BrainBitResistData, *mut c_void),
    *mut BrainBitResistDataListenerHandle,
    *mut c_void, *mut OpStatus,
) -> u8;
type FnRemoveBBResistCb = unsafe extern "C" fn(BrainBitResistDataListenerHandle);

// BrainBit2 signal/resist callbacks
type FnAddBB2SignalCb = unsafe extern "C" fn(
    *mut Sensor,
    unsafe extern "C" fn(*mut Sensor, *mut SignalChannelsData, i32, *mut c_void),
    *mut BrainBit2SignalDataListenerHandle,
    *mut c_void, *mut OpStatus,
) -> u8;
type FnRemoveBB2SignalCb = unsafe extern "C" fn(BrainBit2SignalDataListenerHandle);
type FnAddBB2ResistCb = unsafe extern "C" fn(
    *mut Sensor,
    unsafe extern "C" fn(*mut Sensor, *mut ResistRefChannelsData, i32, *mut c_void),
    *mut BrainBit2ResistDataListenerHandle,
    *mut c_void, *mut OpStatus,
) -> u8;
type FnRemoveBB2ResistCb = unsafe extern "C" fn(BrainBit2ResistDataListenerHandle);

// BrainBit2 amplifier + channels
type FnReadSupportedChannelsBB2 = unsafe extern "C" fn(*mut Sensor, *mut EEGChannelInfo, *mut i32, *mut OpStatus) -> u8;
type FnReadAmplifierParamBB2 = unsafe extern "C" fn(*mut Sensor, *mut BrainBit2AmplifierParam, *mut OpStatus) -> u8;
type FnWriteAmplifierParamBB2 = unsafe extern "C" fn(*mut Sensor, BrainBit2AmplifierParam, *mut OpStatus) -> u8;

// SmartBand
type FnReadAmpSmartBand = unsafe extern "C" fn(*mut Sensor, *mut SmartBandAmplifierParam, *mut OpStatus) -> u8;
type FnWriteAmpSmartBand = unsafe extern "C" fn(*mut Sensor, SmartBandAmplifierParam, *mut OpStatus) -> u8;

// NeuroEEG
type FnReadSupportedChannelsNeuroEEG = unsafe extern "C" fn(*mut Sensor, *mut EEGChannelInfo, *mut i32, *mut OpStatus) -> u8;
type FnReadAmpNeuroEEG = unsafe extern "C" fn(*mut Sensor, *mut NeuroEEGAmplifierParam, *mut OpStatus) -> u8;
type FnWriteAmpNeuroEEG = unsafe extern "C" fn(*mut Sensor, NeuroEEGAmplifierParam, *mut OpStatus) -> u8;
type FnAddNeuroEEGSignalCb = unsafe extern "C" fn(
    *mut Sensor,
    unsafe extern "C" fn(*mut Sensor, *mut SignalChannelsData, i32, *mut c_void),
    *mut NeuroEEGSignalDataListenerHandle,
    *mut c_void, *mut OpStatus,
) -> u8;
type FnRemoveNeuroEEGSignalCb = unsafe extern "C" fn(NeuroEEGSignalDataListenerHandle);
type FnAddNeuroEEGResistCb = unsafe extern "C" fn(
    *mut Sensor,
    unsafe extern "C" fn(*mut Sensor, *mut ResistChannelsData, i32, *mut c_void),
    *mut NeuroEEGResistDataListenerHandle,
    *mut c_void, *mut OpStatus,
) -> u8;
type FnRemoveNeuroEEGResistCb = unsafe extern "C" fn(NeuroEEGResistDataListenerHandle);
type FnAddNeuroEEGSignalResistCb = unsafe extern "C" fn(
    *mut Sensor,
    unsafe extern "C" fn(*mut Sensor, *mut SignalChannelsData, i32, *mut ResistChannelsData, i32, *mut c_void),
    *mut NeuroEEGSignalResistDataListenerHandle,
    *mut c_void, *mut OpStatus,
) -> u8;
type FnRemoveNeuroEEGSignalResistCb = unsafe extern "C" fn(NeuroEEGSignalResistDataListenerHandle);
type FnAddNeuroEEGSignalRawCb = unsafe extern "C" fn(
    *mut Sensor,
    unsafe extern "C" fn(*mut Sensor, *mut u8, i32, *mut c_void),
    *mut NeuroEEGSignalRawDataListenerHandle,
    *mut c_void, *mut OpStatus,
) -> u8;
type FnRemoveNeuroEEGSignalRawCb = unsafe extern "C" fn(NeuroEEGSignalRawDataListenerHandle);

// NeuroEEG filesystem
type FnReadFSStatusNeuroEEG = unsafe extern "C" fn(*mut Sensor, *mut NeuroEEGFSStatus, *mut OpStatus) -> u8;
type FnReadFSDiskInfoNeuroEEG = unsafe extern "C" fn(*mut Sensor, *mut SensorDiskInfo, *mut OpStatus) -> u8;
type FnReadFileInfoNeuroEEG = unsafe extern "C" fn(*mut Sensor, *const u8, *mut SensorFileInfo, *mut OpStatus) -> u8;
type FnReadFileInfoAllNeuroEEG = unsafe extern "C" fn(*mut Sensor, *mut SensorFileInfo, *mut u32, *mut OpStatus) -> u8;
type FnWriteFileNeuroEEG = unsafe extern "C" fn(*mut Sensor, *const u8, *mut u8, u32, u32, *mut OpStatus) -> u8;
type FnReadFileNeuroEEG = unsafe extern "C" fn(*mut Sensor, *const u8, *mut u8, *mut u32, u32, *mut OpStatus) -> u8;
type FnDeleteFileNeuroEEG = unsafe extern "C" fn(*mut Sensor, *const u8, *mut OpStatus) -> u8;
type FnDeleteAllFilesNeuroEEG = unsafe extern "C" fn(*mut Sensor, *const u8, *mut OpStatus) -> u8;
type FnReadFileCRC32NeuroEEG = unsafe extern "C" fn(*mut Sensor, *const u8, u32, u32, *mut u32, *mut OpStatus) -> u8;
type FnFileStreamAutosaveNeuroEEG = unsafe extern "C" fn(*mut Sensor, *const u8, *mut OpStatus) -> u8;
type FnFileStreamReadNeuroEEG = unsafe extern "C" fn(*mut Sensor, *const u8, u32, u32, *mut OpStatus) -> u8;
type FnAddFileStreamReadCbNeuroEEG = unsafe extern "C" fn(
    *mut Sensor,
    unsafe extern "C" fn(*mut Sensor, *mut SensorFileData, i32, *mut c_void),
    *mut NeuroEEGFileStreamDataListenerHandle,
    *mut c_void, *mut OpStatus,
) -> u8;
type FnRemoveFileStreamReadCbNeuroEEG = unsafe extern "C" fn(NeuroEEGFileStreamDataListenerHandle);

// NeuroEEG signal process
type FnCreateSignalProcessParam = unsafe extern "C" fn(NeuroEEGAmplifierParam, *mut NeuroEEGSignalProcessParam, *mut OpStatus) -> u8;
type FnRemoveSignalProcessParam = unsafe extern "C" fn(NeuroEEGSignalProcessParam);
type FnParseRawSignal = unsafe extern "C" fn(
    *mut u8, *mut u32, NeuroEEGSignalProcessParam,
    *mut SignalChannelsData, *mut u32,
    *mut ResistChannelsData, *mut u32,
    *mut OpStatus,
) -> u8;

// NeuroEEG survey + PhotoStim on NeuroEEG
type FnReadSurveyId = unsafe extern "C" fn(*mut Sensor, *mut u32, *mut OpStatus) -> u8;
type FnWriteSurveyId = unsafe extern "C" fn(*mut Sensor, u32, *mut OpStatus) -> u8;
type FnReadPhotoStimNeuroEEG = unsafe extern "C" fn(*mut Sensor) -> *mut Sensor;
type FnWritePhotoStimNeuroEEG = unsafe extern "C" fn(*mut Sensor, *mut Sensor, *mut OpStatus) -> u8;

// Supported EEG channels (generic)
type FnReadSupportedEEGChannels = unsafe extern "C" fn(*mut Sensor, *mut EEGChannelInfo, *mut i32, *mut OpStatus) -> u8;

// Battery callbacks
type FnAddBatteryCb = unsafe extern "C" fn(
    *mut Sensor,
    unsafe extern "C" fn(*mut Sensor, i32, *mut c_void),
    *mut BattPowerListenerHandle,
    *mut c_void, *mut OpStatus,
) -> u8;
type FnRemoveBatteryCb = unsafe extern "C" fn(BattPowerListenerHandle);
type FnAddBatteryVoltageCb = unsafe extern "C" fn(
    *mut Sensor,
    unsafe extern "C" fn(*mut Sensor, i32, *mut c_void),
    *mut BattVoltageListenerHandle,
    *mut c_void, *mut OpStatus,
) -> u8;
type FnRemoveBatteryVoltageCb = unsafe extern "C" fn(BattVoltageListenerHandle);

// Connection state callback
type FnAddConnStateCb = unsafe extern "C" fn(
    *mut Sensor,
    unsafe extern "C" fn(*mut Sensor, SensorState, *mut c_void),
    *mut SensorStateListenerHandle,
    *mut c_void, *mut OpStatus,
) -> u8;
type FnRemoveConnStateCb = unsafe extern "C" fn(SensorStateListenerHandle);

// MEMS
type FnAddMEMSCb = unsafe extern "C" fn(
    *mut Sensor,
    unsafe extern "C" fn(*mut Sensor, *mut MEMSData, i32, *mut c_void),
    *mut MEMSDataListenerHandle,
    *mut c_void, *mut OpStatus,
) -> u8;
type FnRemoveMEMSCb = unsafe extern "C" fn(MEMSDataListenerHandle);

// PhotoStim
type FnReadStimMode = unsafe extern "C" fn(*mut Sensor, *mut SensorStimulMode, *mut OpStatus) -> u8;
type FnReadStimPrograms = unsafe extern "C" fn(*mut Sensor, *mut StimulPhase, *mut i32, *mut OpStatus) -> u8;
type FnWriteStimPrograms = unsafe extern "C" fn(*mut Sensor, *mut StimulPhase, i32, *mut OpStatus) -> u8;
type FnAddStimModeCb = unsafe extern "C" fn(
    *mut Sensor,
    unsafe extern "C" fn(*mut Sensor, SensorStimulMode, *mut c_void),
    *mut StimulModeListenerHandle,
    *mut c_void, *mut OpStatus,
) -> u8;
type FnRemoveStimModeCb = unsafe extern "C" fn(StimulModeListenerHandle);
type FnReadPhotoStimSyncState = unsafe extern "C" fn(*mut Sensor, *mut SensorStimulSyncState, *mut OpStatus) -> u8;
type FnReadPhotoStimTimeDefer = unsafe extern "C" fn(*mut Sensor, *mut f64, *mut OpStatus) -> u8;
type FnWritePhotoStimTimeDefer = unsafe extern "C" fn(*mut Sensor, f64, *mut OpStatus) -> u8;
type FnAddPhotoStimSyncStateCb = unsafe extern "C" fn(
    *mut Sensor,
    unsafe extern "C" fn(*mut Sensor, SensorStimulSyncState, *mut c_void),
    *mut PhotoStimulSyncStateListenerHandle,
    *mut c_void, *mut OpStatus,
) -> u8;
type FnRemovePhotoStimSyncStateCb = unsafe extern "C" fn(PhotoStimulSyncStateListenerHandle);

// Ping / CRC / possible features
type FnPingNeuroSmart = unsafe extern "C" fn(*mut Sensor, u8, *mut OpStatus) -> u8;
type FnCalcCRC32 = unsafe extern "C" fn(*mut u8, u32, *mut u32);
type FnGetPossibleFeaturesCount = unsafe extern "C" fn(SensorFamily, u8) -> i32;
type FnGetPossibleFeatures = unsafe extern "C" fn(SensorFamily, u8, *mut SensorFeature, *mut i32, *mut OpStatus) -> u8;

// ── Library wrapper ──────────────────────────────────────────────────────────

/// Dynamically-loaded NeuroSDK2 library handle with **all** exported functions.
pub struct NeuroSdk2Lib {
    _lib: libloading::Library,

    // ── Scanner ──────────────────────────────────────────────────────────
    pub(crate) fn_create_scanner: FnCreateScanner,
    pub(crate) fn_free_scanner: FnFreeScanner,
    pub(crate) fn_start_scanner: FnStartScanner,
    pub(crate) fn_stop_scanner: FnStopScanner,
    pub(crate) fn_sensors_scanner: FnSensorsScanner,
    pub(crate) fn_add_sensors_callback: FnAddSensorsCallbackScanner,
    pub(crate) fn_remove_sensors_callback: FnRemoveSensorsCallbackScanner,

    // ── Sensor lifecycle ─────────────────────────────────────────────────
    pub(crate) fn_create_sensor: FnCreateSensor,
    pub(crate) fn_free_sensor: FnFreeSensor,
    pub(crate) fn_connect_sensor: FnConnectSensor,
    pub(crate) fn_disconnect_sensor: FnDisconnectSensor,

    // ── Features / commands / parameters ─────────────────────────────────
    pub(crate) fn_get_features_count: FnGetCountI32,
    pub(crate) fn_get_features: FnGetFeatures,
    pub(crate) fn_is_supported_feature: FnIsSupportedFeature,
    pub(crate) fn_get_commands_count: FnGetCountI32,
    pub(crate) fn_get_commands: FnGetCommands,
    pub(crate) fn_is_supported_command: FnIsSupportedCommand,
    pub(crate) fn_get_parameters_count: FnGetCountI32,
    pub(crate) fn_get_parameters: FnGetParameters,
    pub(crate) fn_is_supported_parameter: FnIsSupportedParameter,
    pub(crate) fn_get_channels_count: FnGetCountI32,
    pub(crate) fn_exec_command: FnExecCommand,

    // ── Properties ───────────────────────────────────────────────────────
    pub(crate) fn_get_family: FnGetFamily,
    pub(crate) fn_read_name: FnReadString,
    pub(crate) fn_write_name: FnWriteString,
    pub(crate) fn_read_state: FnReadState,
    pub(crate) fn_read_address: FnReadString,
    pub(crate) fn_read_serial_number: FnReadString,
    pub(crate) fn_write_serial_number: FnWriteString,
    pub(crate) fn_read_batt_power: FnReadI32,
    pub(crate) fn_read_batt_voltage: FnReadI32,
    pub(crate) fn_read_sampling_frequency: FnReadSamplingFreq,
    pub(crate) fn_write_sampling_frequency: FnWriteSamplingFreq,
    pub(crate) fn_read_gain: FnReadGain,
    pub(crate) fn_write_gain: FnWriteGain,
    pub(crate) fn_read_data_offset: FnReadDataOffset,
    pub(crate) fn_write_data_offset: FnWriteDataOffset,
    pub(crate) fn_read_firmware_mode: FnReadFirmwareMode,
    pub(crate) fn_write_firmware_mode: FnWriteFirmwareMode,
    pub(crate) fn_read_version: FnReadVersion,

    // ── Hardware filters ─────────────────────────────────────────────────
    pub(crate) fn_read_hardware_filters: FnReadFilters,
    pub(crate) fn_write_hardware_filters: FnWriteFilters,
    pub(crate) fn_get_supported_filters_count: FnGetCountI32,
    pub(crate) fn_get_supported_filters: FnGetSupportedFilters,
    pub(crate) fn_is_supported_filter: FnIsSupportedFilter,

    // ── External switch / ADC ────────────────────────────────────────────
    pub(crate) fn_read_external_switch: FnReadExtSwitch,
    pub(crate) fn_write_external_switch: FnWriteExtSwitch,
    pub(crate) fn_read_adc_input: FnReadADCInput,
    pub(crate) fn_write_adc_input: FnWriteADCInput,

    // ── Accelerometer / Gyroscope ────────────────────────────────────────
    pub(crate) fn_read_accelerometer_sens: FnReadAccSens,
    pub(crate) fn_write_accelerometer_sens: FnWriteAccSens,
    pub(crate) fn_read_gyroscope_sens: FnReadGyroSens,
    pub(crate) fn_write_gyroscope_sens: FnWriteGyroSens,
    pub(crate) fn_read_sampling_frequency_mems: FnReadSamplingFreq,

    // ── Sampling frequency variants ──────────────────────────────────────
    pub(crate) fn_read_sampling_frequency_resist: FnReadSamplingFreq,
    pub(crate) fn_read_sampling_frequency_resp: FnReadSamplingFreq,
    pub(crate) fn_read_sampling_frequency_fpg: FnReadSamplingFreq,
    pub(crate) fn_read_sampling_frequency_envelope: FnReadSamplingFreq,

    // ── Callibri ─────────────────────────────────────────────────────────
    pub(crate) fn_read_color_callibri: FnReadColor,
    pub(crate) fn_read_electrode_state_callibri: FnReadElectrodeState,
    pub(crate) fn_read_color_info: FnReadColorInfo,
    pub(crate) fn_read_stim_ma_state_callibri: FnReadStimMAState,
    pub(crate) fn_read_stim_param_callibri: FnReadStimParam,
    pub(crate) fn_write_stim_param_callibri: FnWriteStimParam,
    pub(crate) fn_read_ma_param_callibri: FnReadMAParam,
    pub(crate) fn_write_ma_param_callibri: FnWriteMAParam,
    pub(crate) fn_read_motion_counter_param_callibri: FnReadMotionCounterParam,
    pub(crate) fn_write_motion_counter_param_callibri: FnWriteMotionCounterParam,
    pub(crate) fn_read_motion_counter_callibri: FnReadU32,
    pub(crate) fn_read_mems_calibrate_state_callibri: FnReadU8,
    pub(crate) fn_set_signal_settings_callibri: FnSetSignalSettingsCallibri,
    pub(crate) fn_get_signal_settings_callibri: FnGetSignalSettingsCallibri,
    pub(crate) fn_add_signal_cb_callibri: FnAddCallibriSignalCb,
    pub(crate) fn_remove_signal_cb_callibri: FnRemoveCallibriSignalCb,
    pub(crate) fn_add_respiration_cb_callibri: FnAddCallibriRespCb,
    pub(crate) fn_remove_respiration_cb_callibri: FnRemoveCallibriRespCb,
    pub(crate) fn_add_electrode_state_cb_callibri: FnAddCallibriElStateCb,
    pub(crate) fn_remove_electrode_state_cb_callibri: FnRemoveCallibriElStateCb,
    pub(crate) fn_add_envelope_cb_callibri: FnAddCallibriEnvCb,
    pub(crate) fn_remove_envelope_cb_callibri: FnRemoveCallibriEnvCb,

    // ── Quaternion ───────────────────────────────────────────────────────
    pub(crate) fn_add_quaternion_cb: FnAddQuaternionCb,
    pub(crate) fn_remove_quaternion_cb: FnRemoveQuaternionCb,

    // ── FPG ──────────────────────────────────────────────────────────────
    pub(crate) fn_read_ir_amplitude: FnReadIrAmp,
    pub(crate) fn_write_ir_amplitude: FnWriteIrAmp,
    pub(crate) fn_read_red_amplitude: FnReadRedAmp,
    pub(crate) fn_write_red_amplitude: FnWriteRedAmp,
    pub(crate) fn_add_fpg_cb: FnAddFPGCb,
    pub(crate) fn_remove_fpg_cb: FnRemoveFPGCb,

    // ── Headphones ───────────────────────────────────────────────────────
    pub(crate) fn_read_amp_headphones: FnReadAmpHeadphones,
    pub(crate) fn_write_amp_headphones: FnWriteAmpHeadphones,
    pub(crate) fn_add_signal_cb_headphones: FnAddHpSignalCb,
    pub(crate) fn_remove_signal_cb_headphones: FnRemoveHpSignalCb,
    pub(crate) fn_add_resist_cb_headphones: FnAddHpResistCb,
    pub(crate) fn_remove_resist_cb_headphones: FnRemoveHpResistCb,

    // ── Headphones2 ──────────────────────────────────────────────────────
    pub(crate) fn_read_amp_headphones2: FnReadAmpHeadphones2,
    pub(crate) fn_write_amp_headphones2: FnWriteAmpHeadphones2,
    pub(crate) fn_add_signal_cb_headphones2: FnAddHp2SignalCb,
    pub(crate) fn_remove_signal_cb_headphones2: FnRemoveHp2SignalCb,
    pub(crate) fn_add_resist_cb_headphones2: FnAddHp2ResistCb,
    pub(crate) fn_remove_resist_cb_headphones2: FnRemoveHp2ResistCb,

    // ── AmpMode ──────────────────────────────────────────────────────────
    pub(crate) fn_read_amp_mode: FnReadAmpMode,
    pub(crate) fn_add_amp_mode_cb: FnAddAmpModeCb,
    pub(crate) fn_remove_amp_mode_cb: FnRemoveAmpModeCb,

    // ── Headband ─────────────────────────────────────────────────────────
    pub(crate) fn_add_signal_cb_headband: FnAddHeadbandSignalCb,
    pub(crate) fn_remove_signal_cb_headband: FnRemoveHeadbandSignalCb,
    pub(crate) fn_add_resist_cb_headband: FnAddHeadbandResistCb,
    pub(crate) fn_remove_resist_cb_headband: FnRemoveHeadbandResistCb,

    // ── Ping ─────────────────────────────────────────────────────────────
    pub(crate) fn_ping_neuro_smart: FnPingNeuroSmart,

    // ── BrainBit (original) signal/resist ────────────────────────────────
    pub(crate) fn_add_signal_cb_bb: FnAddBBSignalCb,
    pub(crate) fn_remove_signal_cb_bb: FnRemoveBBSignalCb,
    pub(crate) fn_add_resist_cb_bb: FnAddBBResistCb,
    pub(crate) fn_remove_resist_cb_bb: FnRemoveBBResistCb,

    // ── BrainBit2 ────────────────────────────────────────────────────────
    pub(crate) fn_read_supported_channels_bb2: FnReadSupportedChannelsBB2,
    pub(crate) fn_add_signal_cb_bb2: FnAddBB2SignalCb,
    pub(crate) fn_remove_signal_cb_bb2: FnRemoveBB2SignalCb,
    pub(crate) fn_add_resist_cb_bb2: FnAddBB2ResistCb,
    pub(crate) fn_remove_resist_cb_bb2: FnRemoveBB2ResistCb,
    pub(crate) fn_read_amplifier_param_bb2: FnReadAmplifierParamBB2,
    pub(crate) fn_write_amplifier_param_bb2: FnWriteAmplifierParamBB2,

    // ── SmartBand ────────────────────────────────────────────────────────
    pub(crate) fn_read_amp_smart_band: FnReadAmpSmartBand,
    pub(crate) fn_write_amp_smart_band: FnWriteAmpSmartBand,

    // ── Supported EEG channels (generic) ─────────────────────────────────
    pub(crate) fn_read_supported_eeg_channels: FnReadSupportedEEGChannels,

    // ── NeuroEEG ─────────────────────────────────────────────────────────
    pub(crate) fn_read_supported_channels_neuro_eeg: FnReadSupportedChannelsNeuroEEG,
    pub(crate) fn_read_amp_neuro_eeg: FnReadAmpNeuroEEG,
    pub(crate) fn_write_amp_neuro_eeg: FnWriteAmpNeuroEEG,
    pub(crate) fn_add_signal_cb_neuro_eeg: FnAddNeuroEEGSignalCb,
    pub(crate) fn_remove_signal_cb_neuro_eeg: FnRemoveNeuroEEGSignalCb,
    pub(crate) fn_add_resist_cb_neuro_eeg: FnAddNeuroEEGResistCb,
    pub(crate) fn_remove_resist_cb_neuro_eeg: FnRemoveNeuroEEGResistCb,
    pub(crate) fn_add_signal_resist_cb_neuro_eeg: FnAddNeuroEEGSignalResistCb,
    pub(crate) fn_remove_signal_resist_cb_neuro_eeg: FnRemoveNeuroEEGSignalResistCb,
    pub(crate) fn_add_signal_raw_cb_neuro_eeg: FnAddNeuroEEGSignalRawCb,
    pub(crate) fn_remove_signal_raw_cb_neuro_eeg: FnRemoveNeuroEEGSignalRawCb,

    // ── NeuroEEG filesystem ──────────────────────────────────────────────
    pub(crate) fn_read_fs_status_neuro_eeg: FnReadFSStatusNeuroEEG,
    pub(crate) fn_read_fs_disk_info_neuro_eeg: FnReadFSDiskInfoNeuroEEG,
    pub(crate) fn_read_file_info_neuro_eeg: FnReadFileInfoNeuroEEG,
    pub(crate) fn_read_file_info_all_neuro_eeg: FnReadFileInfoAllNeuroEEG,
    pub(crate) fn_write_file_neuro_eeg: FnWriteFileNeuroEEG,
    pub(crate) fn_read_file_neuro_eeg: FnReadFileNeuroEEG,
    pub(crate) fn_delete_file_neuro_eeg: FnDeleteFileNeuroEEG,
    pub(crate) fn_delete_all_files_neuro_eeg: FnDeleteAllFilesNeuroEEG,
    pub(crate) fn_read_file_crc32_neuro_eeg: FnReadFileCRC32NeuroEEG,
    pub(crate) fn_file_stream_autosave_neuro_eeg: FnFileStreamAutosaveNeuroEEG,
    pub(crate) fn_file_stream_read_neuro_eeg: FnFileStreamReadNeuroEEG,
    pub(crate) fn_add_file_stream_read_cb_neuro_eeg: FnAddFileStreamReadCbNeuroEEG,
    pub(crate) fn_remove_file_stream_read_cb_neuro_eeg: FnRemoveFileStreamReadCbNeuroEEG,

    // ── NeuroEEG signal process ──────────────────────────────────────────
    pub(crate) fn_create_signal_process_param_neuro_eeg: FnCreateSignalProcessParam,
    pub(crate) fn_remove_signal_process_param_neuro_eeg: FnRemoveSignalProcessParam,
    pub(crate) fn_parse_raw_signal_neuro_eeg: FnParseRawSignal,

    // ── NeuroEEG survey + photo stim ─────────────────────────────────────
    pub(crate) fn_read_survey_id_neuro_eeg: FnReadSurveyId,
    pub(crate) fn_write_survey_id_neuro_eeg: FnWriteSurveyId,
    pub(crate) fn_read_photo_stim_neuro_eeg: FnReadPhotoStimNeuroEEG,
    pub(crate) fn_write_photo_stim_neuro_eeg: FnWritePhotoStimNeuroEEG,

    // ── PhotoStim ────────────────────────────────────────────────────────
    pub(crate) fn_get_max_stimul_phases_count: FnGetCountI32,
    pub(crate) fn_read_stim_mode: FnReadStimMode,
    pub(crate) fn_read_stim_programs: FnReadStimPrograms,
    pub(crate) fn_write_stim_programs: FnWriteStimPrograms,
    pub(crate) fn_add_stim_mode_cb: FnAddStimModeCb,
    pub(crate) fn_remove_stim_mode_cb: FnRemoveStimModeCb,
    pub(crate) fn_read_photo_stim_sync_state: FnReadPhotoStimSyncState,
    pub(crate) fn_read_photo_stim_time_defer: FnReadPhotoStimTimeDefer,
    pub(crate) fn_write_photo_stim_time_defer: FnWritePhotoStimTimeDefer,
    pub(crate) fn_add_photo_stim_sync_state_cb: FnAddPhotoStimSyncStateCb,
    pub(crate) fn_remove_photo_stim_sync_state_cb: FnRemovePhotoStimSyncStateCb,

    // ── Battery ──────────────────────────────────────────────────────────
    pub(crate) fn_add_battery_cb: FnAddBatteryCb,
    pub(crate) fn_remove_battery_cb: FnRemoveBatteryCb,
    pub(crate) fn_add_battery_voltage_cb: FnAddBatteryVoltageCb,
    pub(crate) fn_remove_battery_voltage_cb: FnRemoveBatteryVoltageCb,

    // ── Connection state ─────────────────────────────────────────────────
    pub(crate) fn_add_connection_state_cb: FnAddConnStateCb,
    pub(crate) fn_remove_connection_state_cb: FnRemoveConnStateCb,

    // ── MEMS ─────────────────────────────────────────────────────────────
    pub(crate) fn_add_mems_cb: FnAddMEMSCb,
    pub(crate) fn_remove_mems_cb: FnRemoveMEMSCb,

    // ── CRC / Ping / Possible features ───────────────────────────────────
    pub(crate) fn_calc_crc32: FnCalcCRC32,
    pub(crate) fn_get_possible_features_count: FnGetPossibleFeaturesCount,
    pub(crate) fn_get_possible_features: FnGetPossibleFeatures,
}

// SAFETY: The NeuroSDK2 library manages its own internal synchronisation.
unsafe impl Send for NeuroSdk2Lib {}
unsafe impl Sync for NeuroSdk2Lib {}

macro_rules! load_fn {
    ($lib:expr, $name:literal, $ty:ty) => {
        *$lib.get::<$ty>($name).map_err(|e| BrainBitError::LibraryNotAvailable {
            reason: format!("{}: {}", std::str::from_utf8($name).unwrap_or("?"), e),
        })?
    };
}

impl NeuroSdk2Lib {
    /// Load the NeuroSDK2 shared library from the system search path.
    ///
    /// If the `BRAINBIT_VERIFY_SDK` environment variable is set (to any value),
    /// the library file is SHA-256 verified against known-good hashes before
    /// loading. Recommended for production deployments.
    fn load() -> Result<Self, BrainBitError> {
        // Optionally find + verify the library before loading
        let verified_path = if std::env::var("BRAINBIT_VERIFY_SDK").is_ok() {
            match crate::verify::find_and_verify_library() {
                Ok(path) => {
                    log::info!("Verified SDK at: {}", path.display());
                    Some(path)
                }
                Err(e) => {
                    log::warn!("SDK verification failed ({}), falling back to system search", e);
                    None
                }
            }
        } else {
            None
        };

        let lib_name = verified_path
            .as_ref()
            .map(|p| p.as_os_str().to_owned())
            .unwrap_or_else(|| libloading::library_filename("neurosdk2"));

        let lib = unsafe { libloading::Library::new(&lib_name) }.map_err(|e| {
            BrainBitError::LibraryNotAvailable {
                reason: format!(
                    "Could not load neurosdk2 library ({:?}): {}\n\
                     Run ./sdk/download.sh to download the official library.",
                    lib_name, e
                ),
            }
        })?;

        unsafe {
            Ok(NeuroSdk2Lib {
                // ── Scanner ──────────────────────────────────────────────
                fn_create_scanner: load_fn!(lib, b"createScanner\0", FnCreateScanner),
                fn_free_scanner: load_fn!(lib, b"freeScanner\0", FnFreeScanner),
                fn_start_scanner: load_fn!(lib, b"startScanner\0", FnStartScanner),
                fn_stop_scanner: load_fn!(lib, b"stopScanner\0", FnStopScanner),
                fn_sensors_scanner: load_fn!(lib, b"sensorsScanner\0", FnSensorsScanner),
                fn_add_sensors_callback: load_fn!(lib, b"addSensorsCallbackScanner\0", FnAddSensorsCallbackScanner),
                fn_remove_sensors_callback: load_fn!(lib, b"removeSensorsCallbackScanner\0", FnRemoveSensorsCallbackScanner),

                // ── Sensor lifecycle ─────────────────────────────────────
                fn_create_sensor: load_fn!(lib, b"createSensor\0", FnCreateSensor),
                fn_free_sensor: load_fn!(lib, b"freeSensor\0", FnFreeSensor),
                fn_connect_sensor: load_fn!(lib, b"connectSensor\0", FnConnectSensor),
                fn_disconnect_sensor: load_fn!(lib, b"disconnectSensor\0", FnDisconnectSensor),

                // ── Features / commands / parameters ─────────────────────
                fn_get_features_count: load_fn!(lib, b"getFeaturesCountSensor\0", FnGetCountI32),
                fn_get_features: load_fn!(lib, b"getFeaturesSensor\0", FnGetFeatures),
                fn_is_supported_feature: load_fn!(lib, b"isSupportedFeatureSensor\0", FnIsSupportedFeature),
                fn_get_commands_count: load_fn!(lib, b"getCommandsCountSensor\0", FnGetCountI32),
                fn_get_commands: load_fn!(lib, b"getCommandsSensor\0", FnGetCommands),
                fn_is_supported_command: load_fn!(lib, b"isSupportedCommandSensor\0", FnIsSupportedCommand),
                fn_get_parameters_count: load_fn!(lib, b"getParametersCountSensor\0", FnGetCountI32),
                fn_get_parameters: load_fn!(lib, b"getParametersSensor\0", FnGetParameters),
                fn_is_supported_parameter: load_fn!(lib, b"isSupportedParameterSensor\0", FnIsSupportedParameter),
                fn_get_channels_count: load_fn!(lib, b"getChannelsCountSensor\0", FnGetCountI32),
                fn_exec_command: load_fn!(lib, b"execCommandSensor\0", FnExecCommand),

                // ── Properties ───────────────────────────────────────────
                fn_get_family: load_fn!(lib, b"getFamilySensor\0", FnGetFamily),
                fn_read_name: load_fn!(lib, b"readNameSensor\0", FnReadString),
                fn_write_name: load_fn!(lib, b"writeNameSensor\0", FnWriteString),
                fn_read_state: load_fn!(lib, b"readStateSensor\0", FnReadState),
                fn_read_address: load_fn!(lib, b"readAddressSensor\0", FnReadString),
                fn_read_serial_number: load_fn!(lib, b"readSerialNumberSensor\0", FnReadString),
                fn_write_serial_number: load_fn!(lib, b"writeSerialNumberSensor\0", FnWriteString),
                fn_read_batt_power: load_fn!(lib, b"readBattPowerSensor\0", FnReadI32),
                fn_read_batt_voltage: load_fn!(lib, b"readBattVoltageSensor\0", FnReadI32),
                fn_read_sampling_frequency: load_fn!(lib, b"readSamplingFrequencySensor\0", FnReadSamplingFreq),
                fn_write_sampling_frequency: load_fn!(lib, b"writeSamplingFrequencySensor\0", FnWriteSamplingFreq),
                fn_read_gain: load_fn!(lib, b"readGainSensor\0", FnReadGain),
                fn_write_gain: load_fn!(lib, b"writeGainSensor\0", FnWriteGain),
                fn_read_data_offset: load_fn!(lib, b"readDataOffsetSensor\0", FnReadDataOffset),
                fn_write_data_offset: load_fn!(lib, b"writeDataOffsetSensor\0", FnWriteDataOffset),
                fn_read_firmware_mode: load_fn!(lib, b"readFirmwareModeSensor\0", FnReadFirmwareMode),
                fn_write_firmware_mode: load_fn!(lib, b"writeFirmwareModeSensor\0", FnWriteFirmwareMode),
                fn_read_version: load_fn!(lib, b"readVersionSensor\0", FnReadVersion),

                // ── Hardware filters ─────────────────────────────────────
                fn_read_hardware_filters: load_fn!(lib, b"readHardwareFiltersSensor\0", FnReadFilters),
                fn_write_hardware_filters: load_fn!(lib, b"writeHardwareFiltersSensor\0", FnWriteFilters),
                fn_get_supported_filters_count: load_fn!(lib, b"getSupportedFiltersCountSensor\0", FnGetCountI32),
                fn_get_supported_filters: load_fn!(lib, b"getSupportedFiltersSensor\0", FnGetSupportedFilters),
                fn_is_supported_filter: load_fn!(lib, b"isSupportedFilterSensor\0", FnIsSupportedFilter),

                // ── External switch / ADC ────────────────────────────────
                fn_read_external_switch: load_fn!(lib, b"readExternalSwitchSensor\0", FnReadExtSwitch),
                fn_write_external_switch: load_fn!(lib, b"writeExternalSwitchSensor\0", FnWriteExtSwitch),
                fn_read_adc_input: load_fn!(lib, b"readADCInputSensor\0", FnReadADCInput),
                fn_write_adc_input: load_fn!(lib, b"writeADCInputSensor\0", FnWriteADCInput),

                // ── Accelerometer / Gyroscope ────────────────────────────
                fn_read_accelerometer_sens: load_fn!(lib, b"readAccelerometerSensSensor\0", FnReadAccSens),
                fn_write_accelerometer_sens: load_fn!(lib, b"writeAccelerometerSensSensor\0", FnWriteAccSens),
                fn_read_gyroscope_sens: load_fn!(lib, b"readGyroscopeSensSensor\0", FnReadGyroSens),
                fn_write_gyroscope_sens: load_fn!(lib, b"writeGyroscopeSensSensor\0", FnWriteGyroSens),
                fn_read_sampling_frequency_mems: load_fn!(lib, b"readSamplingFrequencyMEMSSensor\0", FnReadSamplingFreq),

                // ── Sampling frequency variants ──────────────────────────
                fn_read_sampling_frequency_resist: load_fn!(lib, b"readSamplingFrequencyResistSensor\0", FnReadSamplingFreq),
                fn_read_sampling_frequency_resp: load_fn!(lib, b"readSamplingFrequencyRespSensor\0", FnReadSamplingFreq),
                fn_read_sampling_frequency_fpg: load_fn!(lib, b"readSamplingFrequencyFPGSensor\0", FnReadSamplingFreq),
                fn_read_sampling_frequency_envelope: load_fn!(lib, b"readSamplingFrequencyEnvelopeSensor\0", FnReadSamplingFreq),

                // ── Callibri ─────────────────────────────────────────────
                fn_read_color_callibri: load_fn!(lib, b"readColorCallibri\0", FnReadColor),
                fn_read_electrode_state_callibri: load_fn!(lib, b"readElectrodeStateCallibri\0", FnReadElectrodeState),
                fn_read_color_info: load_fn!(lib, b"readColorInfo\0", FnReadColorInfo),
                fn_read_stim_ma_state_callibri: load_fn!(lib, b"readStimulatorAndMAStateCallibri\0", FnReadStimMAState),
                fn_read_stim_param_callibri: load_fn!(lib, b"readStimulatorParamCallibri\0", FnReadStimParam),
                fn_write_stim_param_callibri: load_fn!(lib, b"writeStimulatorParamCallibri\0", FnWriteStimParam),
                fn_read_ma_param_callibri: load_fn!(lib, b"readMotionAssistantParamCallibri\0", FnReadMAParam),
                fn_write_ma_param_callibri: load_fn!(lib, b"writeMotionAssistantParamCallibri\0", FnWriteMAParam),
                fn_read_motion_counter_param_callibri: load_fn!(lib, b"readMotionCounterParamCallibri\0", FnReadMotionCounterParam),
                fn_write_motion_counter_param_callibri: load_fn!(lib, b"writeMotionCounterParamCallibri\0", FnWriteMotionCounterParam),
                fn_read_motion_counter_callibri: load_fn!(lib, b"readMotionCounterCallibri\0", FnReadU32),
                fn_read_mems_calibrate_state_callibri: load_fn!(lib, b"readMEMSCalibrateStateCallibri\0", FnReadU8),
                fn_set_signal_settings_callibri: load_fn!(lib, b"setSignalSettingsCallibri\0", FnSetSignalSettingsCallibri),
                fn_get_signal_settings_callibri: load_fn!(lib, b"getSignalSettingsCallibri\0", FnGetSignalSettingsCallibri),
                fn_add_signal_cb_callibri: load_fn!(lib, b"addSignalCallbackCallibri\0", FnAddCallibriSignalCb),
                fn_remove_signal_cb_callibri: load_fn!(lib, b"removeSignalCallbackCallibri\0", FnRemoveCallibriSignalCb),
                fn_add_respiration_cb_callibri: load_fn!(lib, b"addRespirationCallbackCallibri\0", FnAddCallibriRespCb),
                fn_remove_respiration_cb_callibri: load_fn!(lib, b"removeRespirationCallbackCallibri\0", FnRemoveCallibriRespCb),
                fn_add_electrode_state_cb_callibri: load_fn!(lib, b"addElectrodeStateCallbackCallibri\0", FnAddCallibriElStateCb),
                fn_remove_electrode_state_cb_callibri: load_fn!(lib, b"removeElectrodeStateCallbackCallibri\0", FnRemoveCallibriElStateCb),
                fn_add_envelope_cb_callibri: load_fn!(lib, b"addEnvelopeDataCallbackCallibri\0", FnAddCallibriEnvCb),
                fn_remove_envelope_cb_callibri: load_fn!(lib, b"removeEnvelopeDataCallbackCallibri\0", FnRemoveCallibriEnvCb),

                // ── Quaternion ───────────────────────────────────────────
                fn_add_quaternion_cb: load_fn!(lib, b"addQuaternionDataCallback\0", FnAddQuaternionCb),
                fn_remove_quaternion_cb: load_fn!(lib, b"removeQuaternionDataCallback\0", FnRemoveQuaternionCb),

                // ── FPG ──────────────────────────────────────────────────
                fn_read_ir_amplitude: load_fn!(lib, b"readIrAmplitudeFPGSensor\0", FnReadIrAmp),
                fn_write_ir_amplitude: load_fn!(lib, b"writeIrAmplitudeFPGSensor\0", FnWriteIrAmp),
                fn_read_red_amplitude: load_fn!(lib, b"readRedAmplitudeFPGSensor\0", FnReadRedAmp),
                fn_write_red_amplitude: load_fn!(lib, b"writeRedAmplitudeFPGSensor\0", FnWriteRedAmp),
                fn_add_fpg_cb: load_fn!(lib, b"addFPGDataCallback\0", FnAddFPGCb),
                fn_remove_fpg_cb: load_fn!(lib, b"removeFPGDataCallback\0", FnRemoveFPGCb),

                // ── Headphones ───────────────────────────────────────────
                fn_read_amp_headphones: load_fn!(lib, b"readAmplifierParamHeadphones\0", FnReadAmpHeadphones),
                fn_write_amp_headphones: load_fn!(lib, b"writeAmplifierParamHeadphones\0", FnWriteAmpHeadphones),
                fn_add_signal_cb_headphones: load_fn!(lib, b"addSignalDataCallbackHeadphones\0", FnAddHpSignalCb),
                fn_remove_signal_cb_headphones: load_fn!(lib, b"removeSignalDataCallbackHeadphones\0", FnRemoveHpSignalCb),
                fn_add_resist_cb_headphones: load_fn!(lib, b"addResistCallbackHeadphones\0", FnAddHpResistCb),
                fn_remove_resist_cb_headphones: load_fn!(lib, b"removeResistCallbackHeadphones\0", FnRemoveHpResistCb),

                // ── Headphones2 ──────────────────────────────────────────
                fn_read_amp_headphones2: load_fn!(lib, b"readAmplifierParamHeadphones2\0", FnReadAmpHeadphones2),
                fn_write_amp_headphones2: load_fn!(lib, b"writeAmplifierParamHeadphones2\0", FnWriteAmpHeadphones2),
                fn_add_signal_cb_headphones2: load_fn!(lib, b"addSignalDataCallbackHeadphones2\0", FnAddHp2SignalCb),
                fn_remove_signal_cb_headphones2: load_fn!(lib, b"removeSignalDataCallbackHeadphones2\0", FnRemoveHp2SignalCb),
                fn_add_resist_cb_headphones2: load_fn!(lib, b"addResistCallbackHeadphones2\0", FnAddHp2ResistCb),
                fn_remove_resist_cb_headphones2: load_fn!(lib, b"removeResistCallbackHeadphones2\0", FnRemoveHp2ResistCb),

                // ── AmpMode ──────────────────────────────────────────────
                fn_read_amp_mode: load_fn!(lib, b"readAmpMode\0", FnReadAmpMode),
                fn_add_amp_mode_cb: load_fn!(lib, b"addAmpModeCallback\0", FnAddAmpModeCb),
                fn_remove_amp_mode_cb: load_fn!(lib, b"removeAmpModeCallback\0", FnRemoveAmpModeCb),

                // ── Headband ─────────────────────────────────────────────
                fn_add_signal_cb_headband: load_fn!(lib, b"addSignalDataCallbackHeadband\0", FnAddHeadbandSignalCb),
                fn_remove_signal_cb_headband: load_fn!(lib, b"removeSignalDataCallbackHeadband\0", FnRemoveHeadbandSignalCb),
                fn_add_resist_cb_headband: load_fn!(lib, b"addResistCallbackHeadband\0", FnAddHeadbandResistCb),
                fn_remove_resist_cb_headband: load_fn!(lib, b"removeResistCallbackHeadband\0", FnRemoveHeadbandResistCb),

                // ── Ping ─────────────────────────────────────────────────
                fn_ping_neuro_smart: load_fn!(lib, b"pingNeuroSmart\0", FnPingNeuroSmart),

                // ── BrainBit (original) signal/resist ────────────────────
                fn_add_signal_cb_bb: load_fn!(lib, b"addSignalDataCallbackBrainBit\0", FnAddBBSignalCb),
                fn_remove_signal_cb_bb: load_fn!(lib, b"removeSignalDataCallbackBrainBit\0", FnRemoveBBSignalCb),
                fn_add_resist_cb_bb: load_fn!(lib, b"addResistCallbackBrainBit\0", FnAddBBResistCb),
                fn_remove_resist_cb_bb: load_fn!(lib, b"removeResistCallbackBrainBit\0", FnRemoveBBResistCb),

                // ── BrainBit2 ────────────────────────────────────────────
                fn_read_supported_channels_bb2: load_fn!(lib, b"readSupportedChannelsBrainBit2\0", FnReadSupportedChannelsBB2),
                fn_add_signal_cb_bb2: load_fn!(lib, b"addSignalCallbackBrainBit2\0", FnAddBB2SignalCb),
                fn_remove_signal_cb_bb2: load_fn!(lib, b"removeSignalCallbackBrainBit2\0", FnRemoveBB2SignalCb),
                fn_add_resist_cb_bb2: load_fn!(lib, b"addResistCallbackBrainBit2\0", FnAddBB2ResistCb),
                fn_remove_resist_cb_bb2: load_fn!(lib, b"removeResistCallbackBrainBit2\0", FnRemoveBB2ResistCb),
                fn_read_amplifier_param_bb2: load_fn!(lib, b"readAmplifierParamBrainBit2\0", FnReadAmplifierParamBB2),
                fn_write_amplifier_param_bb2: load_fn!(lib, b"writeAmplifierParamBrainBit2\0", FnWriteAmplifierParamBB2),

                // ── SmartBand ────────────────────────────────────────────
                fn_read_amp_smart_band: load_fn!(lib, b"readAmplifierParamSmartBand\0", FnReadAmpSmartBand),
                fn_write_amp_smart_band: load_fn!(lib, b"writeAmplifierParamSmartBand\0", FnWriteAmpSmartBand),

                // ── Supported EEG channels (generic) ─────────────────────
                fn_read_supported_eeg_channels: load_fn!(lib, b"readSupportedEEGChannels\0", FnReadSupportedEEGChannels),

                // ── NeuroEEG ─────────────────────────────────────────────
                fn_read_supported_channels_neuro_eeg: load_fn!(lib, b"readSupportedChannelsNeuroEEG\0", FnReadSupportedChannelsNeuroEEG),
                fn_read_amp_neuro_eeg: load_fn!(lib, b"readAmplifierParamNeuroEEG\0", FnReadAmpNeuroEEG),
                fn_write_amp_neuro_eeg: load_fn!(lib, b"writeAmplifierParamNeuroEEG\0", FnWriteAmpNeuroEEG),
                fn_add_signal_cb_neuro_eeg: load_fn!(lib, b"addSignalCallbackNeuroEEG\0", FnAddNeuroEEGSignalCb),
                fn_remove_signal_cb_neuro_eeg: load_fn!(lib, b"removeSignalCallbackNeuroEEG\0", FnRemoveNeuroEEGSignalCb),
                fn_add_resist_cb_neuro_eeg: load_fn!(lib, b"addResistCallbackNeuroEEG\0", FnAddNeuroEEGResistCb),
                fn_remove_resist_cb_neuro_eeg: load_fn!(lib, b"removeResistCallbackNeuroEEG\0", FnRemoveNeuroEEGResistCb),
                fn_add_signal_resist_cb_neuro_eeg: load_fn!(lib, b"addSignalResistCallbackNeuroEEG\0", FnAddNeuroEEGSignalResistCb),
                fn_remove_signal_resist_cb_neuro_eeg: load_fn!(lib, b"removeSignalResistCallbackNeuroEEG\0", FnRemoveNeuroEEGSignalResistCb),
                fn_add_signal_raw_cb_neuro_eeg: load_fn!(lib, b"addSignalRawCallbackNeuroEEG\0", FnAddNeuroEEGSignalRawCb),
                fn_remove_signal_raw_cb_neuro_eeg: load_fn!(lib, b"removeSignalRawCallbackNeuroEEG\0", FnRemoveNeuroEEGSignalRawCb),

                // ── NeuroEEG filesystem ──────────────────────────────────
                fn_read_fs_status_neuro_eeg: load_fn!(lib, b"readFilesystemStatusNeuroEEG\0", FnReadFSStatusNeuroEEG),
                fn_read_fs_disk_info_neuro_eeg: load_fn!(lib, b"readFileSystemDiskInfoNeuroEEG\0", FnReadFSDiskInfoNeuroEEG),
                fn_read_file_info_neuro_eeg: load_fn!(lib, b"readFileInfoNeuroEEG\0", FnReadFileInfoNeuroEEG),
                fn_read_file_info_all_neuro_eeg: load_fn!(lib, b"readFileInfoAllNeuroEEG\0", FnReadFileInfoAllNeuroEEG),
                fn_write_file_neuro_eeg: load_fn!(lib, b"writeFileNeuroEEG\0", FnWriteFileNeuroEEG),
                fn_read_file_neuro_eeg: load_fn!(lib, b"readFileNeuroEEG\0", FnReadFileNeuroEEG),
                fn_delete_file_neuro_eeg: load_fn!(lib, b"deleteFileNeuroEEG\0", FnDeleteFileNeuroEEG),
                fn_delete_all_files_neuro_eeg: load_fn!(lib, b"deleteAllFilesNeuroEEG\0", FnDeleteAllFilesNeuroEEG),
                fn_read_file_crc32_neuro_eeg: load_fn!(lib, b"readFileCRC32NeuroEEG\0", FnReadFileCRC32NeuroEEG),
                fn_file_stream_autosave_neuro_eeg: load_fn!(lib, b"fileStreamAutosaveNeuroEEG\0", FnFileStreamAutosaveNeuroEEG),
                fn_file_stream_read_neuro_eeg: load_fn!(lib, b"fileStreamReadNeuroEEG\0", FnFileStreamReadNeuroEEG),
                fn_add_file_stream_read_cb_neuro_eeg: load_fn!(lib, b"addFileStreamReadCallbackNeuroEEG\0", FnAddFileStreamReadCbNeuroEEG),
                fn_remove_file_stream_read_cb_neuro_eeg: load_fn!(lib, b"removeFileStreamReadCallbackNeuroEEG\0", FnRemoveFileStreamReadCbNeuroEEG),

                // ── NeuroEEG signal process ──────────────────────────────
                fn_create_signal_process_param_neuro_eeg: load_fn!(lib, b"createSignalProcessParamNeuroEEG\0", FnCreateSignalProcessParam),
                fn_remove_signal_process_param_neuro_eeg: load_fn!(lib, b"removeSignalProcessParamNeuroEEG\0", FnRemoveSignalProcessParam),
                fn_parse_raw_signal_neuro_eeg: load_fn!(lib, b"parseRawSignalNeuroEEG\0", FnParseRawSignal),

                // ── NeuroEEG survey + photo stim ─────────────────────────
                fn_read_survey_id_neuro_eeg: load_fn!(lib, b"readSurveyIdNeuroEEG\0", FnReadSurveyId),
                fn_write_survey_id_neuro_eeg: load_fn!(lib, b"writeSurveyIdNeuroEEG\0", FnWriteSurveyId),
                fn_read_photo_stim_neuro_eeg: load_fn!(lib, b"readPhotoStimNeuroEEG\0", FnReadPhotoStimNeuroEEG),
                fn_write_photo_stim_neuro_eeg: load_fn!(lib, b"writePhotoStimNeuroEEG\0", FnWritePhotoStimNeuroEEG),

                // ── PhotoStim ────────────────────────────────────────────
                fn_get_max_stimul_phases_count: load_fn!(lib, b"getMaxStimulPhasesCountSensor\0", FnGetCountI32),
                fn_read_stim_mode: load_fn!(lib, b"readStimMode\0", FnReadStimMode),
                fn_read_stim_programs: load_fn!(lib, b"readStimPrograms\0", FnReadStimPrograms),
                fn_write_stim_programs: load_fn!(lib, b"writeStimPrograms\0", FnWriteStimPrograms),
                fn_add_stim_mode_cb: load_fn!(lib, b"addStimModeCallback\0", FnAddStimModeCb),
                fn_remove_stim_mode_cb: load_fn!(lib, b"removeStimModeCallback\0", FnRemoveStimModeCb),
                fn_read_photo_stim_sync_state: load_fn!(lib, b"readPhotoStimSyncState\0", FnReadPhotoStimSyncState),
                fn_read_photo_stim_time_defer: load_fn!(lib, b"readPhotoStimTimeDefer\0", FnReadPhotoStimTimeDefer),
                fn_write_photo_stim_time_defer: load_fn!(lib, b"writePhotoStimTimeDefer\0", FnWritePhotoStimTimeDefer),
                fn_add_photo_stim_sync_state_cb: load_fn!(lib, b"addPhotoStimSyncStateCallback\0", FnAddPhotoStimSyncStateCb),
                fn_remove_photo_stim_sync_state_cb: load_fn!(lib, b"removePhotoStimSyncStateCallback\0", FnRemovePhotoStimSyncStateCb),

                // ── Battery ──────────────────────────────────────────────
                fn_add_battery_cb: load_fn!(lib, b"addBatteryCallback\0", FnAddBatteryCb),
                fn_remove_battery_cb: load_fn!(lib, b"removeBatteryCallback\0", FnRemoveBatteryCb),
                fn_add_battery_voltage_cb: load_fn!(lib, b"addBatteryVoltageCallback\0", FnAddBatteryVoltageCb),
                fn_remove_battery_voltage_cb: load_fn!(lib, b"removeBatteryVoltageCallback\0", FnRemoveBatteryVoltageCb),

                // ── Connection state ─────────────────────────────────────
                fn_add_connection_state_cb: load_fn!(lib, b"addConnectionStateCallback\0", FnAddConnStateCb),
                fn_remove_connection_state_cb: load_fn!(lib, b"removeConnectionStateCallback\0", FnRemoveConnStateCb),

                // ── MEMS ─────────────────────────────────────────────────
                fn_add_mems_cb: load_fn!(lib, b"addMEMSDataCallback\0", FnAddMEMSCb),
                fn_remove_mems_cb: load_fn!(lib, b"removeMEMSDataCallback\0", FnRemoveMEMSCb),

                // ── CRC / Ping / Possible features ───────────────────────
                fn_calc_crc32: load_fn!(lib, b"calcCRC32\0", FnCalcCRC32),
                fn_get_possible_features_count: load_fn!(lib, b"getPossibleFeaturesCountSensor\0", FnGetPossibleFeaturesCount),
                fn_get_possible_features: load_fn!(lib, b"getPossibleFeaturesSensor\0", FnGetPossibleFeatures),

                _lib: lib,
            })
        }
    }
}

// ── Singleton accessor ───────────────────────────────────────────────────────

static SDK_LIB: OnceLock<Result<NeuroSdk2Lib, String>> = OnceLock::new();

/// Get the global NeuroSDK2 library handle (loaded once on first call).
pub fn sdk_lib() -> Result<&'static NeuroSdk2Lib, BrainBitError> {
    SDK_LIB
        .get_or_init(|| NeuroSdk2Lib::load().map_err(|e| e.to_string()))
        .as_ref()
        .map_err(|e| BrainBitError::LibraryNotAvailable {
            reason: e.clone(),
        })
}
