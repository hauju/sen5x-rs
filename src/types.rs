//! Types for the SEN5x sensor.

/// Scaled sensor measurement data (floating point).
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct SensorData {
    /// Mass concentration PM1.0 in µg/m³
    pub mass_concentration_pm1p0: f32,
    /// Mass concentration PM2.5 in µg/m³
    pub mass_concentration_pm2p5: f32,
    /// Mass concentration PM4.0 in µg/m³
    pub mass_concentration_pm4p0: f32,
    /// Mass concentration PM10.0 in µg/m³
    pub mass_concentration_pm10p0: f32,
    /// Relative humidity in %
    pub ambient_humidity: f32,
    /// Temperature in °C
    pub ambient_temperature: f32,
    /// VOC index (1–500)
    pub voc_index: f32,
    /// NOx index (1–500)
    pub nox_index: f32,
}

/// Raw integer sensor data (ticks, before scaling).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct RawSensorData {
    pub mass_concentration_pm1p0: u16,
    pub mass_concentration_pm2p5: u16,
    pub mass_concentration_pm4p0: u16,
    pub mass_concentration_pm10p0: u16,
    pub ambient_humidity: i16,
    pub ambient_temperature: i16,
    pub voc_index: i16,
    pub nox_index: i16,
}

/// Raw unscaled sensor ticks from ReadMeasuredRawValues.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct RawMeasurementValues {
    pub raw_humidity: i16,
    pub raw_temperature: i16,
    pub raw_voc: u16,
    pub raw_nox: u16,
}

/// Extended PM values including number concentrations and typical particle size.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct PmValues {
    /// Mass concentration PM1.0 in µg/m³
    pub mass_pm1p0: f32,
    /// Mass concentration PM2.5 in µg/m³
    pub mass_pm2p5: f32,
    /// Mass concentration PM4.0 in µg/m³
    pub mass_pm4p0: f32,
    /// Mass concentration PM10.0 in µg/m³
    pub mass_pm10p0: f32,
    /// Number concentration PM0.5 in #/cm³
    pub number_pm0p5: f32,
    /// Number concentration PM1.0 in #/cm³
    pub number_pm1p0: f32,
    /// Number concentration PM2.5 in #/cm³
    pub number_pm2p5: f32,
    /// Number concentration PM4.0 in #/cm³
    pub number_pm4p0: f32,
    /// Number concentration PM10.0 in #/cm³
    pub number_pm10p0: f32,
    /// Typical particle size in µm
    pub typical_particle_size: f32,
}

/// Temperature compensation parameters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct TemperatureOffsetParameters {
    /// Constant temperature offset scaled by 200 (T [°C] = offset / 200)
    pub offset: i16,
    /// Normalized temperature offset slope (factor = slope / 10000)
    pub slope: i16,
    /// Time constant in seconds (0 = immediate)
    pub time_constant: u16,
}

/// VOC/NOx algorithm tuning parameters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct AlgorithmTuningParameters {
    /// Index representing "typical" conditions (range 1–250)
    pub index_offset: i16,
    /// Learning time for offset estimation in hours (range 1–1000)
    pub learning_time_offset_hours: i16,
    /// Learning time for gain estimation in hours (range 1–1000)
    pub learning_time_gain_hours: i16,
    /// Maximum gating duration in minutes (0 = disabled, range 0–3000)
    pub gating_max_duration_minutes: i16,
    /// Initial estimate for standard deviation
    pub std_initial: i16,
    /// Gain factor to amplify/attenuate output (range 1–1000)
    pub gain_factor: i16,
}

/// Firmware, hardware, and protocol version information.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct VersionInfo {
    pub firmware_major: u8,
    pub firmware_minor: u8,
    pub firmware_debug: bool,
    pub hardware_major: u8,
    pub hardware_minor: u8,
    pub protocol_major: u8,
    pub protocol_minor: u8,
}
