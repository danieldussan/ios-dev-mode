# Spec: footer + i18n

## What
1. **Footer** pinned to the bottom of the fixed ~480x560 window: text "Hecho por danieldussan" (translated) + a GitHub icon. Both clickable, open https://github.com/danieldussan in the default browser. Never pushed off screen, even with all wizard steps shown and "Detalles" expanded.
2. **Language selector** (dropdown) with Español, English, Português.
   - Default from environment: `LC_ALL` / `LC_MESSAGES` / `LANG` (first non-empty) starting with `en` → English, `pt` → Português, anything else (or unset) → Español.
   - Switching is live: every visible string (header, body, steps, buttons, "Detalles", footer, the no-usbmuxd popup, window title) changes immediately. No restart, no persistence.

## Why
App is shared publicly; non-Spanish users need it in their language, and the author wants attribution.

## Acceptance criteria
- [ ] All user-facing strings come from one place and exist in es/en/pt; a test fails if any is empty, and adding a string without all 3 translations does not compile.
- [ ] Worker (poller/model) emits no human-language text in `AppState`; errors carry language-independent data. Raw technical log lines in "Detalles" may stay untranslated.
- [ ] Changing the dropdown while in any state (incl. an error) re-renders that state in the new language.
- [ ] The no-usbmuxd popup appears in the current language.
- [ ] Footer visible and clickable in every state; content above scrolls if it does not fit.
- [ ] Existing behavior and tests unchanged except for asserting keys/enums instead of Spanish text.
- [ ] No new crates.
- [ ] Manual (user eyeballs on real device): footer link opens browser, switching languages mid-flow works.
