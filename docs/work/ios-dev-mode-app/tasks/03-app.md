# Task 03 — Orchestrator (poller) + egui UI + main wiring

## Goal
Drive the full Developer Mode flow (check, activate, reboot, reconnect, accept, poll) against the `DevModeBackend` trait, unit-tested with `MockBackend`, and render it as a fixed-size wizard window. `cargo run -- --mock` must demo the whole flow without a phone.

## Owned paths
`src/main.rs` (replace the stub), `src/poller.rs`, `src/app.rs`, `src/ui.rs`. Adding `eframe` + `rfd` to `Cargo.toml` via `cargo add` is allowed (only those two). Do not edit `model.rs`, `backend/*`.

## Contract excerpts (already in the repo)
```rust
// crate::model
pub struct DeviceInfo { pub name, pub product_type, pub ios_version, pub udid: String } // fn ios_major(&self) -> u32 (0 = unparsable)
pub enum BackendError { NoUsbmuxd, NoDevice, NotTrusted(String), Timeout, Other(String) }
pub enum Step { Detect, Reveal, Enable, Reboot, Reconnect, Accept, Confirm } // Step::ALL, step.index()
pub enum AppState { Idle, NoUsbmuxd, NoDevice, NotTrusted(String), NotNeeded(String), Ready(DeviceInfo),
                    Working(Step), WaitingUser { manual: bool }, Success, Error(String) } // AppState::from_error(&e)
pub enum Command { Check, Activate }
pub enum Event { State(AppState), Log(String) }
// crate::backend
#[async_trait] pub trait DevModeBackend: Send + Sync { present()->Result<bool>; detect()->Result<DeviceInfo>;
    status()->Result<bool>; reveal(); enable() /*reboots*/; accept() /*dialog on phone*/ }  // all async, Err = BackendError
// crate::backend::mock — MockBackend::new(Script{present,detect,status: VecDeque<Result<..>>, reveal,enable,accept: Result<()>, ..Default})
//   last queue entry repeats; .calls() -> Vec<&'static str>; MockBackend::demo() (600 ms delay happy path)
// crate::backend::idevice_impl::IdeviceBackend::new()
```

## Steps — orchestrator (`src/poller.rs`)
1. `pub struct PollConfig { gone_cap, back_cap, ready_cap, status_cap, manual_cap, fast, slow: Duration }`, `Default` = 20 s, 90 s, 120 s, 180 s, 300 s, 1 s, 3 s. `type Emit<'a> = &'a (dyn Fn(Event) + Sync);`. Emit a `Log` line for every backend call result (`format!("{step:?}: {res:?}")`).
2. `pub async fn check(b: &dyn DevModeBackend, emit: Emit)`: `Working(Detect)` → `detect()`; Err → `from_error`; Ok(d) with `1..16` contains `d.ios_major()` → `NotNeeded(d.ios_version)`; else `status()`: true → `Success`, false → `Ready(d)`, Err → `from_error`.
3. Generic helper `async fn poll_until<T>(cap, every, f: impl FnMut() -> Fut<Option<T>>) -> Option<T>` using `tokio::time::{Instant, sleep}`; `None` on cap.
4. `pub async fn activate(b, cfg: &PollConfig, emit)`:
   - `Working(Reveal)` → `reveal()`; Err → **manual fallback** (below).
   - `Working(Enable)` → `enable()`; Err → manual fallback.
   - `Working(Reboot)`: poll `present()` every `fast` until `Ok(false)` or `Err(NoDevice)`, cap `gone_cap`; none → `Error("El iPhone no se reinició. Actívalo a mano: Ajustes > Privacidad y seguridad > Modo de desarrollador.")`.
   - `Working(Reconnect)`: poll `present()` until `Ok(true)` (cap `back_cap`), then poll `detect()` every `slow` until Ok (cap `ready_cap`; any Err = still locked/booting, keep going). Timeout → `Error("El iPhone no volvió a conectarse. Desbloquéalo, revisa el cable y toca Reintentar.")`.
   - `Working(Accept)` → `accept()`; Err → `Error(..)` with the backend message.
   - `WaitingUser { manual: false }` → poll `status()` every `slow` until `Ok(true)` (cap `status_cap`, errors ignored) → `Success`; timeout → `Error("No se confirmó a tiempo. En el iPhone toca Activar e ingresa tu código, luego toca Reintentar.")`.
   - Manual fallback: `WaitingUser { manual: true }`, poll `status()` until true with `manual_cap` → `Success` / `Error(<manual instructions>)`.
