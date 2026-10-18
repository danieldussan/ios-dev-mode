# Contracts: footer-i18n

Landed by task 01; task 02 consumes them.

## `src/i18n.rs` (new, no external crate)
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang { Es, En, Pt }

impl Lang {
    pub const ALL: [Lang; 3] = [Lang::Es, Lang::En, Lang::Pt];
    pub fn name(self) -> &'static str;              // "Español" | "English" | "Português"
    pub fn from_locale(s: &str) -> Lang;            // "en*"->En, "pt*"->Pt, else Es (case-insensitive)
    pub fn from_env() -> Lang;                      // first non-empty of LC_ALL, LC_MESSAGES, LANG -> from_locale
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key { /* one variant per UI string, see task 01 */ }

impl Key { pub const ALL: &'static [Key]; }        // every variant, used by the test

/// Templates may contain named placeholders like "{v}", filled by the caller with str::replace.
pub fn t(lang: Lang, key: Key) -> &'static str;
```
Implementation shape that makes the compiler enforce 3 translations:
```rust
pub fn t(lang: Lang, key: Key) -> &'static str {
    let [es, en, pt] = match key { Key::Verify => ["Verificar", "Check", "Verificar"], /* ... */ };
    match lang { Lang::Es => es, Lang::En => en, Lang::Pt => pt }
}
```

## `src/model.rs` change
```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorKind {
    Locked,               // "unlock the iPhone and tap Retry"
    ReadFailed(String),   // couldn't read the iPhone (detail) + manual help
    WatchTimeout,         // watch cap reached -> manual help
    Backend(String),      // other backend error, raw detail shown as-is
}
// AppState::Error(String)  ->  AppState::Error(ErrorKind)
```

## `ui::show` signature (after task 01)
```rust
pub fn show(ui: &mut egui::Ui, state: &AppState, log: &[String], lang: Lang) -> Option<Command>;
```
`App` gains `pub lang: Lang`, initialised with `Lang::from_env()`.
