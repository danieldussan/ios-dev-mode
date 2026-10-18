# ios-dev-mode

Small desktop app that walks you through revealing and enabling **Developer Mode** on an iPhone (iOS 16+), without a Mac or Xcode. Built with Rust, [egui](https://github.com/emilk/egui) and [`idevice`](https://crates.io/crates/idevice).

Languages: English · Español · Português (switch live from the footer). [Leer en español](README.es.md)

<p align="center"><img src="assets/activated.png" alt="Developer Mode enabled screen" width="320"></p>

## What it does

1. Detects the iPhone over USB (model, iOS version, current Developer Mode status).
2. Makes the **Developer Mode** switch appear in *Settings > Privacy & Security*.
3. Watches the phone while you turn it on in Settings and the iPhone restarts. It follows the reboot, the reconnection and the unlock, sends the confirmation request and checks the final status.

It is not 100% automatic: the iPhone never restarts by itself, and after a reboot iOS requires your passcode once. The app waits for you and tells you what to do at each step.

## Requirements

- iPhone connected by USB and **trusted** on this computer.
- **Windows:** iTunes (apple.com) or *Apple Devices* (Microsoft Store). They provide the `usbmuxd` service on `127.0.0.1:27015`.
- **Linux:** the `usbmuxd` package. It stops when no phone is attached and starts again when you plug one in.
- Rust 1.85+ only if you build from source.

## Usage

Download the binary for your system from the [Releases](../../releases) page, or run from source.

- **Windows:** run `ios-dev-mode-windows-x64.exe`.
- **Linux:** make it executable and run it (with the `usbmuxd` service available):

  ```
  chmod +x ios-dev-mode-linux-x64
  ./ios-dev-mode-linux-x64
  ```

From source:

```
cargo run --release
cargo run -- --mock     # try the UI without a phone
```

Typical flow:

1. Connect the iPhone, unlock it and tap **Trust**.
2. Press **Activate**. The app shows the option in Settings.
3. On the iPhone: *Settings > Privacy & Security > Developer Mode*, turn it on, then tap **Restart** when the iPhone asks.
4. After the reboot, unlock with your passcode and tap **Turn On** when asked.
5. The app detects it and shows *Developer Mode active*. If it does not move on, press **Verify now**.

## Tested

| Device | iOS | Result |
|---|---|---|
| iPhone XR | 18.7.9 | Full flow: reveal, manual toggle + reboot, passcode, confirmation, success |
| iPhone 13 | 27.0.1 | Status read while locked, Developer Mode already on |

Behavior on iOS 16/17 is untested (the `reveal` and `enable` calls are documented for iOS 18+). If they fail, the app falls back to manual instructions and keeps checking the status.

## Build

```
cargo build --release
cargo test && cargo clippy --all-targets -- -D warnings && cargo fmt --check
cargo run --example spike -- status   # console probe against a real phone
```

`idevice` is pinned to `=0.1.68`: it ships breaking changes on every minor release until 0.2.0. TLS uses the `ring` feature so Windows builds need no cmake or NASM.

## Notes

- Windows may show a SmartScreen warning because the `.exe` is unsigned.
- Not affiliated with Apple. Use it on your own devices.

## License

[MIT](LICENSE) · made by [danieldussan](https://github.com/danieldussan)
