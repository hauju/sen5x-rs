# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`no_std` Rust driver for the Sensirion SEN5x environmental sensor series (PM, VOC, NOx, humidity, temperature). Communicates over I2C using `embedded-hal` 0.2 blocking traits. Early development stage.

## Build & Test Commands

```bash
cargo build          # Build the library
cargo test           # Run tests (uses embedded-hal-mock)
cargo doc --open     # Generate and view docs
```

Cross-compile for embedded targets (e.g. Raspberry Pi Pico):
```bash
cargo build --target thumbv6m-none-eabi --example rp
```

## Architecture

- **`sen5x.rs`** — Main driver struct `Sen5x<I2C, D>` generic over I2C bus and delay. All sensor operations go through `write_command` (write-only) or `delayed_read_cmd` (write then read with CRC validation via `sensirion-i2c`). The I2C address is fixed at `0x69`.
- **`commands.rs`** — `Command` enum mapping sensor operations to `(u16_command_code, u32_delay_ms)` tuples.
- **`types.rs`** — Data structs: `SensorData` (f32 scaled values), `SensorDataInt` (raw integer ticks), `SensorDataRaw`, `VersionInfo`.
- **`utils.rs`** — Buffer parsing helpers (`get_u16_from_buf`, etc.) for extracting values from I2C response buffers that include CRC bytes at every 3rd position.
- **`error.rs`** — Error type wrapping I2C errors and CRC failures, with conversion from `sensirion_i2c::i2c::Error`.

## Key Conventions

- The library is `#![no_std]` (std only enabled in test cfg).
- I2C response buffers use Sensirion's format: 2 data bytes + 1 CRC byte per word. Buffer index offsets in `read_measured_values_as_integers` step by 3 to skip CRC bytes.
- `Command::as_tuple()` returns `(command_code, delay_ms)` — some call sites destructure a 3-tuple `(cmd, _, _)` in tests.
- Tests use `embedded-hal-mock` for I2C and delay mocking.
