# Task 01 — Foundation: crate, contracts, mock backend, spike

## Goal
Create the Rust crate at the repo root, land the shared contracts (types + trait), a scripted mock backend, and a console spike the **user** runs against a real iPhone. You cannot run the spike against a phone; just make it build.

## Owned paths
`Cargo.toml`, `Cargo.lock`, `.gitignore`, `src/model.rs`, `src/backend/mod.rs`, `src/backend/mock.rs`, `examples/spike.rs`. You also create **stubs** `src/main.rs` and `src/backend/idevice_impl.rs` (afterwards owned by tasks 03 and 02 — keep them minimal).

## Context
- Repo is empty (one init commit). Rust 1.94, edition 2024. Cargo only.
- `idevice 0.1.68` facts (from crate source): features `usbmuxd`, `amfi`, `ring` exist; lockdown needs no feature. `UsbmuxdConnection::default().await`, `.get_devices().await -> Vec<UsbmuxdDevice>`, `dev.to_provider(UsbmuxdAddr::default(), "label")`, `LockdownClient::connect(&provider).await`, `lockdown.start_session(&provider.get_pairing_file().await?).await`, `lockdown.get_value(Some("ProductVersion"), None).await -> plist::Value`, `AmfiClient::connect(&provider).await` (trait `IdeviceService`), then `get_developer_mode_status() -> bool`, `reveal_developer_mode_option_in_ui()`, `enable_developer_mode()` (reboots), `accept_developer_mode()`. All `async`, `Result<_, IdeviceError>`. Exact import paths: check `~/.cargo/registry/src/*/idevice-0.1.68/src/lib.rs`.

## Steps
1. `cargo init --name ios-dev-mode` at repo root (bin crate, edition 2024). Ensure `.gitignore` has `/target`.
2. Add deps: `cargo add idevice@=0.1.68 --no-default-features -F usbmuxd,amfi,ring`; `cargo add tokio -F rt-multi-thread,macros,time,sync`; `cargo add thiserror@2 async-trait anyhow plist`; `cargo add --dev tokio -F test-util,macros,rt`. (Do not add eframe/rfd here — task 03 does.) Add release profile:
   ```toml
   [profile.release]
   opt-level = "z"
   lto = true
   strip = true
   panic = "abort"
   codegen-units = 1
   ```
   If `ring` does not compile, drop `--no-default-features`/`ring` (default `aws-lc`) and note it in the report.
3. Write `src/model.rs` exactly as below (contract):
```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceInfo { pub name: String, pub product_type: String, pub ios_version: String, pub udid: String }
impl DeviceInfo {
    /// Major iOS version; 0 when unparsable (treated as "supported").
    pub fn ios_major(&self) -> u32 { self.ios_version.split('.').next().and_then(|s| s.parse().ok()).unwrap_or(0) }
}
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum BackendError {
    #[error("usbmuxd not reachable")] NoUsbmuxd,
    #[error("no device connected")] NoDevice,
    #[error("device not trusted: {0}")] NotTrusted(String),
    #[error("timed out")] Timeout,
    #[error("{0}")] Other(String),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step { Detect, Reveal, Enable, Reboot, Reconnect, Accept, Confirm }
impl Step {
    pub const ALL: [Step; 7] = [Step::Detect, Step::Reveal, Step::Enable, Step::Reboot, Step::Reconnect, Step::Accept, Step::Confirm];
    pub fn index(self) -> usize { self as usize }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppState {
    Idle, NoUsbmuxd, NoDevice, NotTrusted(String),
    NotNeeded(String),            // iOS version string, major < 16
    Ready(DeviceInfo),            // detected, Developer Mode OFF
    Working(Step),
    WaitingUser { manual: bool }, // manual=true: user toggles in Settings
    Success, Error(String),
}
impl AppState {
    pub fn from_error(e: &BackendError) -> Self {
        match e {
            BackendError::NoUsbmuxd => AppState::NoUsbmuxd,
            BackendError::NoDevice => AppState::NoDevice,
            BackendError::NotTrusted(m) => AppState::NotTrusted(m.clone()),
            other => AppState::Error(other.to_string()),
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub enum Command { Check, Activate }
#[derive(Debug, Clone, PartialEq, Eq)] pub enum Event { State(AppState), Log(String) }
```
   (rustfmt will reflow it.) Add `#[cfg(test)]` tests: `ios_major` for `"17.5.1"`→17, `"16"`→16, `""`→0; `from_error` mapping for all 5 variants.
