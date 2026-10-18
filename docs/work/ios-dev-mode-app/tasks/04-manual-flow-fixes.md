# 04 - Manual flow fixes (from real-device QA + review)

Owned paths: `src/poller.rs`, `src/model.rs`, `src/app.rs`, `src/ui.rs`, `src/backend/idevice_impl.rs`, `src/backend/mock.rs`, `src/main.rs`. Run on branch `feat/ios-dev-mode-app` directly (single unit). Test: `cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --check`. Commit with ds-commit-style (small commits).

## Real-device findings (iPhone XR, iOS 18.7.9, dev mode off)
- `reveal()` OK -> option appeared in Settings. `enable()` did NOT reboot the phone by itself (user did it manually: Settings > Privacy & Security > Developer Mode > on > restart > after boot the phone itself asked "Turn On" > passcode). So **assume the phone never reboots on its own**; the app must watch for the user doing it.
- UI got stuck on "Actívalo a mano ... Confirmar en el iPhone", button "Trabajando..." disabled, after dev mode was actually active. Cause: `manual()` only polls `status()` and ignores errors silently. While the phone is locked, `AmfiClient::connect` fails with `unknown error 'PasswordProtected' returned from device` (reproduced with `cargo run --example spike -- status` on the locked phone). So status can only be read when the phone is UNLOCKED, and the app showed no hint.

## Required changes
1. **Locked detection**: map `PasswordProtected` (match by message "PasswordProtected"/"password"; keep `"locked"` not `"lock"`) to a locked error (reuse `NotTrusted` only if no better; prefer new `BackendError::Locked`). UI must say "Desbloquea el iPhone" while polling continues. Locked is never fatal and never a timeout error during manual/wait phases.
2. **Manual mode becomes an active watcher, no dead end**: after reveal ok and (enable ok OR enable Err other than NoUsbmuxd/NoDevice - reviewer finding: enable may drop the connection), do not assume reboot. Enter a single "waiting for user" phase that polls every ~2s: `present()` + `status()`. Track transitions and show them as distinct steps in the wizard: (a) "Activa el modo en Ajustes y reinicia" (b) phone disappeared -> "Reiniciando..." (c) phone reappeared -> "Desbloquea el iPhone" (if Locked) (d) once unlocked after reboot, call `accept()` (ignore Err, retry on later polls, at most every ~10s; user may also confirm on the phone by themselves) -> "Confirma en el iPhone: toca Activar e ingresa tu código" (e) `status()==true` -> Success. Success detection must work no matter which path the user took (e.g. dev mode already on after reboot).
3. **Escape hatch button**: while in that waiting phase the main button is enabled and reads "Ya reinicié / Verificar ahora"; click = send `Command::Verify` which immediately runs detect + status (+ accept if device present and status false) and reports in the log. Never leave a disabled "Trabajando..." button for more than the automatic part. Also add a "Cancelar"/"Volver" secondary that returns to Check.
4. **Caps**: no hard error after 5 min in this phase; raise to 15 min total, then Error with MANUAL_HELP and Reintentar. Auto flow (when enable does reboot the phone) keeps working: gone/back logic can be merged into the same watcher.
5. **Log**: every poll failure that is not "expected" goes to the Detalles log once per change (not spammed each 2s), including the raw error text.
6. Reviewer items to fold in (cheap ones only): `map_err` use "locked" not "lock"; map `get_pairing_file` failure / `UnexpectedResponse` containing "PairRecord" to `NotTrusted`; send `Command::Check` automatically at startup; remove crate-wide `#![allow(dead_code)]` if no warnings result; hide the 7-step list during `Working(Detect)`; wrap errors from check/accept in Spanish text + MANUAL_HELP.

## Tests (paused-time with MockBackend, extend the Script as needed)
- enable=Err(Timeout), present [true,false,true], status [Err(Locked), Ok(false), Ok(true)] -> reaches Success, passes through states Reboot -> Locked/unlock hint -> WaitingUser -> Success.
- Phone never reboots, user toggles dev mode in Settings only: status eventually Ok(true) -> Success (no gone required).
- Locked errors never produce Error state before the 15 min cap.
- `Command::Verify` works from the waiting phase and from Error.
- map_err: PasswordProtected message -> Locked.
Real-device manual check afterwards by the user (iPhone XR dev mode off, repeat the flow).
