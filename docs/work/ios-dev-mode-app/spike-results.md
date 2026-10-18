# Spike results (phase 0, filled by the user)

Run on Linux with `usbmuxd` running (`sudo systemctl start usbmuxd`), phone unlocked and trusted:

```
cargo run --example spike -- info
cargo run --example spike -- status
cargo run --example spike -- reveal
cargo run --example spike -- enable    # phone reboots; time it until it shows up again
cargo run --example spike -- accept    # after unlocking post-reboot
cargo run --example spike -- status    # after tapping "Activar" + passcode
```

| Field | Value |
|---|---|
| iPhone model / ProductType | |
| iOS version | |
| Passcode set? | |
| `info` (lockdown session) result | |
| `status` result before | |
| `reveal` result | |
| `enable` result (Ok / exact error) | |
| Reboot: seconds until device listed again | |
| `accept` result; dialog shown on phone? | |
| `status` result after confirm | |
| Errors seen when phone locked / untrusted (exact text) | |
| Did `ring` TLS work? | |

## Decision
- [ ] `status()` and `enable()` work → continue with task 02 (idevice).
- [ ] They fail → plan B (libimobiledevice binaries behind the same trait); replan task 02.

Notes:
