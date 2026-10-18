# Spec: ios-dev-mode-app

## What
A small desktop GUI (final target: Windows `.exe`) that enables **Developer Mode** on a USB-connected iPhone with as few clicks as possible. The user plugs the phone in, presses one button, and follows on-screen steps; the app does everything that can be automated and tells the user exactly what to do on the phone for the rest (unlock, tap "Activar", enter passcode).

## Why
Enabling Developer Mode normally requires Xcode/macOS or hunting through iPhone settings. Windows users need a one-button tool.

## Scope (phases 0-3)
- Phase 0: console spike against a real iPhone to validate the library (user-run, gate).
- Phase 1: device backend (detect, status, reveal, enable, accept) with typed errors and timeouts.
- Phase 2: orchestrator that drives the full flow including the reboot and reconnect.
- Phase 3: wizard-style UI.
- Phases 4 (real-device test matrix) and 5 (Windows packaging) are a manual checklist, not tasks: see [manual-qa.md](manual-qa.md).

## Flow
detect usbmuxd → detect iPhone → read iOS → if Developer Mode already on: Success. Else reveal → enable (iPhone reboots and disappears) → wait reconnect → user unlocks → accept (dialog on phone) → user taps "Activar" + passcode → poll status until on → Success. If reveal/enable are not supported (iOS 16/17), show manual instructions and keep polling status.

## Acceptance criteria
1. No usbmuxd (no iTunes/Apple Devices) → popup explaining to install iTunes from Apple's site and restart the app.
2. usbmuxd up, no phone → screen "Conecta tu iPhone por USB, desbloquéalo y toca Confiar".
3. Phone not trusted / pairing fails → specific "toca Confiar en el iPhone" screen with retry.
4. Ready screen shows device name, model, iOS version and Developer Mode status.
5. iOS < 16 → message that the device does not need Developer Mode.
6. Developer Mode already on → green success screen "Modo desarrollador activo".
7. Activation shows a step list (done / current / pending) and a progress bar; survives the phone reboot without user action in the app.
8. Waiting-on-user steps show exact instructions in Spanish (neutral tuteo), including the path "Ajustes > Privacidad y seguridad > Modo de desarrollador" for the manual fallback.
9. Any timeout or failure → error screen with the reason, manual instructions and a "Reintentar" button.
10. Collapsible "Detalles" panel shows the raw log.
11. A `--mock` flag runs the whole UI flow without a phone.
12. Orchestrator logic is covered by unit tests with a mock backend (no phone needed).
13. Fixed ~480x560 window, one main button whose label changes per state ("Verificar", "Activar", "Reintentar").

## Non-goals
Multiple simultaneous devices (first device wins), macOS/Linux packaging, installers, auto-update, localization beyond Spanish.
