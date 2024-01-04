#[derive(Debug, Copy, Clone)]
/// List of SCD4x sensor commands.
pub enum Command {
    DeviceReset,
    GetSerialNumber,
    GetVersion,
    GetProductName,
    ReadDeviceStatus,
    SetTemperatureOffsetParameters,
    StartMeasurement,
    StartMeasurementWithoutPm,
    StopMeasurement,
    ReadMeasuredValuesAsIntegers,
    ReadMeasuredRawValues,
    StartFanCleaning,
    GetWarmStartParameter,
    SetWarmStartParameter,

}

impl Command {
    /// Get the command byte
    pub fn as_tuple(&self) -> (u16, u32) {
        match self {
            Command::DeviceReset => (0xD304, 200),
            Command::GetSerialNumber => (0xD033, 50),
            Command::GetVersion => (0xD100, 20),
            Command::GetProductName => (0xD014, 50),
            Command::ReadDeviceStatus => (0xD206, 20),
            Command::SetTemperatureOffsetParameters => (0x60B2, 20),
            Command::StartMeasurement => (0x0021, 50),
            Command::StartMeasurementWithoutPm => (0x0037, 50),
            Command::StopMeasurement => (0x0104, 50),
            Command::ReadMeasuredValuesAsIntegers => (0x03C4, 20),
            Command::ReadMeasuredRawValues => (0x03D2, 20),
            Command::StartFanCleaning => (0x5607, 20),
            Command::GetWarmStartParameter => (0x60C6, 20),
            Command::SetWarmStartParameter => (0x60C6, 20),

        }
    }
}