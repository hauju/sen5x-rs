//! SEN5x example for Linux (e.g. Raspberry Pi).
//!
//! Reads PM, VOC, NOx, temperature, and humidity from the sensor via I2C.
//!
//! # Run
//! ```bash
//! cargo run --release
//! ```

use embedded_hal::delay::DelayNs;
use hal::{Delay, I2cdev};
use linux_embedded_hal as hal;

use sen5x::Sen5x;

fn main() {
    let dev = I2cdev::new("/dev/i2c-1").unwrap();
    let mut sensor = Sen5x::new(dev, Delay);

    sensor.device_reset().unwrap();
    hal::Delay.delay_ms(200u32);

    let serial = sensor.serial_number().unwrap();
    let serial_str = core::str::from_utf8(&serial)
        .unwrap_or("???")
        .trim_end_matches('\0');
    println!("Serial: {serial_str}");

    let version = sensor.version().unwrap();
    println!(
        "Firmware: {}.{}, Hardware: {}.{}",
        version.firmware_major, version.firmware_minor, version.hardware_major,
        version.hardware_minor,
    );

    sensor.start_measurement().unwrap();
    println!("Waiting for first measurement... (1 sec)");

    loop {
        hal::Delay.delay_ms(1000u32);

        if sensor.data_ready().unwrap() {
            let data = sensor.measurement().unwrap();
            println!(
                "PM1.0: {:.1}, PM2.5: {:.1}, PM4.0: {:.1}, PM10: {:.1} \u{00b5}g/m\u{00b3} | \
                 Humidity: {:.1}%, Temp: {:.1}\u{00b0}C, VOC: {:.1}, NOx: {:.1}",
                data.mass_concentration_pm1p0,
                data.mass_concentration_pm2p5,
                data.mass_concentration_pm4p0,
                data.mass_concentration_pm10p0,
                data.ambient_humidity,
                data.ambient_temperature,
                data.voc_index,
                data.nox_index,
            );
        }
    }
}
