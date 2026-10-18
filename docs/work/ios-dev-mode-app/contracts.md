# Contracts (landed by task 01, first commit)

Shared by all units. Changing these after task 01 merges requires updating tasks 02 and 03.

Deviation from the brief (deliberate): one error enum `BackendError` instead of `DetectError` + `BackendError` (every call can hit NoDevice/Timeout); no `Unsupported` variant (iOS < 16 is decided by the orchestrator from `DeviceInfo::ios_major()`); added `present()` to the trait so the poller can watch the device vanish/return without a lockdown round-trip; no separate `Rebooting` state (it is `Working(Step::Reboot)`).

## `src/model.rs`
```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceInfo {
    pub name: String,         // lockdown DeviceName
    pub product_type: String, // lockdown ProductType, e.g. "iPhone14,2"
    pub ios_version: String,  // lockdown ProductVersion, e.g. "17.5.1"
    pub udid: String,         // lockdown UniqueDeviceID
}

impl DeviceInfo {
    /// Major iOS version; 0 when unparsable (treated as "supported").
    pub fn ios_major(&self) -> u32 {
        self.ios_version.split('.').next().and_then(|s| s.parse().ok()).unwrap_or(0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum BackendError {
    #[error("usbmuxd not reachable")]
    NoUsbmuxd,
    #[error("no device connected")]
    NoDevice,
    #[error("device not trusted: {0}")]
    NotTrusted(String),
    #[error("timed out")]
    Timeout,
    #[error("{0}")]
    Other(String),
}

/// Wizard steps, in display order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step { Detect, Reveal, Enable, Reboot, Reconnect, Accept, Confirm }

impl Step {
    pub const ALL: [Step; 7] = [
        Step::Detect, Step::Reveal, Step::Enable, Step::Reboot,
        Step::Reconnect, Step::Accept, Step::Confirm,
    ];
    pub fn index(self) -> usize { self as usize }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppState {
    Idle,                         // app start, nothing checked yet
    NoUsbmuxd,
    NoDevice,
    NotTrusted(String),
    NotNeeded(String),            // iOS version string, major < 16
    Ready(DeviceInfo),            // detected, Developer Mode is OFF
    Working(Step),
    WaitingUser { manual: bool }, // manual=true: reveal/enable unsupported, user toggles in Settings
    Success,
    Error(String),
}

impl AppState {
    /// Map a backend error to the screen to show.
    pub fn from_error(e: &BackendError) -> Self {
        match e {
            BackendError::NoUsbmuxd => AppState::NoUsbmuxd,
            BackendError::NoDevice => AppState::NoDevice,
            BackendError::NotTrusted(m) => AppState::NotTrusted(m.clone()),
            other => AppState::Error(other.to_string()),
        }
    }
}

/// UI -> worker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command { Check, Activate }

/// Worker -> UI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event { State(AppState), Log(String) }
```

## `src/backend/mod.rs`
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

## `src/backend/mock.rs` (public API)
- `pub struct Script { pub present, pub detect, pub status: VecDeque<Result<..>>, pub reveal, pub enable, pub accept: Result<(), BackendError> }` (`#[derive(Default)]`; default `reveal/enable/accept` = `Ok(())`).
- Queue rule: pop front while `len > 1`; the last entry repeats forever; empty queue → `Err(Other("mock: empty script"))`.
- `MockBackend::new(script) -> Self`, `.with_delay(Duration) -> Self` (sleep before each call), `.calls() -> Vec<&'static str>` (method names in call order).
- `MockBackend::demo() -> Self`: happy path with 600 ms delay (used by `--mock`).

## `src/backend/idevice_impl.rs` (public API)
- `pub struct IdeviceBackend { .. }`, `IdeviceBackend::new() -> Self`, `impl DevModeBackend for IdeviceBackend`.
- Task 01 lands a stub where every method returns `Err(BackendError::Other("not implemented".into()))`; task 02 fills it.
