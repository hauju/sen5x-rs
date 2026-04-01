# Sensirion I2C SEN5x Driver

[![Crates.io](https://img.shields.io/crates/v/sen5x.svg)](https://crates.io/crates/sen5x)
[![Docs.rs](https://docs.rs/sen5x/badge.svg)](https://docs.rs/sen5x)
[![License](https://img.shields.io/crates/l/sen5x.svg)](https://github.com/hauju/sen5x-rs)
[![no_std](https://img.shields.io/badge/target-no__std-blue)](https://crates.io/crates/sen5x)

A platform-agnostic `no_std` Rust driver for the [Sensirion SEN5x series](https://sensirion.com/products/catalog/SEK-SEN5x) (SEN50/SEN54/SEN55), built on [embedded-hal](https://docs.rs/embedded-hal/) traits. Based on [embedded-i2c-sen5x](https://github.com/Sensirion/embedded-i2c-sen5x) and [sgpc3-rs](https://github.com/mjaakkol/sgpc3-rs).

## Sensirion SEN5x

The SEN5x is an all-in-one sensor solution platform for the accurate measurement of various environmental parameters, such as particulate matter, volatile organic compounds (VOCs), oxidizing gases, such as nitrogen oxide compounds (NOx), as well as humidity & temperature.

- **SEN50**: Particulate matter only
- **SEN54**: PM + VOC + humidity + temperature
- **SEN55**: PM + VOC + NOx + humidity + temperature

Further information: [Datasheet Environmental Node SEN5x](https://sensirion.com/media/documents/6791EFA0/62A1F68F/Sensirion_Datasheet_Environmental_Node_SEN5x.pdf)

## Features

- Full SEN5x command set (measurement control, fan cleaning, device info, status)
- Temperature compensation parameters (offset, slope, time constant)
- VOC/NOx algorithm tuning and state backup/restore
- Fan auto cleaning interval configuration
- RH/T acceleration mode
- Async support via the `embedded-hal-async` feature
- Optional `defmt` support for embedded logging
- Optional `thiserror` integration for `std` environments

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
sen5x = "0.1"

# For async support:
# sen5x = { version = "0.1", features = ["embedded-hal-async"] }
```

### Blocking Example

```rust
use sen5x::Sen5x;

let mut sensor = Sen5x::new(i2c, delay);

sensor.device_reset()?;
delay.delay_ms(200);

let serial = sensor.serial_number()?;
let version = sensor.version()?;

sensor.start_measurement()?;

loop {
    delay.delay_ms(1000);
    if sensor.data_ready()? {
        let data = sensor.measurement()?;
        // data.mass_concentration_pm2p5, data.ambient_temperature, etc.
    }
}
```

### Async Example

```rust
use sen5x::Sen5xAsync;

let mut sensor = Sen5xAsync::new(i2c, delay);

sensor.start_measurement().await?;

loop {
    delay.delay_ms(1000).await;
    if sensor.data_ready().await? {
        let data = sensor.measurement().await?;
    }
}
```

### Full Examples

- [Linux / Raspberry Pi](examples/linux/) — blocking driver with `linux-embedded-hal`
- [ESP32-C3 Embassy](examples/embassy-esp32c3/) — async driver with `esp-hal` and Embassy

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
   http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or
   http://opensource.org/licenses/MIT) at your option.

### Contributing

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
