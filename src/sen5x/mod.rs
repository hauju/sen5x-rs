use embedded_hal as hal;
use hal::delay::DelayNs;
use hal::i2c::I2c;

use crate::commands::Command;
use crate::error::Error;
use crate::types::{
    AlgorithmTuningParameters, PmValues, RawMeasurementValues, RawSensorData, SensorData,
    TemperatureOffsetParameters, VersionInfo,
};
use sensirion_i2c::{crc8, i2c};

const SEN5X_I2C_ADDRESS: u8 = 0x69;

#[cfg(feature = "embedded-hal-async")]
mod async_impl;
#[cfg(feature = "embedded-hal-async")]
pub use async_impl::Sen5xAsync;

/// SEN5x sensor instance. Use related methods to take measurements.
#[derive(Debug, Default)]
pub struct Sen5x<I2C, D> {
    i2c: I2C,
    delay: D,
    is_running: bool,
}

impl<I2C, D, E> Sen5x<I2C, D>
where
    I2C: I2c<Error = E>,
    D: DelayNs,
{
    /// Create a new SEN5x driver instance.
    pub fn new(i2c: I2C, delay: D) -> Self {
        Sen5x {
            i2c,
            delay,
            is_running: false,
        }
    }

    /// Destroy the driver and return the I2C bus.
    pub fn destroy(self) -> I2C {
        self.i2c
    }

    /// Start periodic measurement, signal update interval is 1 second.
    pub fn start_measurement(&mut self) -> Result<(), Error<E>> {
        self.write_command(Command::StartMeasurement)?;
        self.is_running = true;
        Ok(())
    }

    /// Start periodic measurement without PM (low-power mode, RH/T/VOC/NOx only).
    pub fn start_measurement_without_pm(&mut self) -> Result<(), Error<E>> {
        self.write_command(Command::StartMeasurementWithoutPm)?;
        self.is_running = true;
        Ok(())
    }

    /// Stop periodic measurement and return to idle mode.
    pub fn stop_measurement(&mut self) -> Result<(), Error<E>> {
        self.write_command(Command::StopMeasurement)?;
        self.is_running = false;
        Ok(())
    }

    /// Check whether new measurement data is available for read-out.
    pub fn data_ready(&mut self) -> Result<bool, Error<E>> {
        let mut buf = [0; 3];
        self.delayed_read_cmd(Command::ReadDataReady, &mut buf)?;
        let status = u16::from_be_bytes([buf[0], buf[1]]);
        Ok((status & 0x07FF) != 0)
    }

    /// Read converted sensor data (f32 scaled values).
    pub fn measurement(&mut self) -> Result<SensorData, Error<E>> {
        let raw = self.read_measured_values_as_integers()?;
        Ok(SensorData::from_raw(raw))
    }

    /// Read measured values as raw integer ticks.
    pub fn read_measured_values_as_integers(&mut self) -> Result<RawSensorData, Error<E>> {
        let mut buf = [0; 24];
        self.delayed_read_cmd(Command::ReadMeasuredValuesAsIntegers, &mut buf)?;

        Ok(RawSensorData {
            mass_concentration_pm1p0: u16::from_be_bytes([buf[0], buf[1]]),
            mass_concentration_pm2p5: u16::from_be_bytes([buf[3], buf[4]]),
            mass_concentration_pm4p0: u16::from_be_bytes([buf[6], buf[7]]),
            mass_concentration_pm10p0: u16::from_be_bytes([buf[9], buf[10]]),
            ambient_humidity: i16::from_be_bytes([buf[12], buf[13]]),
            ambient_temperature: i16::from_be_bytes([buf[15], buf[16]]),
            voc_index: i16::from_be_bytes([buf[18], buf[19]]),
            nox_index: i16::from_be_bytes([buf[21], buf[22]]),
        })
    }

    /// Read measured raw sensor ticks (unscaled humidity, temperature, VOC, NOx).
    pub fn read_measured_raw_values(&mut self) -> Result<RawMeasurementValues, Error<E>> {
        let mut buf = [0; 12];
        self.delayed_read_cmd(Command::ReadMeasuredRawValues, &mut buf)?;

        Ok(RawMeasurementValues {
            raw_humidity: i16::from_be_bytes([buf[0], buf[1]]),
            raw_temperature: i16::from_be_bytes([buf[3], buf[4]]),
            raw_voc: u16::from_be_bytes([buf[6], buf[7]]),
            raw_nox: u16::from_be_bytes([buf[9], buf[10]]),
        })
    }

    /// Read extended PM values including number concentrations and typical particle size.
    pub fn read_measured_pm_values(&mut self) -> Result<PmValues, Error<E>> {
        let mut buf = [0; 30];
        self.delayed_read_cmd(Command::ReadMeasuredPmValues, &mut buf)?;

        Ok(PmValues {
            mass_pm1p0: u16::from_be_bytes([buf[0], buf[1]]) as f32 / 10.0,
            mass_pm2p5: u16::from_be_bytes([buf[3], buf[4]]) as f32 / 10.0,
            mass_pm4p0: u16::from_be_bytes([buf[6], buf[7]]) as f32 / 10.0,
            mass_pm10p0: u16::from_be_bytes([buf[9], buf[10]]) as f32 / 10.0,
            number_pm0p5: u16::from_be_bytes([buf[12], buf[13]]) as f32 / 10.0,
            number_pm1p0: u16::from_be_bytes([buf[15], buf[16]]) as f32 / 10.0,
            number_pm2p5: u16::from_be_bytes([buf[18], buf[19]]) as f32 / 10.0,
            number_pm4p0: u16::from_be_bytes([buf[21], buf[22]]) as f32 / 10.0,
            number_pm10p0: u16::from_be_bytes([buf[24], buf[25]]) as f32 / 10.0,
            typical_particle_size: u16::from_be_bytes([buf[27], buf[28]]) as f32 / 1000.0,
        })
    }

    /// Start fan cleaning manually. Only available during measurement with PM.
    pub fn start_fan_cleaning(&mut self) -> Result<(), Error<E>> {
        self.write_command(Command::StartFanCleaning)?;
        Ok(())
    }

    /// Get fan auto cleaning interval in seconds. 0 means auto cleaning is disabled.
    pub fn fan_auto_cleaning_interval(&mut self) -> Result<u32, Error<E>> {
        let mut buf = [0; 6];
        self.delayed_read_cmd(Command::GetFanAutoCleaningInterval, &mut buf)?;
        let hi = u16::from_be_bytes([buf[0], buf[1]]);
        let lo = u16::from_be_bytes([buf[3], buf[4]]);
        Ok((u32::from(hi) << 16) | u32::from(lo))
    }

    /// Set fan auto cleaning interval in seconds. Set to 0 to disable auto cleaning.
    pub fn set_fan_auto_cleaning_interval(&mut self, seconds: u32) -> Result<(), Error<E>> {
        let hi = (seconds >> 16) as u16;
        let lo = seconds as u16;
        self.write_command_with_words(Command::SetFanAutoCleaningInterval, &[hi, lo])?;
        Ok(())
    }

    /// Get temperature compensation parameters.
    pub fn temperature_offset_parameters(
        &mut self,
    ) -> Result<TemperatureOffsetParameters, Error<E>> {
        let mut buf = [0; 9];
        self.delayed_read_cmd(Command::GetTemperatureOffsetParameters, &mut buf)?;
        Ok(TemperatureOffsetParameters {
            offset: i16::from_be_bytes([buf[0], buf[1]]),
            slope: i16::from_be_bytes([buf[3], buf[4]]),
            time_constant: u16::from_be_bytes([buf[6], buf[7]]),
        })
    }

    /// Set temperature compensation parameters (offset, slope, time constant).
    pub fn set_temperature_offset_parameters(
        &mut self,
        params: &TemperatureOffsetParameters,
    ) -> Result<(), Error<E>> {
        self.write_command_with_words(
            Command::SetTemperatureOffsetParameters,
            &[
                params.offset as u16,
                params.slope as u16,
                params.time_constant,
            ],
        )?;
        Ok(())
    }

    /// Set a simple temperature offset in °C (slope=0, time_constant=0).
    pub fn set_temperature_offset_simple(&mut self, temp_offset: f32) -> Result<(), Error<E>> {
        let offset_ticks = (temp_offset * 200.0) as i16;
        self.write_command_with_words(
            Command::SetTemperatureOffsetParameters,
            &[offset_ticks as u16, 0, 0],
        )?;
        Ok(())
    }

    /// Get warm start parameter.
    pub fn warm_start_parameter(&mut self) -> Result<u16, Error<E>> {
        let mut buf = [0; 3];
        self.delayed_read_cmd(Command::GetWarmStartParameter, &mut buf)?;
        Ok(u16::from_be_bytes([buf[0], buf[1]]))
    }

    /// Set warm start parameter (0–65535). Applied at next measurement start.
    pub fn set_warm_start_parameter(&mut self, warm_start: u16) -> Result<(), Error<E>> {
        self.write_command_with_data(Command::SetWarmStartParameter, warm_start)?;
        Ok(())
    }

    /// Get VOC algorithm tuning parameters.
    pub fn voc_algorithm_tuning(&mut self) -> Result<AlgorithmTuningParameters, Error<E>> {
        let mut buf = [0; 18];
        self.delayed_read_cmd(Command::GetVocAlgorithmTuningParameters, &mut buf)?;
        Ok(algorithm_tuning_from_buf(&buf))
    }

    /// Set VOC algorithm tuning parameters.
    pub fn set_voc_algorithm_tuning(
        &mut self,
        params: &AlgorithmTuningParameters,
    ) -> Result<(), Error<E>> {
        self.write_command_with_words(
            Command::SetVocAlgorithmTuningParameters,
            &algorithm_tuning_to_words(params),
        )?;
        Ok(())
    }

    /// Get NOx algorithm tuning parameters (SEN55 only).
    pub fn nox_algorithm_tuning(&mut self) -> Result<AlgorithmTuningParameters, Error<E>> {
        let mut buf = [0; 18];
        self.delayed_read_cmd(Command::GetNoxAlgorithmTuningParameters, &mut buf)?;
        Ok(algorithm_tuning_from_buf(&buf))
    }

    /// Set NOx algorithm tuning parameters (SEN55 only).
    pub fn set_nox_algorithm_tuning(
        &mut self,
        params: &AlgorithmTuningParameters,
    ) -> Result<(), Error<E>> {
        self.write_command_with_words(
            Command::SetNoxAlgorithmTuningParameters,
            &algorithm_tuning_to_words(params),
        )?;
        Ok(())
    }

    /// Get RH/T acceleration mode.
    pub fn rht_acceleration_mode(&mut self) -> Result<u16, Error<E>> {
        let mut buf = [0; 3];
        self.delayed_read_cmd(Command::GetRhtAccelerationMode, &mut buf)?;
        Ok(u16::from_be_bytes([buf[0], buf[1]]))
    }

    /// Set RH/T acceleration mode. Applied at next measurement start.
    pub fn set_rht_acceleration_mode(&mut self, mode: u16) -> Result<(), Error<E>> {
        self.write_command_with_data(Command::SetRhtAccelerationMode, mode)?;
        Ok(())
    }

    /// Get VOC algorithm state for backup (8 bytes as 4 u16 words).
    pub fn voc_algorithm_state(&mut self) -> Result<[u16; 4], Error<E>> {
        let mut buf = [0; 12];
        self.delayed_read_cmd(Command::GetVocAlgorithmState, &mut buf)?;
        Ok([
            u16::from_be_bytes([buf[0], buf[1]]),
            u16::from_be_bytes([buf[3], buf[4]]),
            u16::from_be_bytes([buf[6], buf[7]]),
            u16::from_be_bytes([buf[9], buf[10]]),
        ])
    }

    /// Set VOC algorithm state for warm start after power cycle.
    pub fn set_voc_algorithm_state(&mut self, state: &[u16; 4]) -> Result<(), Error<E>> {
        self.write_command_with_words(Command::SetVocAlgorithmState, state)?;
        Ok(())
    }

    /// Get product name as ASCII bytes (null-terminated).
    pub fn product_name(&mut self) -> Result<[u8; 32], Error<E>> {
        let mut buf = [0; 48];
        self.delayed_read_cmd(Command::GetProductName, &mut buf)?;
        Ok(compact_ascii_words(&buf))
    }

    /// Get serial number as ASCII bytes (null-terminated).
    pub fn serial_number(&mut self) -> Result<[u8; 32], Error<E>> {
        let mut buf = [0; 48];
        self.delayed_read_cmd(Command::GetSerialNumber, &mut buf)?;
        Ok(compact_ascii_words(&buf))
    }

    /// Get firmware, hardware, and protocol version information.
    pub fn version(&mut self) -> Result<VersionInfo, Error<E>> {
        let mut buf = [0; 12];
        self.delayed_read_cmd(Command::GetVersion, &mut buf)?;
        Ok(VersionInfo {
            firmware_major: buf[0],
            firmware_minor: buf[1],
            firmware_debug: buf[3] != 0,
            hardware_major: buf[4],
            hardware_minor: buf[6],
            protocol_major: buf[7],
            protocol_minor: buf[9],
        })
    }

    /// Read device status register.
    pub fn device_status(&mut self) -> Result<u32, Error<E>> {
        let mut buf = [0; 6];
        self.delayed_read_cmd(Command::ReadDeviceStatus, &mut buf)?;
        let hi = u16::from_be_bytes([buf[0], buf[1]]);
        let lo = u16::from_be_bytes([buf[3], buf[4]]);
        Ok((u32::from(hi) << 16) | u32::from(lo))
    }

    /// Read and clear device status register.
    pub fn read_and_clear_device_status(&mut self) -> Result<u32, Error<E>> {
        let mut buf = [0; 6];
        self.delayed_read_cmd(Command::ReadAndClearDeviceStatus, &mut buf)?;
        let hi = u16::from_be_bytes([buf[0], buf[1]]);
        let lo = u16::from_be_bytes([buf[3], buf[4]]);
        Ok((u32::from(hi) << 16) | u32::from(lo))
    }

    /// Reset the sensor (equivalent to power cycle).
    pub fn device_reset(&mut self) -> Result<(), Error<E>> {
        self.write_command(Command::DeviceReset)?;
        self.is_running = false;
        Ok(())
    }

    // --- Internal I2C helpers ---

    /// Write command then read response with CRC validation.
    fn delayed_read_cmd(&mut self, cmd: Command, data: &mut [u8]) -> Result<(), Error<E>> {
        self.write_command(cmd)?;
        i2c::read_words_with_crc(&mut self.i2c, SEN5X_I2C_ADDRESS, data)?;
        Ok(())
    }

    /// Write a command without additional data.
    fn write_command(&mut self, cmd: Command) -> Result<(), Error<E>> {
        let (command, delay, allowed_if_running) = cmd.as_tuple();
        if !allowed_if_running && self.is_running {
            return Err(Error::NotAllowed);
        }
        i2c::write_command_u16(&mut self.i2c, SEN5X_I2C_ADDRESS, command).map_err(Error::I2c)?;
        self.delay.delay_ms(delay);
        Ok(())
    }

    /// Write a command with a single u16 data word (+ CRC).
    fn write_command_with_data(&mut self, cmd: Command, data: u16) -> Result<(), Error<E>> {
        let (command, delay, allowed_if_running) = cmd.as_tuple();
        if !allowed_if_running && self.is_running {
            return Err(Error::NotAllowed);
        }
        let buf = encode_cmd_with_data(command, data);
        self.i2c
            .write(SEN5X_I2C_ADDRESS, &buf)
            .map_err(Error::I2c)?;
        self.delay.delay_ms(delay);
        Ok(())
    }

    /// Write a command with multiple u16 data words (each followed by CRC).
    fn write_command_with_words(&mut self, cmd: Command, words: &[u16]) -> Result<(), Error<E>> {
        let (command, delay, allowed_if_running) = cmd.as_tuple();
        if !allowed_if_running && self.is_running {
            return Err(Error::NotAllowed);
        }
        let mut buf = [0u8; 20]; // max: 2 cmd + 6 words * 3 = 20
        let len = encode_cmd_with_words(command, words, &mut buf);
        self.i2c
            .write(SEN5X_I2C_ADDRESS, &buf[..len])
            .map_err(Error::I2c)?;
        self.delay.delay_ms(delay);
        Ok(())
    }
}

impl SensorData {
    fn from_raw(raw: RawSensorData) -> Self {
        SensorData {
            mass_concentration_pm1p0: raw.mass_concentration_pm1p0 as f32 / 10.0,
            mass_concentration_pm2p5: raw.mass_concentration_pm2p5 as f32 / 10.0,
            mass_concentration_pm4p0: raw.mass_concentration_pm4p0 as f32 / 10.0,
            mass_concentration_pm10p0: raw.mass_concentration_pm10p0 as f32 / 10.0,
            ambient_humidity: raw.ambient_humidity as f32 / 100.0,
            ambient_temperature: raw.ambient_temperature as f32 / 200.0,
            voc_index: raw.voc_index as f32 / 10.0,
            nox_index: raw.nox_index as f32 / 10.0,
        }
    }
}

fn encode_cmd_with_data(command: u16, data: u16) -> [u8; 5] {
    let c = command.to_be_bytes();
    let d = data.to_be_bytes();
    let mut buf = [0; 5];
    buf[0..2].copy_from_slice(&c);
    buf[2..4].copy_from_slice(&d);
    buf[4] = crc8::calculate(&d);
    buf
}

fn encode_cmd_with_words(command: u16, words: &[u16], buf: &mut [u8]) -> usize {
    let c = command.to_be_bytes();
    buf[0] = c[0];
    buf[1] = c[1];
    let mut offset = 2;
    for &word in words {
        let d = word.to_be_bytes();
        buf[offset] = d[0];
        buf[offset + 1] = d[1];
        buf[offset + 2] = crc8::calculate(&d);
        offset += 3;
    }
    offset
}

