
#[derive(Debug)]
pub struct SensorData {
    /// Mass concentration PM1.0 in μg/m3
    pub mass_concentration_pm1p0: f32,
    /// Mass concentration PM2.5 in μg/m3
    pub mass_concentration_pm2p5: f32,
    /// Mass concentration PM4.0 in μg/m3
    pub mass_concentration_pm4p0: f32,
    /// Mass concentration PM10.0 in μg/m3
    pub mass_concentration_pm10p0: f32,
    /// Relative humidity in %
    pub ambient_humidity: f32,
    /// Temperature in °C
    pub ambient_temperature: f32,
    /// VOC index
    pub voc_index: f32,
    /// NOx index
    pub nox_index: f32,
}

pub struct SensorDataInt {
    pub mass_concentration_pm1p0: u16,
    pub mass_concentration_pm2p5: u16,
    pub mass_concentration_pm4p0: u16,
    pub mass_concentration_pm10p0: u16,
    pub ambient_humidity: i16,
    pub ambient_temperature: i16,
    pub voc_index: i16,
    pub nox_index: i16,
}

pub struct SensorDataRaw {
    pub raw_humidity: i16,
    pub raw_temperature: i16,
    pub raw_voc: u16,
    pub raw_nox: u16,
}

pub struct ProductName {
    pub name: &'static str,
}

pub struct VersionInfo {
    pub firmware_major: u8,
    pub firmware_minor: u8,
    pub firmware_debug: bool,
    pub hardware_major: u8,
    pub hardware_minor: u8,
    pub protocol_major: u8,
    pub protocol_minor: u8,
}