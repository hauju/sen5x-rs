# Sensirion I2C SEN5x Driver

This library provides an embedded `no_std` driver for the [Sensirion SEN5x series](https://sensirion.com/products/catalog/SEK-SEN5x). This driver was built using [embedded-hal](https://docs.rs/embedded-hal/) traits. The implementaion are based on [embedded-i2c-sen5x](https://github.com/Sensirion/embedded-i2c-sen5x) and [sgpc3-rs](https://github.com/mjaakkol/sgpc3-rs).

## Sensirion SEN5x

The SEN5x is an all-in-one sensor solution platform for the accurate measurement of various environmental parameters, such as particulate matter, volatile organic compounds (VOCs), oxidizing gases, such as nitrogen oxide compounds (NOx), as well as humidity & temperature.

Further information: [Datasheet CO2 Sensor SEN5x](https://developer.sensirion.com/fileadmin/user_upload/customers/sensirion/Dokumente/15_Environmental_Sensor_Node/Datasheets/Sensirion_Environmental_Sensor_Node_SEN5x_Datasheet.pdf)

## Usage

Coming soon...

```bash
```

## Development Status

The driver is in an early development state. It allows you to:
- Get the serial number.
- Read the measurement output.

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