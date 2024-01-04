
use embedded_hal::blocking::delay::DelayMs;
use embedded_hal::blocking::i2c::{Read, Write, WriteRead};

use crate::commands::Command;
use crate::error::Error;
use crate::types::{SensorDataInt, SensorData, SensorDataRaw, VersionInfo};
use crate::utils::*;
use sensirion_i2c::i2c;

const SEN5X_I2C_ADDRESS: u8 = 0x69;

/// SEN5X sensor instance. Use related methods to take measurements.
#[derive(Debug, Default)]
pub struct Sen5x<I2C, D> {
    i2c: I2C,
    delay: D,
}

impl<I2C, D, E> Sen5x<I2C, D>
where
    I2C: Read<Error = E> + Write<Error = E> + WriteRead<Error = E>,
    D: DelayMs<u32>,
{
    pub fn new(i2c: I2C, delay: D) -> Self {
        Sen5x {
            i2c,
            delay,
        }
    }

    /// Reset the sensor
    pub fn device_reset(&mut self) -> Result<(), Error<E>> {
        self.write_command(Command::DeviceReset)?;
        Ok(())
    }

    /// Get 48-bit serial number
    pub fn serial_number(&mut self) -> Result<u64, Error<E>> {
        let mut buf = [0; 9];
        self.delayed_read_cmd(Command::GetSerialNumber, &mut buf)?;
        
        let serial = get_u64_from_buf(&buf, 0);

        Ok(serial)
    }

    pub fn get_version_info(&mut self) -> Result<VersionInfo, Error<E>> {
        let mut buf = [0; 12];
        self.delayed_read_cmd(Command::GetVersion, &mut buf)?;

        Ok(VersionInfo{
            firmware_major: get_u8_from_buf(&buf, 0),
            firmware_minor: get_u8_from_buf(&buf, 1),
            firmware_debug: get_bool_from_buf(&buf, 2),
            hardware_major: get_u8_from_buf(&buf, 3),
            hardware_minor: get_u8_from_buf(&buf, 4),
            protocol_major: get_u8_from_buf(&buf, 5),
            protocol_minor: get_u8_from_buf(&buf, 6),
        })
    }

    pub fn read_device_status(&mut self) -> Result<u32, Error<E>> {
        let mut buf = [0; 6];
        self.delayed_read_cmd(Command::ReadDeviceStatus, &mut buf)?;
        
        let status = get_u32_from_buf(&buf, 0);

        Ok(status)
    }


    /// Start periodic measurement
    pub fn start_measurement(&mut self) -> Result<(), Error<E>> {
        self.write_command(Command::StartMeasurement)?;
        Ok(())
    }

    /// Start periodic measurement without PM
    pub fn start_measurement_without_pm(&mut self) -> Result<(), Error<E>> {
        self.write_command(Command::StartMeasurementWithoutPm)?;
        Ok(())
    }

    /// Stop periodic measurement
    pub fn stop_measurement(&mut self) -> Result<(), Error<E>> {
        self.write_command(Command::StopMeasurement)?;
        Ok(())
    }

    /// Start fan cleaning
    pub fn start_fan_cleaning(&mut self) -> Result<(), Error<E>> {
        self.write_command(Command::StartFanCleaning)?;
        Ok(())
    }

    /// Get warm start parameter
    pub fn get_warm_start_parameter(&mut self) -> Result<u16, Error<E>> {
        let mut buf = [0; 3];
        self.delayed_read_cmd(Command::GetWarmStartParameter, &mut buf)?;
        
        let warm_start = get_u16_from_buf(&buf, 0);

        Ok(warm_start)
    }

    /// Set warm start parameter
    pub fn set_warm_start_parameter(&mut self, warm_start: u16) -> Result<(), Error<E>> {
        self.write_command_with_u16(Command::SetWarmStartParameter, warm_start)?;
        Ok(())
    }

    /// Read measured values from the sensor
    pub fn read_measured_values(&mut self) -> Result<SensorData, Error<E>>
    {
        let raw_data = self.read_measured_values_as_integers()?;
        let mass_concentration_pm1p0 = raw_data.mass_concentration_pm1p0 as f32 / 10.0;
        let mass_concentration_pm2p5 = raw_data.mass_concentration_pm2p5 as f32 / 10.0;
        let mass_concentration_pm4p0 = raw_data.mass_concentration_pm4p0 as f32 / 10.0;
        let mass_concentration_pm10p0 = raw_data.mass_concentration_pm10p0 as f32 / 10.0;
        let ambient_humidity = raw_data.ambient_humidity as f32 / 100.0;
        let ambient_temperature = raw_data.ambient_temperature as f32 / 200.0;
        let voc_index = raw_data.voc_index as f32 / 10.0;
        let nox_index = raw_data.nox_index as f32 / 10.0;

        Ok(SensorData {
            mass_concentration_pm1p0,
            mass_concentration_pm2p5,
            mass_concentration_pm4p0,
            mass_concentration_pm10p0,
            ambient_humidity,
            ambient_temperature,
            voc_index,
            nox_index,
        })
    }

    pub fn read_measured_raw_values(&mut self) -> Result<SensorDataRaw, Error<E>>
    {
        let mut buf = [0; 12];
        self.delayed_read_cmd(Command::ReadMeasuredRawValues, &mut buf)?;
        
        
        let raw_humidity = i16::from_be_bytes([buf[0], buf[1]]);
        let raw_temperature = i16::from_be_bytes([buf[2], buf[3]]);
        let raw_voc = u16::from_be_bytes([buf[4], buf[5]]);
        let raw_nox = u16::from_be_bytes([buf[6], buf[7]]);

        Ok(SensorDataRaw {
            raw_humidity,
            raw_temperature,
            raw_voc,
            raw_nox,
        })
    }

    fn read_measured_values_as_integers(&mut self) -> Result<SensorDataInt, Error<E>> {
        let mut buf = [0; 24];
        self.delayed_read_cmd(Command::ReadMeasuredValuesAsIntegers, &mut buf)?;
        
        Ok(SensorDataInt {
            mass_concentration_pm1p0: get_u16_from_buf(&buf, 0),
            mass_concentration_pm2p5: get_u16_from_buf(&buf, 3),
            mass_concentration_pm4p0: get_u16_from_buf(&buf, 6),
            mass_concentration_pm10p0: get_u16_from_buf(&buf, 9),
            ambient_humidity: get_i16_from_buf(&buf, 12),
            ambient_temperature: get_i16_from_buf(&buf, 15),
            voc_index: get_i16_from_buf(&buf, 18),
            nox_index: get_i16_from_buf(&buf, 21),
        })
    }

    

    /// Command for reading values from the sensor
    fn delayed_read_cmd(&mut self, cmd: Command, data: &mut [u8]) -> Result<(), Error<E>> {
        self.write_command(cmd)?;
        i2c::read_words_with_crc(&mut self.i2c, SEN5X_I2C_ADDRESS, data)?;
        Ok(())
    }

    /// Writes commands without additional arguments.
    fn write_command(&mut self, cmd: Command) -> Result<(), Error<E>> {
        let (command, delay) = cmd.as_tuple();
        
        i2c::write_command_u16(&mut self.i2c, SEN5X_I2C_ADDRESS, command).map_err(Error::I2c)?;
        self.delay.delay_ms(delay);
        Ok(())
    }


    /// Sets sensor internal parameter
    fn write_command_with_u16(&mut self, cmd: Command, data: u16) -> Result<(), Error<E>> {
        let (command, delay) = cmd.as_tuple();
        let c = command.to_be_bytes();
        let d = data.to_be_bytes();

        let mut buf = [0; 5];
        buf[0..2].copy_from_slice(&c);
        buf[2..4].copy_from_slice(&d);
        buf[4] = sensirion_i2c::crc8::calculate(&d);

        self.i2c
            .write(SEN5X_I2C_ADDRESS, &buf)
            .map_err(Error::I2c)?;
        self.delay.delay_ms(delay);
        Ok(())
    }
}


#[cfg(test)]
mod tests {
    use embedded_hal_mock as hal;

    use self::hal::delay::MockNoop as DelayMock;
    use self::hal::i2c::{Mock as I2cMock, Transaction};
    use super::*;

    /// Test the get_serial_number function
    #[test]
    fn test_get_serial_number() {
        // Arrange
        let (cmd, _, _) = Command::GetSerialNumber.as_tuple();
        let expectations = [
            Transaction::write(SEN5X_I2C_ADDRESS, cmd.to_be_bytes().to_vec()),
            Transaction::read(
                SEN5X_I2C_ADDRESS,
                vec![0xbe, 0xef, 0x92, 0xbe, 0xef, 0x92, 0xbe, 0xef, 0x92],
            ),
        ];
        let mock = I2cMock::new(&expectations);
        let mut sensor = Sen5x::new(mock, DelayMock);
        // Act
        let serial = sensor.serial_number().unwrap();
        // Assert
        assert_eq!(serial, 0xbeefbeefbeef);
    }
}