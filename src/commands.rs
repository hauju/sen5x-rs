#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// List of SEN5x sensor commands.
pub enum Command {
    /// Start measurement mode. Signal update interval is 1 second.
    StartMeasurement,
    /// Start measurement without PM. Only RH/T/VOC/NOx are measured.
    StartMeasurementWithoutPm,
    /// Stop measurement and return to idle mode.
    StopMeasurement,
    /// Check whether new measurement data is available for read-out.
    ReadDataReady,
    /// Read measured values as integers (scaled ticks).
    ReadMeasuredValuesAsIntegers,
    /// Read measured raw values (unscaled sensor ticks).
    ReadMeasuredRawValues,
    /// Read extended PM values including number concentrations and typical particle size.
    ReadMeasuredPmValues,
    /// Start fan cleaning manually. Only available in measurement mode with PM.
    StartFanCleaning,
    /// Set temperature compensation parameters (offset, slope, time constant).
    SetTemperatureOffsetParameters,
    /// Get temperature compensation parameters.
    GetTemperatureOffsetParameters,
    /// Set warm start parameter (0–65535). Applied at next measurement start.
    SetWarmStartParameter,
    /// Get warm start parameter.
    GetWarmStartParameter,
    /// Set VOC algorithm tuning parameters (6 values).
    SetVocAlgorithmTuningParameters,
    /// Get VOC algorithm tuning parameters.
    GetVocAlgorithmTuningParameters,
    /// Set NOx algorithm tuning parameters (6 values). SEN55 only.
    SetNoxAlgorithmTuningParameters,
    /// Get NOx algorithm tuning parameters. SEN55 only.
    GetNoxAlgorithmTuningParameters,
    /// Set RH/T acceleration mode. Applied at next measurement start.
    SetRhtAccelerationMode,
    /// Get RH/T acceleration mode.
    GetRhtAccelerationMode,
    /// Set VOC algorithm state for warm start after power cycle.
    SetVocAlgorithmState,
    /// Get VOC algorithm state for backup.
    GetVocAlgorithmState,
    /// Set fan auto cleaning interval in seconds. 0 disables auto cleaning.
    SetFanAutoCleaningInterval,
    /// Get fan auto cleaning interval in seconds.
    GetFanAutoCleaningInterval,
    /// Get the product name as ASCII string.
    GetProductName,
    /// Get the serial number as ASCII string.
    GetSerialNumber,
    /// Get firmware, hardware, and protocol version information.
    GetVersion,
    /// Read device status register.
    ReadDeviceStatus,
    /// Read and clear device status register.
    ReadAndClearDeviceStatus,
    /// Reset the sensor (equivalent to power cycle).
    DeviceReset,
}

impl Command {
    /// Returns (command_code, delay_ms, allowed_if_running).
    pub fn as_tuple(self) -> (u16, u32, bool) {
        match self {
            Self::StartMeasurement => (0x0021, 50, false),
            Self::StartMeasurementWithoutPm => (0x0037, 50, false),
            Self::StopMeasurement => (0x0104, 200, true),
            Self::ReadDataReady => (0x0202, 20, true),
            Self::ReadMeasuredValuesAsIntegers => (0x03C4, 20, true),
            Self::ReadMeasuredRawValues => (0x03D2, 20, true),
            Self::ReadMeasuredPmValues => (0x0413, 20, true),
            Self::StartFanCleaning => (0x5607, 20, true),
            Self::SetTemperatureOffsetParameters => (0x60B2, 20, false),
            Self::GetTemperatureOffsetParameters => (0x60B2, 20, true),
            Self::SetWarmStartParameter => (0x60C6, 20, false),
            Self::GetWarmStartParameter => (0x60C6, 20, true),
            Self::SetVocAlgorithmTuningParameters => (0x60D0, 20, false),
            Self::GetVocAlgorithmTuningParameters => (0x60D0, 20, true),
            Self::SetNoxAlgorithmTuningParameters => (0x60E1, 20, false),
            Self::GetNoxAlgorithmTuningParameters => (0x60E1, 20, true),
            Self::SetRhtAccelerationMode => (0x60F7, 20, false),
            Self::GetRhtAccelerationMode => (0x60F7, 20, true),
            Self::SetVocAlgorithmState => (0x6181, 20, false),
            Self::GetVocAlgorithmState => (0x6181, 20, true),
            Self::SetFanAutoCleaningInterval => (0x8004, 20, false),
            Self::GetFanAutoCleaningInterval => (0x8004, 20, true),
            Self::GetProductName => (0xD014, 50, false),
            Self::GetSerialNumber => (0xD033, 50, false),
            Self::GetVersion => (0xD100, 20, false),
            Self::ReadDeviceStatus => (0xD206, 20, true),
            Self::ReadAndClearDeviceStatus => (0xD210, 20, true),
            Self::DeviceReset => (0xD304, 200, true),
        }
    }
}
