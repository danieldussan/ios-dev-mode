# Manual QA (phases 4-5, not tasks)

## Phase 4 — real iPhone test matrix (release build)
Run `cargo run --release` (Linux) and the Windows `.exe`. For each row note pass/fail + log excerpt from "Detalles".

| # | Scenario | Expected |
|---|---|---|
| 1 | usbmuxd stopped / iTunes not installed | "Falta iTunes" popup, Reintentar works after starting it |
| 2 | No phone plugged | "Conecta tu iPhone…" |
| 3 | Phone plugged, never trusted | NotTrusted screen; tap Confiar, Reintentar → Ready |
| 4 | Phone locked at Verificar | Clear error or NotTrusted, no hang (timeout ≤ 10 s) |
| 5 | Developer Mode already on | Green "Modo desarrollador activo" |
| 6 | Full activation, iOS 18+ | Steps advance through reboot, accept dialog appears, Success |
| 7 | Full activation, iOS 16/17 | reveal/enable fail → manual instructions, Success after user toggles |
| 8 | Phone stays locked after reboot | Waits on Reconnect, proceeds after unlock |
| 9 | User cancels dialog on phone | Error after cap, Reintentar restarts cleanly |
| 10 | Cable pulled mid-flow | Error with reason, no crash |
| 11 | iOS < 16 device (if available) | "no necesita modo desarrollador" |

## Phase 5 — Windows packaging
- [ ] Build: on Windows `cargo build --release` (MSVC), or from Linux `rustup target add x86_64-pc-windows-gnu` + mingw-w64 and `cargo build --release --target x86_64-pc-windows-gnu`.
- [ ] `.exe` runs without a console window; window 480x560, not resizable.
- [ ] Works with iTunes (Apple site) and with "Apple Devices" (Microsoft Store) usbmuxd.
- [ ] Clean Windows VM without Rust: app starts, no missing DLLs.
- [ ] Note exe size; SmartScreen warning expected (unsigned).
