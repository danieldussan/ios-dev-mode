# Task 02 — Real backend over idevice

## Goal
Replace the stub in `src/backend/idevice_impl.rs` with a real `DevModeBackend` built on `idevice =0.1.68`, using typed errors and a timeout on every call.

## Precondition
Task 01 merged **and** the user spike gate passed: read `docs/work/ios-dev-mode-app/spike-results.md` first (iOS tested, which AMFI calls worked, observed errors, reboot time). Shape error mapping and timeouts on those findings. If the gate says "plan B", stop and report.

## Owned paths
`src/backend/idevice_impl.rs` only. Do not edit `model.rs`, `backend/mod.rs`, `mock.rs`, `Cargo.toml` (if you truly need a new dep, report it instead).

## Contract (already in the repo, do not change)
```rust
pub enum BackendError { NoUsbmuxd, NoDevice, NotTrusted(String), Timeout, Other(String) }
pub struct DeviceInfo { pub name: String, pub product_type: String, pub ios_version: String, pub udid: String }
#[async_trait::async_trait]
pub trait DevModeBackend: Send + Sync {
    async fn present(&self) -> Result<bool, BackendError>; // Ok(false)=no device, Err(NoUsbmuxd)=daemon down
    async fn detect(&self) -> Result<DeviceInfo, BackendError>;
    async fn status(&self) -> Result<bool, BackendError>;
    async fn reveal(&self) -> Result<(), BackendError>;
    async fn enable(&self) -> Result<(), BackendError>;  // reboots device
    async fn accept(&self) -> Result<(), BackendError>;  // dialog on device
}
```
Public API to keep: `pub struct IdeviceBackend`, `IdeviceBackend::new() -> Self`, `Default`.

## idevice facts (verified in crate source)
- `UsbmuxdConnection::default().await` (Windows: TCP 127.0.0.1:27015; unix: socket). Connect failure ⇒ usbmuxd missing.
- `conn.get_devices().await -> Vec<UsbmuxdDevice>`; `dev.to_provider(UsbmuxdAddr::default(), "ios-dev-mode") -> UsbmuxdProvider`.
- `LockdownClient::connect(&provider).await`; `lockdown.start_session(&provider.get_pairing_file().await?).await`; `lockdown.get_value(Some("DeviceName"), None).await -> plist::Value` (`.as_string()`).
- `AmfiClient::connect(&provider).await` (trait `IdeviceService`; it does its own lockdown session + TLS); then `get_developer_mode_status() -> bool`, `reveal_developer_mode_option_in_ui()`, `enable_developer_mode()`, `accept_developer_mode()` — all `&mut self`, async, `Result<_, IdeviceError>`.
- Source: `~/.cargo/registry/src/*/idevice-0.1.68/src/{usbmuxd/mod.rs,services/lockdown.rs,services/amfi.rs,lib.rs}`. Check exact `use` paths and `IdeviceError` variants there.

## Steps
1. `IdeviceBackend { timeout: Duration }`, `new()` → 10 s. Private `async fn with_timeout<T>(&self, fut) -> Result<T, BackendError>` mapping elapsed → `BackendError::Timeout`.
2. Private `async fn provider(&self) -> Result<UsbmuxdProvider, BackendError>`: **new** usbmuxd connection every call (the phone drops sessions on reboot; never cache providers/clients). Connect error → `NoUsbmuxd`; empty device list → `NoDevice`; else first device's provider.
3. Private `fn map_err(e: IdeviceError) -> BackendError`: pairing-file/session/TLS/"not paired"/password-protected-style errors → `NotTrusted(e.to_string())`; device-gone/connection-refused-style → `NoDevice`; rest → `Other(e.to_string())`. Pick variants by reading the `IdeviceError` enum and the spike findings; keep it one `match`, with a catch-all.
4. `present`: usbmuxd connect (`NoUsbmuxd` on error) + `get_devices` → `Ok(!list.is_empty())`. Wrap in timeout.
5. `detect`: provider → lockdown connect → `start_session` (failure ⇒ `NotTrusted`) → read the 4 keys (missing/non-string ⇒ empty string, except `ProductVersion` ⇒ `Other`) → `DeviceInfo`. Whole thing in one timeout.
6. `status`/`reveal`/`enable`/`accept`: each = provider → `AmfiClient::connect` → one call, all inside one timeout. Small private helper `async fn amfi(&self) -> Result<AmfiClient, BackendError>` to avoid repetition.
7. `enable`: the device may drop the connection while rebooting; if the spike showed `enable` returning an I/O/disconnect error even on success, treat that specific error as `Ok(())` and leave a `// ponytail:` comment citing the spike note.
8. Unit tests (no phone): `map_err` for one sample per mapped group, and `with_timeout` returning `Timeout` for a never-completing future under `#[tokio::test(start_paused = true)]`.
9. No real-device check in this task (do not edit other files to wire it up); real-device verification is in `manual-qa.md`. Say so in your report.
10. Commit (`ds-commit-style`).

## Test command
`cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --check`

## Done when
Commands pass; no `not implemented` left; every public method uses a fresh provider and a timeout; error mapping documented in a short comment above `map_err`.

## Out of scope
Orchestration, retries, polling, UI (task 03). Multi-device selection (first device wins).
