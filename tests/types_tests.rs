//! Unit tests for FFI type layouts and conversions (no hardware required).

use brainbit::types::*;

// ── OpStatus ─────────────────────────────────────────────────────────────────

#[test]
fn test_op_status_default_is_not_ok() {
    let s = OpStatus::default();
    // success=0 means failure
    assert!(!s.is_ok());
    assert_eq!(s.error, 0);
    assert_eq!(s.message(), "");
}

#[test]
fn test_op_status_success() {
    let mut s = OpStatus::default();
    s.success = 1;
    assert!(s.is_ok());
}

#[test]
fn test_op_status_message_extraction() {
    let mut s = OpStatus::default();
    let msg = b"test error message";
    s.error_msg[..msg.len()].copy_from_slice(msg);
    assert_eq!(s.message(), "test error message");
}

#[test]
fn test_op_status_message_full_buffer() {
    let mut s = OpStatus::default();
    // Fill entire buffer with non-null bytes
    s.error_msg.fill(b'A');
    assert_eq!(s.message().len(), ERR_MSG_LEN);
}

// ── SensorInfo ───────────────────────────────────────────────────────────────

#[test]
fn test_sensor_info_default() {
    let info = SensorInfo::default();
    assert_eq!(info.sens_family, SensorFamily::Unknown);
    assert_eq!(info.sens_model, 0);
    assert_eq!(info.name_str(), "");
    assert_eq!(info.address_str(), "");
    assert_eq!(info.serial_number_str(), "");
    assert_eq!(info.rssi, 0);
}

#[test]
fn test_sensor_info_name_extraction() {
    let mut info = SensorInfo::default();
    let name = b"BrainBit";
    info.name[..name.len()].copy_from_slice(name);
    assert_eq!(info.name_str(), "BrainBit");
}

#[test]
fn test_sensor_info_address_extraction() {
    let mut info = SensorInfo::default();
    let addr = b"AA:BB:CC:DD:EE:FF";
    info.address[..addr.len()].copy_from_slice(addr);
    assert_eq!(info.address_str(), "AA:BB:CC:DD:EE:FF");
}

// ── SensorFamily ─────────────────────────────────────────────────────────────

#[test]
fn test_sensor_family_values() {
    assert_eq!(SensorFamily::Unknown as u8, 0);
    assert_eq!(SensorFamily::LEBrainBit as u8, 3);
    assert_eq!(SensorFamily::LEBrainBit2 as u8, 18);
    assert_eq!(SensorFamily::LEBrainBitPro as u8, 19);
    assert_eq!(SensorFamily::LEBrainBitFlex as u8, 20);
    assert_eq!(SensorFamily::LECallibri as u8, 1);
    assert_eq!(SensorFamily::LENeuroEEG as u8, 14);
    assert_eq!(SensorFamily::LEHeadPhones as u8, 5);
    assert_eq!(SensorFamily::LEHeadband as u8, 11);
    assert_eq!(SensorFamily::LEPhotoStim as u8, 21);
}

// ── SensorSamplingFrequency ──────────────────────────────────────────────────

#[test]
fn test_sampling_frequency_to_hz() {
    assert_eq!(SensorSamplingFrequency::Hz250.to_hz(), Some(250));
    assert_eq!(SensorSamplingFrequency::Hz500.to_hz(), Some(500));
    assert_eq!(SensorSamplingFrequency::Hz1000.to_hz(), Some(1000));
    assert_eq!(SensorSamplingFrequency::Hz10.to_hz(), Some(10));
    assert_eq!(SensorSamplingFrequency::Unsupported.to_hz(), None);
}

#[test]
fn test_sampling_frequency_values() {
    assert_eq!(SensorSamplingFrequency::Hz10 as u8, 0);
    assert_eq!(SensorSamplingFrequency::Hz250 as u8, 4);
    assert_eq!(SensorSamplingFrequency::Unsupported as u8, 0xFF);
}

// ── SensorGain ───────────────────────────────────────────────────────────────

#[test]
fn test_sensor_gain_values() {
    assert_eq!(SensorGain::Gain1 as i8, 0);
    assert_eq!(SensorGain::Gain3 as i8, 2);
    assert_eq!(SensorGain::Gain8 as i8, 5);
    assert_eq!(SensorGain::Unsupported as i8, 11);
}

// ── SensorCommand ────────────────────────────────────────────────────────────

#[test]
fn test_sensor_command_values() {
    assert_eq!(SensorCommand::StartSignal as i8, 0);
    assert_eq!(SensorCommand::StopSignal as i8, 1);
    assert_eq!(SensorCommand::StartResist as i8, 2);
    assert_eq!(SensorCommand::StopResist as i8, 3);
    assert_eq!(SensorCommand::FindMe as i8, 12);
    assert_eq!(SensorCommand::PowerDown as i8, 22);
}

// ── SensorState ──────────────────────────────────────────────────────────────

#[test]
fn test_sensor_state_values() {
    assert_eq!(SensorState::InRange as i8, 0);
    assert_eq!(SensorState::OutOfRange as i8, 1);
}

// ── ParameterInfo ────────────────────────────────────────────────────────────

#[test]
fn test_parameter_info_layout() {
    let pi = ParameterInfo {
        param: SensorParameter::Gain,
        param_access: SensorParamAccess::ReadWrite,
    };
    assert_eq!(pi.param as i8, 7);
    assert_eq!(pi.param_access as i8, 1);
}

// ── BrainBitSignalData ───────────────────────────────────────────────────────