/// Compact 16-word CRC response (48 bytes) into 32 ASCII data bytes.
/// The sensirion-i2c crate's `read_words_with_crc` leaves the buffer in
/// [d0, d1, crc, d2, d3, crc, ...] format. Extract data bytes only.
fn compact_ascii_words(buf: &[u8; 48]) -> [u8; 32] {
    let mut result = [0u8; 32];
    for i in 0..16 {
        result[i * 2] = buf[i * 3];
        result[i * 2 + 1] = buf[i * 3 + 1];
    }
    result
}

fn algorithm_tuning_from_buf(buf: &[u8; 18]) -> AlgorithmTuningParameters {
    AlgorithmTuningParameters {
        index_offset: i16::from_be_bytes([buf[0], buf[1]]),
        learning_time_offset_hours: i16::from_be_bytes([buf[3], buf[4]]),
        learning_time_gain_hours: i16::from_be_bytes([buf[6], buf[7]]),
        gating_max_duration_minutes: i16::from_be_bytes([buf[9], buf[10]]),
        std_initial: i16::from_be_bytes([buf[12], buf[13]]),
        gain_factor: i16::from_be_bytes([buf[15], buf[16]]),
    }
}

fn algorithm_tuning_to_words(params: &AlgorithmTuningParameters) -> [u16; 6] {
    [
        params.index_offset as u16,
        params.learning_time_offset_hours as u16,
        params.learning_time_gain_hours as u16,
        params.gating_max_duration_minutes as u16,
        params.std_initial as u16,
        params.gain_factor as u16,
    ]
}

