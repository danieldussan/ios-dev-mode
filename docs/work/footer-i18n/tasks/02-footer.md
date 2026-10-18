# Task 02: footer + language dropdown

Branch `feat/ios-dev-mode-app` (current worktree). Needs task 01 committed. Cargo only, no new crates.
Owned: `src/app.rs`, `src/ui.rs`, `src/main.rs`.
Test: `cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --check`. Commit via ds-commit-style: `feat: add footer link and language selector`.

## Goal
Bottom-pinned footer in the fixed 480x560 window: left = "Hecho por danieldussan" (translated) + GitHub icon, both clickable → https://github.com/danieldussan; right = language ComboBox (Español/English/Português). Switching re-renders everything live, including window title.

## Available from task 01 (already in code)
```rust
// src/i18n.rs
pub enum Lang { Es, En, Pt }  // Lang::ALL, lang.name(), Lang::from_env()
pub fn t(lang: Lang, key: Key) -> &'static str;  // Key::MadeBy, Key::Language, Key::WindowTitle exist
// src/app.rs: App { pub lang: Lang, .. }; ui::show(ui, &state, &log, lang) -> Option<Command>
```

## Steps
1. **Check egui 0.35 API first** (Context7 `/emilk/egui` or `cargo doc -p egui --open`): name of bottom panel in 0.35 (`egui::Panel::bottom` vs `TopBottomPanel::bottom`) and how it is shown from `App::ui(&mut self, ui: &mut Ui, ..)` (existing code calls `CentralPanel::default().show(ui, ..)`); `ComboBox::from_id_salt(..).selected_text(..).show_ui(..)`; `ui.selectable_value`; `ctx.load_texture`, `egui::ColorImage::from_rgba_unmultiplied`, `egui::Image::new((tex.id(), size))`, `ImageButton`/`Image::sense(Sense::click())`; `ViewportCommand::Title`.
2. **`src/ui.rs`** add:
   - `const GITHUB_URL: &str = "https://github.com/danieldussan";`
   - ICON (user-approved, overrides bitmap idea): use the real SVG at `assets/github.svg` (already in repo, black fill; the UI is dark, so tint it with `Image::tint(ui.visuals().text_color())` or replace fill with white in the string before loading). Add `egui_extras` (match the egui version, features `svg` + image loaders via `egui_extras::install_image_loaders(ctx)` once at app creation), load with `egui::include_image!("../assets/github.svg")` and show `egui::Image::new(...)` at ~18x18 with click sense. If `egui_extras` svg fails to build on Windows target concerns (resvg is pure Rust, should be fine), report instead of falling back silently.
   - `pub fn footer(ui: &mut egui::Ui, lang: &mut Lang)`: `ui.horizontal`: `ui.hyperlink_to(t(*lang, Key::MadeBy), GITHUB_URL)`; icon image with click sense → `ui.ctx().open_url(egui::OpenUrl::new_tab(GITHUB_URL))`, `on_hover_cursor(PointingHand)`; then right-aligned (`ui.with_layout(Layout::right_to_left(Align::Center), ..)`) `ComboBox` labelled/tooltipped `t(*lang, Key::Language)`, `selected_text(lang.name())`, `for l in Lang::ALL { ui.selectable_value(lang, l, l.name()); }`.
   - In `show`: wrap the existing content in `egui::ScrollArea::vertical().auto_shrink([false; 2])` so long states (7 steps + Detalles open) scroll instead of pushing anything out.
3. **`src/app.rs`** in `ui()`: show the bottom panel **before** `CentralPanel` (panel id `"footer"`), calling `ui::footer(ui, &mut self.lang, &tex)`. If `self.lang` changed this frame: `ui.ctx().send_viewport_cmd(egui::ViewportCommand::Title(t(self.lang, Key::WindowTitle).into()))`.
4. **`src/main.rs`**: only if needed for the texture/title wiring; otherwise untouched.
5. Test: unit test that `assets/github.svg` is included and non-empty (`include_str!` len > 0 and contains "<path"). Run test command.
6. Smoke: `cargo run -- --mock`; check in Idle, Waiting (all steps shown) with Detalles expanded: footer visible, click text and icon open browser, switching language changes header/steps/buttons/footer/title instantly, popup (mock no-usbmuxd if reachable) in current language. Commit.

## Done when
Footer always visible at 480x560 in every state; link + icon open GitHub; dropdown switches language live incl. title; test command green.

## Out of scope
Persisting language; other new crates beyond `egui_extras` (svg feature); hand-made bitmap icon.
