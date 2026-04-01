use super::*;
use async_hal::{delay::DelayNs, i2c::I2c};
use embedded_hal_async as async_hal;
use sensirion_i2c::i2c_async;

/// Asynchronous SEN5x sensor instance, for use with [`embedded_hal_async`].
///
/// Use related methods to take measurements.
#[derive(Debug, Default)]
pub struct Sen5xAsync<I2C, D> {
    i2c: I2C,
    delay: D,
    is_running: bool,
}

impl<I2C, D, E> Sen5xAsync<I2C, D>
where
    I2C: I2c<Error = E>,
    D: DelayNs,
{
    pub fn new(i2c: I2C, delay: D) -> Self {
        Self {
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
    pub async fn start_measurement(&mut self) -> Result<(), Error<E>> {
        self.write_command(Command::StartMeasurement).await?;
        self.is_running = true;
        Ok(())
    }

    /// Start periodic measurement without PM (low-power mode, RH/T/VOC/NOx only).
    pub async fn start_measurement_without_pm(&mut self) -> Result<(), Error<E>> {
        self.write_command(Command::StartMeasurementWithoutPm)
            .await?;
        self.is_running = true;
        Ok(())
    }

    /// Stop periodic measurement and return to idle mode.
    pub async fn stop_measurement(&mut self) -> Result<(), Error<E>> {
        self.write_command(Command::StopMeasurement).await?;
        self.is_running = false;
        Ok(())
    }

    /// Check whether new measurement data is available for read-out.
    pub async fn data_ready(&mut self) -> Result<bool, Error<E>> {
        let mut buf = [0; 3];
        self.delayed_read_cmd(Command::ReadDataReady, &mut buf)
            .await?;
        let status = u16::from_be_bytes([buf[0], buf[1]]);
        Ok((status & 0x07FF) != 0)
    }

    /// Read converted sensor data (f32 scaled values).
    pub async fn measurement(&mut self) -> Result<SensorData, Error<E>> {
        let raw = self.read_measured_values_as_integers().await?;
        Ok(SensorData::from_raw(raw))
    }

    /// Read measured values as raw integer ticks.
    pub async fn read_measured_values_as_integers(&mut self) -> Result<RawSensorData, Error<E>> {
        let mut buf = [0; 24];
        self.delayed_read_cmd(Command::ReadMeasuredValuesAsIntegers, &mut buf)
            .await?;

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
    pub async fn read_measured_raw_values(&mut self) -> Result<RawMeasurementValues, Error<E>> {
        let mut buf = [0; 12];
        self.delayed_read_cmd(Command::ReadMeasuredRawValues, &mut buf)
            .await?;

        Ok(RawMeasurementValues {
            raw_humidity: i16::from_be_bytes([buf[0], buf[1]]),
            raw_temperature: i16::from_be_bytes([buf[3], buf[4]]),
            raw_voc: u16::from_be_bytes([buf[6], buf[7]]),
            raw_nox: u16::from_be_bytes([buf[9], buf[10]]),
        })
    }

    /// Read extended PM values including number concentrations and typical particle size.
    pub async fn read_measured_pm_values(&mut self) -> Result<PmValues, Error<E>> {
        let mut buf = [0; 30];
        self.delayed_read_cmd(Command::ReadMeasuredPmValues, &mut buf)
            .await?;

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
    pub async fn start_fan_cleaning(&mut self) -> Result<(), Error<E>> {
        self.write_command(Command::StartFanCleaning).await?;
        Ok(())
    }

    /// Get fan auto cleaning interval in seconds. 0 means auto cleaning is disabled.
    pub async fn fan_auto_cleaning_interval(&mut self) -> Result<u32, Error<E>> {
        let mut buf = [0; 6];
        self.delayed_read_cmd(Command::GetFanAutoCleaningInterval, &mut buf)
            .await?;
        let hi = u16::from_be_bytes([buf[0], buf[1]]);
        let lo = u16::from_be_bytes([buf[3], buf[4]]);
        Ok((u32::from(hi) << 16) | u32::from(lo))
    }

    /// Set fan auto cleaning interval in seconds. Set to 0 to disable auto cleaning.
    pub async fn set_fan_auto_cleaning_interval(&mut self, seconds: u32) -> Result<(), Error<E>> {
        let hi = (seconds >> 16) as u16;
        let lo = seconds as u16;
        self.write_command_with_words(Command::SetFanAutoCleaningInterval, &[hi, lo])
            .await?;
        Ok(())
    }

    /// Get temperature compensation parameters.
    pub async fn temperature_offset_parameters(
        &mut self,
    ) -> Result<TemperatureOffsetParameters, Error<E>> {
        let mut buf = [0; 9];
        self.delayed_read_cmd(Command::GetTemperatureOffsetParameters, &mut buf)
            .await?;
        Ok(TemperatureOffsetParameters {
            offset: i16::from_be_bytes([buf[0], buf[1]]),
            slope: i16::from_be_bytes([buf[3], buf[4]]),
            time_constant: u16::from_be_bytes([buf[6], buf[7]]),
        })
    }

    /// Set temperature compensation parameters (offset, slope, time constant).
    pub async fn set_temperature_offset_parameters(
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
        )
        .await?;
        Ok(())
    }

    /// Set a simple temperature offset in °C (slope=0, time_constant=0).
    pub async fn set_temperature_offset_simple(
        &mut self,
        temp_offset: f32,
    ) -> Result<(), Error<E>> {
        let offset_ticks = (temp_offset * 200.0) as i16;
        self.write_command_with_words(
            Command::SetTemperatureOffsetParameters,
            &[offset_ticks as u16, 0, 0],
        )
        .await?;
        Ok(())
    }

    /// Get warm start parameter.
    pub async fn warm_start_parameter(&mut self) -> Result<u16, Error<E>> {
        let mut buf = [0; 3];
        self.delayed_read_cmd(Command::GetWarmStartParameter, &mut buf)
            .await?;
        Ok(u16::from_be_bytes([buf[0], buf[1]]))
    }

    /// Set warm start parameter (0–65535). Applied at next measurement start.
    pub async fn set_warm_start_parameter(&mut self, warm_start: u16) -> Result<(), Error<E>> {
        self.write_command_with_data(Command::SetWarmStartParameter, warm_start)
            .await?;
        Ok(())
    }

    /// Get VOC algorithm tuning parameters.
    pub async fn voc_algorithm_tuning(&mut self) -> Result<AlgorithmTuningParameters, Error<E>> {
        let mut buf = [0; 18];
        self.delayed_read_cmd(Command::GetVocAlgorithmTuningParameters, &mut buf)
            .await?;
        Ok(algorithm_tuning_from_buf(&buf))
    }

    /// Set VOC algorithm tuning parameters.
    pub async fn set_voc_algorithm_tuning(
        &mut self,
        params: &AlgorithmTuningParameters,
    ) -> Result<(), Error<E>> {
        self.write_command_with_words(
            Command::SetVocAlgorithmTuningParameters,
            &algorithm_tuning_to_words(params),
        )
        .await?;
        Ok(())
    }

    /// Get NOx algorithm tuning parameters (SEN55 only).
    pub async fn nox_algorithm_tuning(&mut self) -> Result<AlgorithmTuningParameters, Error<E>> {
        let mut buf = [0; 18];
        self.delayed_read_cmd(Command::GetNoxAlgorithmTuningParameters, &mut buf)
            .await?;
        Ok(algorithm_tuning_from_buf(&buf))
    }

    /// Set NOx algorithm tuning parameters (SEN55 only).
    pub async fn set_nox_algorithm_tuning(
        &mut self,
        params: &AlgorithmTuningParameters,
    ) -> Result<(), Error<E>> {
        self.write_command_with_words(
            Command::SetNoxAlgorithmTuningParameters,
            &algorithm_tuning_to_words(params),
        )
        .await?;
        Ok(())
    }

    /// Get RH/T acceleration mode.
    pub async fn rht_acceleration_mode(&mut self) -> Result<u16, Error<E>> {
        let mut buf = [0; 3];
        self.delayed_read_cmd(Command::GetRhtAccelerationMode, &mut buf)
            .await?;
        Ok(u16::from_be_bytes([buf[0], buf[1]]))
    }

    /// Set RH/T acceleration mode. Applied at next measurement start.
    pub async fn set_rht_acceleration_mode(&mut self, mode: u16) -> Result<(), Error<E>> {
        self.write_command_with_data(Command::SetRhtAccelerationMode, mode)
            .await?;
        Ok(())
    }

    /// Get VOC algorithm state for backup (8 bytes as 4 u16 words).
    pub async fn voc_algorithm_state(&mut self) -> Result<[u16; 4], Error<E>> {
        let mut buf = [0; 12];
        self.delayed_read_cmd(Command::GetVocAlgorithmState, &mut buf)
            .await?;
        Ok([
            u16::from_be_bytes([buf[0], buf[1]]),
            u16::from_be_bytes([buf[3], buf[4]]),
            u16::from_be_bytes([buf[6], buf[7]]),
            u16::from_be_bytes([buf[9], buf[10]]),
        ])
    }

    /// Set VOC algorithm state for warm start after power cycle.
    pub async fn set_voc_algorithm_state(&mut self, state: &[u16; 4]) -> Result<(), Error<E>> {
        self.write_command_with_words(Command::SetVocAlgorithmState, state)
            .await?;
        Ok(())
    }

    /// Get product name as ASCII bytes (null-terminated).
    pub async fn product_name(&mut self) -> Result<[u8; 32], Error<E>> {
        let mut buf = [0; 48];
        self.delayed_read_cmd(Command::GetProductName, &mut buf)
            .await?;
        Ok(compact_ascii_words(&buf))
    }

    /// Get serial number as ASCII bytes (null-terminated).
    pub async fn serial_number(&mut self) -> Result<[u8; 32], Error<E>> {
        let mut buf = [0; 48];
        self.delayed_read_cmd(Command::GetSerialNumber, &mut buf)
            .await?;
        Ok(compact_ascii_words(&buf))
    }

    /// Get firmware, hardware, and protocol version information.
    pub async fn version(&mut self) -> Result<VersionInfo, Error<E>> {
        let mut buf = [0; 12];
        self.delayed_read_cmd(Command::GetVersion, &mut buf).await?;
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
    pub async fn device_status(&mut self) -> Result<u32, Error<E>> {
        let mut buf = [0; 6];
        self.delayed_read_cmd(Command::ReadDeviceStatus, &mut buf)
            .await?;
        let hi = u16::from_be_bytes([buf[0], buf[1]]);
        let lo = u16::from_be_bytes([buf[3], buf[4]]);
        Ok((u32::from(hi) << 16) | u32::from(lo))
    }

    /// Read and clear device status register.
    pub async fn read_and_clear_device_status(&mut self) -> Result<u32, Error<E>> {
        let mut buf = [0; 6];
        self.delayed_read_cmd(Command::ReadAndClearDeviceStatus, &mut buf)
            .await?;
        let hi = u16::from_be_bytes([buf[0], buf[1]]);
        let lo = u16::from_be_bytes([buf[3], buf[4]]);
        Ok((u32::from(hi) << 16) | u32::from(lo))
    }

    /// Reset the sensor (equivalent to power cycle).
    pub async fn device_reset(&mut self) -> Result<(), Error<E>> {
        self.write_command(Command::DeviceReset).await?;
        self.is_running = false;
        Ok(())
    }

    // --- Internal I2C helpers ---

    async fn delayed_read_cmd(&mut self, cmd: Command, data: &mut [u8]) -> Result<(), Error<E>> {
        self.write_command(cmd).await?;
        i2c_async::read_words_with_crc(&mut self.i2c, SEN5X_I2C_ADDRESS, data).await?;
        Ok(())
    }

    async fn write_command(&mut self, cmd: Command) -> Result<(), Error<E>> {
        let (command, delay, allowed_if_running) = cmd.as_tuple();
        if !allowed_if_running && self.is_running {
            return Err(Error::NotAllowed);
        }
        i2c_async::write_command_u16(&mut self.i2c, SEN5X_I2C_ADDRESS, command)
            .await
            .map_err(Error::I2c)?;
        self.delay.delay_ms(delay).await;
        Ok(())
    }

    async fn write_command_with_data(&mut self, cmd: Command, data: u16) -> Result<(), Error<E>> {
        let (command, delay, allowed_if_running) = cmd.as_tuple();
        if !allowed_if_running && self.is_running {
            return Err(Error::NotAllowed);
        }
        let buf = encode_cmd_with_data(command, data);
        self.i2c
            .write(SEN5X_I2C_ADDRESS, &buf)
            .await
            .map_err(Error::I2c)?;
        self.delay.delay_ms(delay).await;
        Ok(())
    }

    async fn write_command_with_words(
        &mut self,
        cmd: Command,
        words: &[u16],
    ) -> Result<(), Error<E>> {
        let (command, delay, allowed_if_running) = cmd.as_tuple();
        if !allowed_if_running && self.is_running {
            return Err(Error::NotAllowed);
        }
        let mut buf = [0u8; 20];
        let len = encode_cmd_with_words(command, words, &mut buf);
        self.i2c
            .write(SEN5X_I2C_ADDRESS, &buf[..len])
            .await
            .map_err(Error::I2c)?;
        self.delay.delay_ms(delay).await;
        Ok(())
    }
}