#[cfg(test)]
mod tests {
    use embedded_hal_mock::eh1 as hal;

    use self::hal::delay::NoopDelay as DelayMock;
    use self::hal::i2c::{Mock as I2cMock, Transaction};
    use super::*;

    /// Build a 3-byte response word: [msb, lsb, crc]
    fn word(msb: u8, lsb: u8) -> [u8; 3] {
        [msb, lsb, crc8::calculate(&[msb, lsb])]
    }

    #[test]
    fn test_get_serial_number() {
        let (cmd, _, _) = Command::GetSerialNumber.as_tuple();
        let expectations = [
            Transaction::write(SEN5X_I2C_ADDRESS, cmd.to_be_bytes().to_vec()),
            Transaction::read(
                SEN5X_I2C_ADDRESS,
                [
                    word(b'A', b'B'),
                    word(b'C', b'D'),
                    word(0, 0),
                    word(0, 0),
                    word(0, 0),
                    word(0, 0),
                    word(0, 0),
                    word(0, 0),
                    word(0, 0),
                    word(0, 0),
                    word(0, 0),
                    word(0, 0),
                    word(0, 0),
                    word(0, 0),
                    word(0, 0),
                    word(0, 0),
                ]
                .concat(),
            ),
        ];
        let mock = I2cMock::new(&expectations);
        let mut sensor = Sen5x::new(mock, DelayMock);

        let serial = sensor.serial_number().unwrap();
        assert_eq!(&serial[..4], b"ABCD");
        assert_eq!(serial[4], 0);

        let mut mock = sensor.destroy();
        mock.done();
    }

