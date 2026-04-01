//! SEN5x example for ESP32-C3 using Embassy async runtime.
//!
//! Reads PM, VOC, NOx, temperature, and humidity from the sensor via async I2C.
//!
//! # Hardware
//! - Board: ESP32-C3 (e.g. ESP32-C3-DevKitM-1)
//! - SEN5x sensor connected via I2C: GPIO6 = SDA, GPIO7 = SCL
//! - Target: riscv32imc-unknown-none-elf
//!
//! # Flash & monitor
//! ```bash
//! cargo run --release
//! ```

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_backtrace as _;
use esp_hal::i2c::master::{Config as I2cConfig, I2c};
use esp_hal::timer::timg::TimerGroup;
use sen5x::Sen5xAsync;

#[esp_hal_embassy::main]
async fn main(_spawner: Spawner) {
    esp_println::println!("SEN5x ESP32-C3 example starting...");

    let config = esp_hal::Config::default();
    let peripherals = esp_hal::init(config);

    // Heap allocator
    esp_alloc::heap_allocator!(72 * 1024);

    let timg0 = TimerGroup::new(peripherals.TIMG0);

    // Initialize embassy time driver
    esp_hal_embassy::init(timg0.timer0);

    // Initialize I2C for SEN5x sensor (GPIO6 = SDA, GPIO7 = SCL)
    let i2c = I2c::new(peripherals.I2C0, I2cConfig::default())
        .unwrap()
        .with_sda(peripherals.GPIO6)
        .with_scl(peripherals.GPIO7)
        .into_async();
    let mut sen5x = Sen5xAsync::new(i2c, embassy_time::Delay);

    // Reset and wait for sensor
    let _ = sen5x.device_reset().await;
    Timer::after_millis(200).await;

    let serial = sen5x.serial_number().await.unwrap();
    let serial_str = core::str::from_utf8(&serial)
        .unwrap_or("???")
        .trim_end_matches('\0');
    esp_println::println!("SEN5x serial: {}", serial_str);

    let version = sen5x.version().await.unwrap();
    esp_println::println!(
        "Firmware: {}.{}, Hardware: {}.{}",
        version.firmware_major,
        version.firmware_minor,
        version.hardware_major,
        version.hardware_minor,
    );

    sen5x.start_measurement().await.unwrap();
    esp_println::println!("Waiting for first measurement... (1 sec)");

    // Main loop: read sensor every second
    loop {
        Timer::after(Duration::from_secs(1)).await;

        match sen5x.data_ready().await {
            Ok(true) => match sen5x.measurement().await {
                Ok(data) => {
                    esp_println::println!(
                        "PM2.5: {:.1} \u{00b5}g/m\u{00b3}, Temp: {:.1}\u{00b0}C, \
                         Humidity: {:.1}%, VOC: {:.1}, NOx: {:.1}",
                        data.mass_concentration_pm2p5,
                        data.ambient_temperature,
                        data.ambient_humidity,
                        data.voc_index,
                        data.nox_index,
                    );
                }
                Err(e) => {
                    esp_println::println!("SEN5x measurement error: {:?}", e);
                }
            },
            Ok(false) => {
                esp_println::println!("Data not ready yet");
            }
            Err(e) => {
                esp_println::println!("SEN5x I2C error: {:?}", e);
            }
        }
    }
}
