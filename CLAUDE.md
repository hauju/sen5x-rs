# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

`no_std` Rust driver for the Sensirion SEN5x environmental sensor series (PM, VOC, NOx, humidity, temperature). Communicates over I2C using `embedded-hal` 1.0 blocking traits with optional `embedded-hal-async` support. Based on the Sensirion C reference implementation and follows the same patterns as the scd4x-rs driver.

## Build & Test Commands

```bash
cargo build                                # Build the library
cargo test                                 # Run tests (uses embedded-hal-mock)
cargo clippy                               # Lint
cargo doc --open                           # Generate and view docs
cargo build --features embedded-hal-async  # Build with async support
cargo build --features defmt               # Build with defmt support
```

## Architecture

- **`sen5x/mod.rs`** — Main driver struct `Sen5x<I2C, D>` generic over I2C bus and delay. Tracks `is_running` state to enforce command validity (idle-only vs allowed-during-measurement). All sensor operations go through internal helpers: `write_command`, `write_command_with_data` (single u16), `write_command_with_words` (multi-word), and `delayed_read_cmd`. I2C address is fixed at `0x69`.
- **`sen5x/async_impl.rs`** — `Sen5xAsync<I2C, D>` mirroring all public methods as `async fn`. Uses `sensirion_i2c::i2c_async`.
- **`commands.rs`** — `Command` enum (28 variants) mapping to `(u16_command_code, u32_delay_ms, bool_allowed_if_running)` tuples.
- **`types.rs`** — Data structs: `SensorData` (f32 scaled), `RawSensorData` (integer ticks), `RawMeasurementValues`, `PmValues` (extended PM), `TemperatureOffsetParameters`, `AlgorithmTuningParameters`, `VersionInfo`.
- **`error.rs`** — Error type wrapping I2C errors and CRC failures, with optional `defmt` and `thiserror` support.

## Key Conventions

- The library is `#![no_std]` (std only enabled in test cfg or via `std` feature).
- I2C response buffers use Sensirion's format: 2 data bytes + 1 CRC byte per word. Buffer indices skip every 3rd byte for CRC.
- Multi-word write commands (temperature offset, algorithm tuning, VOC state, fan interval) use `write_command_with_words` with a stack-allocated `[u8; 20]` buffer.
- Serial number and product name are ASCII strings returned as `[u8; 32]`.
- Tests use `embedded-hal-mock::eh1` with `NoopDelay`, `word()` helper for CRC, and `destroy()` + `done()` pattern.
- Features: `embedded-hal-async`, `defmt`, `thiserror`, `std`.