    #[test]
    fn test_measurement() {
        let (cmd, _, _) = Command::ReadMeasuredValuesAsIntegers.as_tuple();
        let expectations = [
            Transaction::write(SEN5X_I2C_ADDRESS, cmd.to_be_bytes().to_vec()),
            Transaction::read(
                SEN5X_I2C_ADDRESS,
                [
                    word(0x00, 0x64), // PM1.0 = 100
                    word(0x00, 0xC8), // PM2.5 = 200
                    word(0x01, 0x2C), // PM4.0 = 300
                    word(0x01, 0x90), // PM10 = 400
                    word(0x13, 0x88), // Humidity = 5000
                    word(0x13, 0x88), // Temperature = 5000
                    word(0x00, 0x64), // VOC = 100
                    word(0x00, 0x32), // NOx = 50
                ]
                .concat(),
            ),
        ];
        let mock = I2cMock::new(&expectations);
        let mut sensor = Sen5x::new(mock, DelayMock);

        let data = sensor.measurement().unwrap();
        assert_eq!(data.mass_concentration_pm1p0, 10.0);
        assert_eq!(data.mass_concentration_pm2p5, 20.0);
        assert_eq!(data.mass_concentration_pm4p0, 30.0);
        assert_eq!(data.mass_concentration_pm10p0, 40.0);
        assert_eq!(data.ambient_humidity, 50.0);
        assert_eq!(data.ambient_temperature, 25.0);
        assert_eq!(data.voc_index, 10.0);
        assert_eq!(data.nox_index, 5.0);

        let mut mock = sensor.destroy();
        mock.done();
    }