5. `pub async fn run_worker(b: Arc<dyn DevModeBackend>, mut rx: tokio::sync::mpsc::UnboundedReceiver<Command>, cfg: PollConfig, emit: impl Fn(Event) + Send + Sync + 'static)`: loop over `rx.recv()`, dispatch `Check`/`Activate`, exit when the channel closes.
6. Tests with `#[tokio::test(start_paused = true)]` + `MockBackend` (collect events in `Mutex<Vec<Event>>`; assert final state and `calls()`):
   check → Success when status true; check → `NoUsbmuxd` on detect Err; iOS `"15.7"` → `NotNeeded`; activate happy path (present `[true,false,true]`, detect `[Err(NotTrusted), Ok]`, status `[false,false,true]`) → Success with call order reveal, enable, …, accept, status; never-disappears → Error; status never true → Error; reveal Err → `WaitingUser{manual:true}` then Success.

## Steps — UI
7. `cargo add eframe --no-default-features -F default_fonts,glow,wayland,x11` and `cargo add rfd`. Check current eframe/rfd APIs with context7 (`/emilk/egui`) — do not rely on memory.
8. `src/ui.rs` (Spanish UI, neutral tuteo):
   - `pub fn main_button(s: &AppState) -> (&'static str, Option<Command>)`: Idle/Success/NotNeeded → ("Verificar", Check); NoUsbmuxd/NoDevice/NotTrusted/Error → ("Reintentar", Check); Ready → ("Activar", Activate); Working/WaitingUser → ("Trabajando…", None). Unit-test it.
   - `pub fn step_label(s: Step) -> &'static str`: Detectar iPhone · Mostrar opción en Ajustes · Activar modo desarrollador · Reinicio del iPhone · Reconexión · Enviar confirmación · Confirmar en el iPhone.
   - `pub fn show(ui, state, log: &[String]) -> Option<Command>`: big status header (icon + title, color: green Success, red Error/NoUsbmuxd, amber waiting) · body text per state (device name/model/iOS for Ready; "Conecta tu iPhone por USB, desbloquéalo y toca Confiar"; "Toca Confiar en el iPhone y vuelve a intentar"; "Este iPhone (iOS X) no necesita modo desarrollador"; Reconnect: "Desbloquea tu iPhone"; WaitingUser: "En el iPhone toca Activar e ingresa tu código"; manual: "Abre Ajustes > Privacidad y seguridad > Modo de desarrollador, actívalo y reinicia"; Success: green check "Modo desarrollador activo") · step list (✔ done / ▶ current / ○ pending) + `ProgressBar` when Working/WaitingUser/Success · one main button from `main_button` (disabled when `None`) · `CollapsingHeader("Detalles")` with a scrollable log. Returns the clicked command.
9. `src/app.rs`: `pub struct App { state, log: Vec<String>, cmd_tx: tokio::sync::mpsc::UnboundedSender<Command>, ev_rx: std::sync::mpsc::Receiver<Event>, warned_usbmuxd: bool }`; `impl eframe::App`: drain `ev_rx.try_iter()`, render `ui::show` in a `CentralPanel`, send returned command. On first entry to `NoUsbmuxd` show a blocking `rfd::MessageDialog` (Warning, title "Falta iTunes", text "Instala iTunes (o Apple Devices) desde la web de Apple y vuelve a abrir esta app.", buttons to open `https://www.apple.com/itunes/` or close); open via `ctx.open_url`.
10. `src/main.rs`: `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]`; `mod app; mod backend; mod model; mod poller; mod ui;`. Parse `--mock` from `std::env::args()`. Backend `Arc<dyn DevModeBackend>` = `MockBackend::demo()` or `IdeviceBackend::new()`. Build `tokio::runtime::Runtime` (multi-thread), keep it alive in `main`. Window: title "Modo desarrollador iOS", inner size `[480.0, 560.0]`, not resizable. In the `run_native` creator: clone `cc.egui_ctx`, spawn `run_worker` on the runtime handle with `emit = move |ev| { let _ = ev_tx.send(ev); ctx.request_repaint(); }`.
11. Run `cargo run -- --mock`, click through Verificar → Activar → Success; screenshot or describe in the report. Commit(s) via `ds-commit-style` (orchestrator, then UI).

## Test command
`cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --check`

## Done when
Commands pass; all poller scenarios above are tests; `--mock` reaches "Modo desarrollador activo"; UI strings in Spanish, code in English.

## Out of scope
Real idevice calls (task 02), packaging/icons/installer, multi-device.
