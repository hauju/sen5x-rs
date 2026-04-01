# Sensirion I2C SEN5x Driver

[![Crates.io](https://img.shields.io/crates/v/sen5x.svg)](https://crates.io/crates/sen5x)
[![Docs.rs](https://docs.rs/sen5x/badge.svg)](https://docs.rs/sen5x)
[![License](https://img.shields.io/crates/l/sen5x.svg)](https://github.com/hauju/sen5x-rs)

This library provides an embedded `no_std` driver for the [Sensirion SEN5x series](https://sensirion.com/products/catalog/SEK-SEN5x). This driver was built using [embedded-hal](https://docs.rs/embedded-hal/) traits. The implementaion are based on [embedded-i2c-sen5x](https://github.com/Sensirion/embedded-i2c-sen5x) and [sgpc3-rs](https://github.com/mjaakkol/sgpc3-rs).

## Sensirion SEN5x

The SEN5x is an all-in-one sensor solution platform for the accurate measurement of various environmental parameters, such as particulate matter, volatile organic compounds (VOCs), oxidizing gases, such as nitrogen oxide compounds (NOx), as well as humidity & temperature.

Further information: [Datasheet Environmental Node SEN5x](https://sensirion.com/media/documents/6791EFA0/62A1F68F/Sensirion_Datasheet_Environmental_Node_SEN5x.pdf)

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