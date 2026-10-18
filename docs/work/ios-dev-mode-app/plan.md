# Plan: ios-dev-mode-app (index)

Spec: [spec.md](spec.md) · Contracts: [contracts.md](contracts.md) · Status: [status.md](status.md) · Spike log: [spike-results.md](spike-results.md) · Manual QA (phases 4-5): [manual-qa.md](manual-qa.md)

## Goal
Rust desktop app (egui/eframe + tokio + `idevice =0.1.68`) that enables iPhone Developer Mode in one guided flow; dev on Linux, ship as a Windows `.exe`.

## Non-goals
Multi-device, installers, auto-update, macOS/Linux packaging, Windows packaging automation (manual checklist only), plan B (libimobiledevice binaries) unless the spike gate fails.

## Stack
Pure Rust, edition 2024, rustc 1.94 (Linux dev). Cargo only (no JS tooling). Deps: `idevice =0.1.68` (`default-features = false`, features `usbmuxd`, `amfi`, `ring`), `tokio` (rt-multi-thread, macros, time, sync), `eframe` (latest compatible), `rfd`, `thiserror 2`, `async-trait`, `anyhow` (spike only). Dev-dep: `tokio` with `test-util`.

## Units

| # | Task file | Owned paths | Depends on | Test command |
|---|---|---|---|---|
| 01 | [tasks/01-foundation.md](tasks/01-foundation.md) | `Cargo.toml`, `Cargo.lock`, `.gitignore`, `src/model.rs`, `src/backend/mod.rs`, `src/backend/mock.rs`, `examples/spike.rs`; creates stubs `src/main.rs`, `src/backend/idevice_impl.rs` (ownership passes to 03 / 02) | - | `cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --check && cargo build --example spike` |
| 02 | [tasks/02-backend.md](tasks/02-backend.md) | `src/backend/idevice_impl.rs` | 01 + **user spike gate** | `cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --check` |
| 03 | [tasks/03-app.md](tasks/03-app.md) | `src/main.rs`, `src/poller.rs`, `src/app.rs`, `src/ui.rs` | 01 | `cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --check` |

02 and 03 have disjoint paths and both code against the contracts, so they run in parallel once 01 is merged. 03 works fully against `MockBackend`; it never needs a phone.

## Merge order
1. **01-foundation** — commit 1: contracts (`model.rs`, `backend/mod.rs`, stubs, `Cargo.toml`); commit 2: mock + spike.
2. **USER GATE (phase 0)**: the user runs `examples/spike` against the real iPhone and fills [spike-results.md](spike-results.md). Decision: `status()`/`enable()` work → continue with 02. They fail → stop; replan plan B (libimobiledevice binaries) behind the same trait. 03 may proceed during the gate.
3. **03-app** and **02-backend** in any order (parallel).
4. Final: `cargo run -- --mock` smoke + [manual-qa.md](manual-qa.md) with the real iPhone.

## Risks / open questions
- **idevice API churn**: `=0.1.68` pinned; signatures verified in the crate source (AMFI methods, `UsbmuxdConnection::default()`, `get_devices()`, `to_provider()`, `LockdownClient::get_value(Option<&str>, Option<&str>)`, `IdeviceService::connect(&dyn IdeviceProvider)`). Anything else: read `~/.cargo/registry/src/*/idevice-0.1.68/src`.
- **TLS backend on Windows**: default feature `aws-lc` needs cmake+NASM on Windows; plan uses `ring` instead. If `ring` fails to compile/handshake, fall back to default features and note it.
- **iOS 16/17**: `reveal`/`enable` documented as iOS 18+. Fallback = manual instructions + status polling; confirmed only by the spike/QA.
- **Locked phone hangs**: every backend call is wrapped in `tokio::time::timeout`.
- **usbmuxd on Linux dev**: needs the `usbmuxd` package/daemon running (`sudo systemctl start usbmuxd`) for the spike.
- **eframe/rfd API drift**: use current versions; check APIs with context7 (`/emilk/egui`) instead of memory.
- **rfd blocking dialog** on the UI thread freezes rendering while open; accepted (modal by intent).
