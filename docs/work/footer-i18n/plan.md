# Plan: footer-i18n

Spec: [spec.md](spec.md) · Contracts: [contracts.md](contracts.md) · Status: [status.md](status.md)

## Goal
Live es/en/pt translation via one `src/i18n.rs`, plus a pinned footer with attribution link, GitHub icon and language dropdown.

## Non-goals
- Persisting the chosen language. Translating raw technical log lines. Any i18n crate. Image crates other than `egui_extras` (svg) for the user-supplied GitHub SVG at `assets/github.svg`.

## Units (sequential, same branch `feat/ios-dev-mode-app`, no new worktree)
| # | Task | Owned paths | Depends on |
|---|------|-------------|------------|
| 01 | [i18n refactor](tasks/01-i18n.md) | `src/i18n.rs` (new), `src/model.rs`, `src/poller.rs`, `src/ui.rs`, `src/app.rs`, `src/main.rs` (only `mod i18n;`) | — |
| 02 | [footer + dropdown](tasks/02-footer.md) | `src/app.rs`, `src/ui.rs`, `src/main.rs` | 01 |

Not parallel: both touch `app.rs`/`ui.rs`; 02 needs `Lang`/`t`. Could be one session; split keeps each reviewable.

Test command (both): `cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --check`

## Merge order
01 (one commit: `refactor: route ui strings through i18n module`) → 02 (`feat: add footer link and language selector`).

## Risks / open questions
- egui 0.35 panel API: `App::ui` gets a `Ui`; bottom panel may be `egui::Panel::bottom` or `TopBottomPanel::bottom` — check via Context7/`cargo doc` before coding.
- Windows usually has no `LANG` → defaults to Spanish. Acceptable per brief (dropdown covers it).
- Translations are machine-quality; user eyeballs pt/en.
- Icon: user-approved real SVG via `egui_extras` (svg feature, pure-Rust resvg); check build size/time.
