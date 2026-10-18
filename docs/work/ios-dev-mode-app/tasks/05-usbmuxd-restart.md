# 05 - usbmuxd disappears while the phone reboots

Owned: `src/poller.rs`, `src/model.rs`, `src/ui.rs`, `src/main.rs`, `src/backend/*`. Branch feat/ios-dev-mode-app. Test: `cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --check`. Commit via ds-commit-style.

## Evidence (real device, Linux)
usbmuxd journal: on phone reboot `Removed device 1` -> `usbmuxd shutting down` -> socket gone (`Connection refused`) for ~1-2 min -> udev restarts usbmuxd when the phone re-enumerates -> `Connected to device`. During that window `UsbmuxdConnection::default()` fails -> `BackendError::NoUsbmuxd` -> app shows the "install iTunes" popup mid-activation. (Windows' Apple Mobile Device Service does not stop, but the app must be robust on both.)

## Required
1. In the watcher/activation phases (after the Check that started it succeeded, i.e. we have seen the device once), `NoUsbmuxd` = "phone gone / rebooting", exactly like `present()==false` or `NoDevice`. Never a popup, never an Error, keep polling until watch cap. Show phase "Reiniciando..." (reuse existing phase) and a note that the connection returns on its own.
2. The iTunes popup/NoUsbmuxd screen only from the initial Check (or Verificar from a non-watch state) when the device was never seen in this run. Also in Check: if usbmuxd is down, do NOT stop auto-retrying forever silently: retry every ~3s while on NoUsbmuxd/NoDevice screens so the app recovers by itself when usbmuxd/phone returns (Linux case), keeping the popup shown only once per run (do not re-open it on each retry).
3. When the phone is back but locked (`Locked`), existing "Desbloquea el iPhone" screen applies.
4. Tests (paused time, MockBackend): activation/watch with present sequence including Err(NoUsbmuxd) x N then true then status Locked then Ok(true) -> Success with no Error/NoUsbmuxd state emitted after the first Check; Check on NoUsbmuxd then recovery auto-retry -> Ready; popup requested once.