    #[test]
    fn test_data_ready() {
        let (cmd, _, _) = Command::ReadDataReady.as_tuple();
        let w = word(0x00, 0x01);
        let expectations = [
            Transaction::write(SEN5X_I2C_ADDRESS, cmd.to_be_bytes().to_vec()),
            Transaction::read(SEN5X_I2C_ADDRESS, w.to_vec()),
        ];
        let mock = I2cMock::new(&expectations);
        let mut sensor = Sen5x::new(mock, DelayMock);

        assert!(sensor.data_ready().unwrap());

        let mut mock = sensor.destroy();
        mock.done();
    }

    #[test]
    fn test_data_not_ready() {
        let (cmd, _, _) = Command::ReadDataReady.as_tuple();
        let w = word(0xF8, 0x00);
        let expectations = [
            Transaction::write(SEN5X_I2C_ADDRESS, cmd.to_be_bytes().to_vec()),
            Transaction::read(SEN5X_I2C_ADDRESS, w.to_vec()),
        ];
        let mock = I2cMock::new(&expectations);
        let mut sensor = Sen5x::new(mock, DelayMock);

        assert!(!sensor.data_ready().unwrap());

        let mut mock = sensor.destroy();
        mock.done();
    }

    #[test]
    fn test_version() {
        let (cmd, _, _) = Command::GetVersion.as_tuple();
        let expectations = [
            Transaction::write(SEN5X_I2C_ADDRESS, cmd.to_be_bytes().to_vec()),
            Transaction::read(
                SEN5X_I2C_ADDRESS,
                [
                    word(0x02, 0x01), // fw_major=2, fw_minor=1
                    word(0x00, 0x03), // fw_debug=false, hw_major=3
                    word(0x00, 0x01), // hw_minor=0, proto_major=1
                    word(0x00, 0x00), // proto_minor=0, padding
                ]
                .concat(),
            ),
        ];
        let mock = I2cMock::new(&expectations);
        let mut sensor = Sen5x::new(mock, DelayMock);

        let v = sensor.version().unwrap();
        assert_eq!(v.firmware_major, 2);
        assert_eq!(v.firmware_minor, 1);
        assert!(!v.firmware_debug);
        assert_eq!(v.hardware_major, 3);
        assert_eq!(v.hardware_minor, 0);
        assert_eq!(v.protocol_major, 1);
        assert_eq!(v.protocol_minor, 0);

        let mut mock = sensor.destroy();
        mock.done();
    }