#[test]
fn test_brainbit_signal_data_layout() {
    let d = BrainBitSignalData {
        pack_num: 42,
        marker: 0,
        o1: 0.000_001,
        o2: 0.000_002,
        t3: -0.000_001,
        t4: 0.0,
    };
    assert_eq!(d.pack_num, 42);
    assert!((d.o1 - 1e-6).abs() < 1e-12);
    assert!((d.t3 + 1e-6).abs() < 1e-12);
}

// ── BrainBitResistData ───────────────────────────────────────────────────────

#[test]
fn test_brainbit_resist_data_layout() {
    let d = BrainBitResistData {
        o1: 1000.0,
        o2: 2000.0,
        t3: 3000.0,
        t4: 4000.0,
    };
    assert_eq!(d.o1 as u32, 1000);
    assert_eq!(d.t4 as u32, 4000);
}

// ── EEGChannelInfo ───────────────────────────────────────────────────────────

#[test]
fn test_eeg_channel_info_name() {
    let mut ch = EEGChannelInfo {
        id: EEGChannelId::O1,
        ch_type: EEGChannelType::SingleA1,
        name: [0u8; SENSOR_CHANNEL_NAME_LEN],
        num: 0,
    };
    ch.name[..2].copy_from_slice(b"O1");
    assert_eq!(ch.name_str(), "O1");
    assert_eq!(ch.id, EEGChannelId::O1);
}

#[test]
fn test_eeg_channel_id_values() {
    assert_eq!(EEGChannelId::O1 as u8, 1);
    assert_eq!(EEGChannelId::O2 as u8, 16);
    assert_eq!(EEGChannelId::T3 as u8, 7);
    assert_eq!(EEGChannelId::T4 as u8, 10);
    assert_eq!(EEGChannelId::Fp1 as u8, 5);
    assert_eq!(EEGChannelId::Fp2 as u8, 12);
}

// ── Struct sizes (ABI compatibility) ─────────────────────────────────────────

#[test]
fn test_op_status_size() {
    // success(1) + padding(3) + error(4) + error_msg(512) = 520
    // But actual layout depends on alignment — just check it's reasonable
    let size = std::mem::size_of::<OpStatus>();
    assert!(size >= 517, "OpStatus too small: {}", size);
    assert!(size <= 520, "OpStatus too large: {}", size);
}

#[test]
fn test_sensor_info_size() {
    let size = std::mem::size_of::<SensorInfo>();
    // family(1) + model(1) + name(256) + address(128) + serial(128) + pairing(1) + rssi(2)
    // = 517 minimum
    assert!(size >= 517, "SensorInfo too small: {}", size);
}

#[test]
fn test_brainbit_signal_data_size() {
    let size = std::mem::size_of::<BrainBitSignalData>();
    // pack_num(4) + marker(1) + padding(3) + o1(8) + o2(8) + t3(8) + t4(8) = 40
    assert_eq!(size, 40);
}

#[test]
fn test_brainbit_resist_data_size() {
    let size = std::mem::size_of::<BrainBitResistData>();
    // 4 × f64 = 32
    assert_eq!(size, 32);
}

#[test]
fn test_sensor_version_size() {
    let size = std::mem::size_of::<SensorVersion>();
    // 7 × u32 = 28
    assert_eq!(size, 28);
}

#[test]
fn test_parameter_info_size() {
    let size = std::mem::size_of::<ParameterInfo>();
    // param(1) + param_access(1) = 2
    assert_eq!(size, 2);
}

// ── Callibri types ───────────────────────────────────────────────────────────

#[test]
fn test_callibri_color_values() {
    assert_eq!(CallibriColorType::Red as u8, 0);
    assert_eq!(CallibriColorType::Yellow as u8, 1);
    assert_eq!(CallibriColorType::Blue as u8, 2);
    assert_eq!(CallibriColorType::White as u8, 3);
    assert_eq!(CallibriColorType::Unknown as u8, 4);
}

#[test]
fn test_signal_type_callibri_values() {
    assert_eq!(SignalTypeCallibri::EEG as u8, 0);
    assert_eq!(SignalTypeCallibri::EMG as u8, 1);
    assert_eq!(SignalTypeCallibri::ECG as u8, 2);
    assert_eq!(SignalTypeCallibri::EDA as u8, 3);
}

// ── AmpMode ──────────────────────────────────────────────────────────────────

#[test]
fn test_amp_mode_values() {
    assert_eq!(SensorAmpMode::Invalid as u8, 0);
    assert_eq!(SensorAmpMode::Signal as u8, 3);
    assert_eq!(SensorAmpMode::Resist as u8, 4);
    assert_eq!(SensorAmpMode::Envelope as u8, 6);
}

// ── StimulPhase ──────────────────────────────────────────────────────────────

#[test]
fn test_stimul_phase_layout() {
    let phase = StimulPhase {
        frequency: 10.0,
        power: 50.0,
        pulse: 0.001,
        stimul_duration: 5.0,
        pause: 2.0,
        filling_frequency: 1000.0,
    };
    assert_eq!(std::mem::size_of::<StimulPhase>(), 48); // 6 × f64
    assert_eq!(phase.frequency as u32, 10);
    assert_eq!(phase.power as u32, 50);
}

// ── SensorFilter ─────────────────────────────────────────────────────────────

#[test]
fn test_sensor_filter_values() {
    assert_eq!(SensorFilter::HPFBwhLvl1CutoffFreq1Hz as u16, 0);
    assert_eq!(SensorFilter::BSFBwhLvl2CutoffFreq45_55Hz as u16, 2);
    assert_eq!(SensorFilter::Unknown as u16, 0xFF);
}