4. Write `src/backend/mod.rs` (contract):
```rust
pub mod idevice_impl;
pub mod mock;
use crate::model::{BackendError, DeviceInfo};
#[async_trait::async_trait]
pub trait DevModeBackend: Send + Sync {
    /// Ok(true) if usbmuxd lists >= 1 device, Ok(false) if none, Err(NoUsbmuxd) if daemon down.
    async fn present(&self) -> Result<bool, BackendError>;
    /// First device + lockdown session + DeviceName/ProductType/ProductVersion/UniqueDeviceID.
    async fn detect(&self) -> Result<DeviceInfo, BackendError>;
    async fn status(&self) -> Result<bool, BackendError>;
    async fn reveal(&self) -> Result<(), BackendError>;
    /// Reboots the device on success.
    async fn enable(&self) -> Result<(), BackendError>;
    /// Shows the confirm dialog on the device.
    async fn accept(&self) -> Result<(), BackendError>;
}
```
5. Stub `src/backend/idevice_impl.rs`: `pub struct IdeviceBackend;` with `pub fn new() -> Self` (+ `Default`) and `impl DevModeBackend` where every method returns `Err(BackendError::Other("not implemented".into()))`.
6. Stub `src/main.rs`: `mod backend; mod model; fn main() {}` with `#![allow(dead_code)]` so clippy passes. **Commit 1** (contracts).
7. `src/backend/mock.rs`:
   - `#[derive(Default)] pub struct Script { pub present: VecDeque<Result<bool, BackendError>>, pub detect: VecDeque<Result<DeviceInfo, BackendError>>, pub status: VecDeque<Result<bool, BackendError>>, pub reveal: Result<(), BackendError>, pub enable: Result<(), BackendError>, pub accept: Result<(), BackendError> }` — implement `Default` by hand so the three `Result<()>` default to `Ok(())`.
   - `pub struct MockBackend { script: std::sync::Mutex<Script>, calls: std::sync::Mutex<Vec<&'static str>>, delay: Duration }`; `new(script)`, `with_delay(self, d)`, `calls(&self) -> Vec<&'static str>`.
   - Helper `fn next<T: Clone>(q: &mut VecDeque<Result<T, BackendError>>) -> Result<T, BackendError>`: `len > 1` → pop_front; `len == 1` → clone front (repeats); empty → `Err(Other("mock: empty script".into()))`.
   - Each trait method: `if !delay.is_zero() { tokio::time::sleep(delay).await }`, push its name (`"present"`, `"detect"`, ...) to `calls`, return the scripted value. Never hold a lock across `.await`.
   - `pub fn demo() -> Self`: delay 600 ms; device `DeviceInfo { name: "iPhone de prueba", product_type: "iPhone15,2", ios_version: "18.1", udid: "0000-DEMO" }`; `detect: [Ok(dev)]`, `status: [Ok(false), Ok(false), Ok(true)]`, `present: [Ok(false), Ok(true)]`.
   - Tests (`#[tokio::test]`): repeat-last rule, empty → Err, `calls()` order, `reveal` default Ok.
8. `examples/spike.rs` (`#[tokio::main]`, `anyhow::Result`): subcommand from `std::env::args().nth(1)`: `info` | `status` | `reveal` | `enable` | `accept`. Always: connect usbmuxd (print error + hint "is usbmuxd/iTunes running?" on failure), list devices (print count, exit if 0), take first, build provider, `LockdownClient::connect` + `start_session`, print ProductVersion/ProductType/DeviceName/UniqueDeviceID, measure and print elapsed ms for each call. Then per subcommand: open `AmfiClient::connect(&provider)` and call the matching method, printing `Ok(..)`/`Err({e:?})`. Print a `Usage:` line on unknown arg. Top-of-file doc comment: how to run (`cargo run --example spike -- status`), Linux needs `usbmuxd` daemon, `enable` reboots the phone, run `accept` after unlocking post-reboot.
9. **Commit 2** (mock + spike). Use the `ds-commit-style` skill.

## Test command
`cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --check && cargo build --example spike`

## Done when
All four commands pass; `model.rs`/`backend/mod.rs` match the contract exactly (names, variants, signatures); stubs compile; two commits.

## Out of scope
Real backend logic (task 02), UI/orchestrator/eframe (task 03), running the spike on a phone (user gate).