    #[test]
    fn test_not_allowed_when_running() {
        let (start_cmd, _, _) = Command::StartMeasurement.as_tuple();
        let expectations = [Transaction::write(
            SEN5X_I2C_ADDRESS,
            start_cmd.to_be_bytes().to_vec(),
        )];
        let mock = I2cMock::new(&expectations);
        let mut sensor = Sen5x::new(mock, DelayMock);

        sensor.start_measurement().unwrap();

        // These commands have allowed_if_running=false
        assert_eq!(sensor.serial_number(), Err(Error::NotAllowed));
        assert_eq!(sensor.version(), Err(Error::NotAllowed));
        assert_eq!(sensor.product_name(), Err(Error::NotAllowed));
        assert_eq!(sensor.set_warm_start_parameter(100), Err(Error::NotAllowed));

        let mut mock = sensor.destroy();
        mock.done();
    }

    #[test]
    fn test_allowed_when_running() {
        let (start_cmd, _, _) = Command::StartMeasurement.as_tuple();
        let (ready_cmd, _, _) = Command::ReadDataReady.as_tuple();
        let (meas_cmd, _, _) = Command::ReadMeasuredValuesAsIntegers.as_tuple();
        let ready_word = word(0x00, 0x01);
        let expectations = [
            Transaction::write(SEN5X_I2C_ADDRESS, start_cmd.to_be_bytes().to_vec()),
            Transaction::write(SEN5X_I2C_ADDRESS, ready_cmd.to_be_bytes().to_vec()),
            Transaction::read(SEN5X_I2C_ADDRESS, ready_word.to_vec()),
            Transaction::write(SEN5X_I2C_ADDRESS, meas_cmd.to_be_bytes().to_vec()),
            Transaction::read(
                SEN5X_I2C_ADDRESS,
                [
                    word(0x00, 0x64),
                    word(0x00, 0xC8),
                    word(0x01, 0x2C),
                    word(0x01, 0x90),
                    word(0x13, 0x88),
                    word(0x13, 0x88),
                    word(0x00, 0x64),
                    word(0x00, 0x32),
                ]
                .concat(),
            ),
        ];
        let mock = I2cMock::new(&expectations);
        let mut sensor = Sen5x::new(mock, DelayMock);

        sensor.start_measurement().unwrap();
        assert!(sensor.data_ready().unwrap());
        let data = sensor.measurement().unwrap();
        assert_eq!(data.mass_concentration_pm1p0, 10.0);

        let mut mock = sensor.destroy();
        mock.done();
    }

    #[test]
    fn test_stop_clears_running() {
        let (start_cmd, _, _) = Command::StartMeasurement.as_tuple();
        let (stop_cmd, _, _) = Command::StopMeasurement.as_tuple();
        let (serial_cmd, _, _) = Command::GetSerialNumber.as_tuple();
        let expectations = [
            Transaction::write(SEN5X_I2C_ADDRESS, start_cmd.to_be_bytes().to_vec()),
            Transaction::write(SEN5X_I2C_ADDRESS, stop_cmd.to_be_bytes().to_vec()),
            Transaction::write(SEN5X_I2C_ADDRESS, serial_cmd.to_be_bytes().to_vec()),
            Transaction::read(
                SEN5X_I2C_ADDRESS,
                [
                    word(b'T', b'E'),
                    word(b'S', b'T'),
                    word(0, 0),
                    word(0, 0),
                    word(0, 0),
                    word(0, 0),
                    word(0, 0),
                    word(0, 0),
                    word(0, 0),
                    word(0, 0),
                    word(0, 0),
                    word(0, 0),
                    word(0, 0),
                    word(0, 0),
                    word(0, 0),
                    word(0, 0),
                ]
                .concat(),
            ),
        ];
        let mock = I2cMock::new(&expectations);
        let mut sensor = Sen5x::new(mock, DelayMock);

        sensor.start_measurement().unwrap();
        assert_eq!(sensor.serial_number(), Err(Error::NotAllowed));
        sensor.stop_measurement().unwrap();
        assert!(sensor.serial_number().is_ok());

        let mut mock = sensor.destroy();
        mock.done();
    }

    #[test]
    fn test_device_reset_clears_running() {
        let (start_cmd, _, _) = Command::StartMeasurement.as_tuple();
        let (reset_cmd, _, _) = Command::DeviceReset.as_tuple();
        let (serial_cmd, _, _) = Command::GetSerialNumber.as_tuple();
        let expectations = [
            Transaction::write(SEN5X_I2C_ADDRESS, start_cmd.to_be_bytes().to_vec()),
            Transaction::write(SEN5X_I2C_ADDRESS, reset_cmd.to_be_bytes().to_vec()),
            Transaction::write(SEN5X_I2C_ADDRESS, serial_cmd.to_be_bytes().to_vec()),
            Transaction::read(
                SEN5X_I2C_ADDRESS,
                [
                    word(b'S', b'N'),
                    word(0, 0),
                    word(0, 0),
                    word(0, 0),
                    word(0, 0),
                    word(0, 0),
                    word(0, 0),
                    word(0, 0),
                    word(0, 0),
                    word(0, 0),
                    word(0, 0),
                    word(0, 0),
                    word(0, 0),
                    word(0, 0),
                    word(0, 0),
                    word(0, 0),
                ]
                .concat(),
            ),
        ];
        let mock = I2cMock::new(&expectations);
        let mut sensor = Sen5x::new(mock, DelayMock);

        sensor.start_measurement().unwrap();
        assert_eq!(sensor.serial_number(), Err(Error::NotAllowed));
        sensor.device_reset().unwrap();
        assert!(sensor.serial_number().is_ok());

        let mut mock = sensor.destroy();
        mock.done();
    }

    #[test]
    fn test_warm_start_parameter() {
        let (cmd, _, _) = Command::GetWarmStartParameter.as_tuple();
        let w = word(0x00, 0xC8);
        let expectations = [
            Transaction::write(SEN5X_I2C_ADDRESS, cmd.to_be_bytes().to_vec()),
            Transaction::read(SEN5X_I2C_ADDRESS, w.to_vec()),
        ];
        let mock = I2cMock::new(&expectations);
        let mut sensor = Sen5x::new(mock, DelayMock);

        assert_eq!(sensor.warm_start_parameter().unwrap(), 200);

        let mut mock = sensor.destroy();
        mock.done();
    }

    #[test]
    fn test_device_status() {
        let (cmd, _, _) = Command::ReadDeviceStatus.as_tuple();
        let expectations = [
            Transaction::write(SEN5X_I2C_ADDRESS, cmd.to_be_bytes().to_vec()),
            Transaction::read(
                SEN5X_I2C_ADDRESS,
                [word(0x00, 0x21), word(0x00, 0x00)].concat(),
            ),
        ];
        let mock = I2cMock::new(&expectations);
        let mut sensor = Sen5x::new(mock, DelayMock);

        assert_eq!(sensor.device_status().unwrap(), 0x00210000);

        let mut mock = sensor.destroy();
        mock.done();
    }

    #[test]
    fn test_fan_auto_cleaning_interval() {
        let (cmd, _, _) = Command::GetFanAutoCleaningInterval.as_tuple();
        let expectations = [
            Transaction::write(SEN5X_I2C_ADDRESS, cmd.to_be_bytes().to_vec()),
            Transaction::read(
                SEN5X_I2C_ADDRESS,
                [word(0x00, 0x09), word(0x3A, 0x80)].concat(),
            ),
        ];
        let mock = I2cMock::new(&expectations);
        let mut sensor = Sen5x::new(mock, DelayMock);

        assert_eq!(sensor.fan_auto_cleaning_interval().unwrap(), 604800);

        let mut mock = sensor.destroy();
        mock.done();
    }
}
