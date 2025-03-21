# uds-client-rs

## Overview
`uds-client-rs` is a Rust library for handling UDS (Unified Diagnostic Services) communication over CAN, implementing ISO 14229. It enables interaction with ECUs (Electronic Control Units) in automotive applications.

## Features
- Supports Single Frame (SF), First Frame (FF), Consecutive Frame (CF), and Flow Control (FC) messages.
- Implements ISO 15765-2 CAN Transport Protocol.
- Async support using `tokio`.
- UDS services such as diagnostic session control, ECU reset, and real-time data requests.

## Build
This project only supports Peak CAN devices for Windows.
```
cargo run --release
```
## Installation (TODO)
Add the following to your `Cargo.toml`:

```toml
[dependencies]
uds-client-rs = "0.1"
tokio = { version = "1", features = ["full"] }
