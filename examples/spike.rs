//! Manual spike against a real iPhone.
//!
//!   cargo run --example spike -- <info|status|reveal|enable|accept>
//!
//! Linux needs the `usbmuxd` daemon running and the phone trusted.
//! `enable` REBOOTS the phone; run `accept` after unlocking it post-reboot.

use idevice::{
    IdeviceService,
    amfi::AmfiClient,
    lockdown::LockdownClient,
    provider::IdeviceProvider,
    usbmuxd::{UsbmuxdAddr, UsbmuxdConnection},
};
use std::time::Instant;

macro_rules! timed {
    ($label:expr, $e:expr) => {{
        let t = Instant::now();
        let r = $e;
        println!("[{} ms] {}", t.elapsed().as_millis(), $label);
        r
    }};
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cmd = std::env::args().nth(1).unwrap_or_default();
    if !["info", "status", "reveal", "enable", "accept"].contains(&cmd.as_str()) {
        println!("Usage: spike <info|status|reveal|enable|accept>");
        return Ok(());
    }

    let mut mux = match timed!("usbmuxd connect", UsbmuxdConnection::default().await) {
        Ok(m) => m,
        Err(e) => {
            println!("usbmuxd error: {e:?}\nhint: is usbmuxd/iTunes running?");
            return Ok(());
        }
    };
    let devices = timed!("get_devices", mux.get_devices().await)?;
    println!("devices: {}", devices.len());
    let Some(dev) = devices.first() else {
        return Ok(());
    };
    let provider = dev.to_provider(UsbmuxdAddr::default(), "ios-dev-mode-spike");

    let mut lockdown = timed!("lockdown connect", LockdownClient::connect(&provider).await)?;
    let pairing = provider.get_pairing_file().await?;
    timed!("start_session", lockdown.start_session(&pairing).await)?;
    for key in [
        "ProductVersion",
        "ProductType",
        "DeviceName",
        "UniqueDeviceID",
    ] {
        let v = timed!(key, lockdown.get_value(Some(key), None).await)?;
        println!("{key}: {v:?}");
    }

    if cmd == "info" {
        return Ok(());
    }
    let mut amfi = timed!("amfi connect", AmfiClient::connect(&provider).await)?;
    match cmd.as_str() {
        "status" => report(timed!("status", amfi.get_developer_mode_status().await)),
        "reveal" => report(timed!(
            "reveal",
            amfi.reveal_developer_mode_option_in_ui().await
        )),
        "enable" => report(timed!("enable", amfi.enable_developer_mode().await)),
        "accept" => report(timed!("accept", amfi.accept_developer_mode().await)),
        _ => unreachable!(),
    }
    Ok(())
}

fn report<T: std::fmt::Debug, E: std::fmt::Debug>(r: Result<T, E>) {
    match r {
        Ok(v) => println!("Ok({v:?})"),
        Err(e) => println!("Err({e:?})"),
    }
}
