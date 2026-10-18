use std::time::Duration;

use idevice::{
    IdeviceError, IdeviceService,
    amfi::AmfiClient,
    lockdown::LockdownClient,
    provider::IdeviceProvider,
    usbmuxd::{UsbmuxdAddr, UsbmuxdConnection},
};

use super::DevModeBackend;
use crate::model::{BackendError, DeviceInfo};

pub struct IdeviceBackend {
    timeout: Duration,
}

impl IdeviceBackend {
    pub fn new() -> Self {
        Self {
            timeout: Duration::from_secs(10),
        }
    }

    async fn with_timeout<T>(
        &self,
        fut: impl Future<Output = Result<T, BackendError>>,
    ) -> Result<T, BackendError> {
        tokio::time::timeout(self.timeout, fut)
            .await
            .unwrap_or(Err(BackendError::Timeout))
    }
}

impl Default for IdeviceBackend {
    fn default() -> Self {
        Self::new()
    }
}

async fn mux() -> Result<UsbmuxdConnection, BackendError> {
    UsbmuxdConnection::default()
        .await
        .map_err(|_| BackendError::NoUsbmuxd)
}

/// New usbmuxd connection every call: the phone drops sessions on reboot, so nothing is cached.
async fn provider() -> Result<impl IdeviceProvider, BackendError> {
    let devices = mux()
        .await?
        .get_devices()
        .await
        .map_err(|e| BackendError::Other(e.to_string()))?;
    let dev = devices.first().ok_or(BackendError::NoDevice)?;
    Ok(dev.to_provider(UsbmuxdAddr::default(), "ios-dev-mode"))
}

async fn amfi() -> Result<AmfiClient, BackendError> {
    AmfiClient::connect(&provider().await?)
        .await
        .map_err(map_err)
}

// Error mapping:
//   Locked:     DeviceLocked, or a message mentioning PasswordProtected/password (what lockdown and
//               AMFI return while the phone is locked).
//   NotTrusted: pairing/session/TLS failures (InvalidHostID, SessionInactive, plus messages
//               mentioning locked, pairing, PairRecord, trust, tls or certificate).
//   NoDevice:   DeviceNotFound / NotFound.
//   Timeout:    idevice's own Timeout.
//   Other:      everything else, with the original message.
fn map_err(e: IdeviceError) -> BackendError {
    match e {
        IdeviceError::DeviceNotFound | IdeviceError::NotFound => BackendError::NoDevice,
        IdeviceError::Timeout => BackendError::Timeout,
        IdeviceError::DeviceLocked => BackendError::Locked,
        IdeviceError::InvalidHostID | IdeviceError::SessionInactive => {
            BackendError::NotTrusted(e.to_string())
        }
        e => classify(e.to_string()),
    }
}

fn classify(msg: String) -> BackendError {
    let lower = msg.to_lowercase();
    let has = |ks: &[&str]| ks.iter().any(|k| lower.contains(k));
    if has(&["passwordprotected", "password"]) {
        BackendError::Locked
    } else if has(&[
        "locked",
        "pairing",
        "pairrecord",
        "trust",
        "tls",
        "certificate",
    ]) {
        BackendError::NotTrusted(msg)
    } else {
        BackendError::Other(msg)
    }
}

fn text(v: Option<plist::Value>) -> Option<String> {
    v.and_then(|v| v.as_string().map(str::to_owned))
}

#[async_trait::async_trait]
impl DevModeBackend for IdeviceBackend {
    async fn present(&self) -> Result<bool, BackendError> {
        self.with_timeout(async {
            let devices = mux()
                .await?
                .get_devices()
                .await
                .map_err(|e| BackendError::Other(e.to_string()))?;
            Ok(!devices.is_empty())
        })
        .await
    }

    async fn detect(&self) -> Result<DeviceInfo, BackendError> {
        self.with_timeout(async {
            let p = provider().await?;
            let mut ld = LockdownClient::connect(&p).await.map_err(map_err)?;
            let pairing = p
                .get_pairing_file()
                .await
                .map_err(|e| BackendError::NotTrusted(e.to_string()))?;
            ld.start_session(&pairing)
                .await
                .map_err(|e| BackendError::NotTrusted(e.to_string()))?;
            let mut get = async |key: &str| {
                ld.get_value(Some(key), None)
                    .await
                    .map(|v| text(Some(v)))
                    .map_err(map_err)
            };
            let ios_version = get("ProductVersion")
                .await?
                .ok_or_else(|| BackendError::Other("missing ProductVersion".into()))?;
            Ok(DeviceInfo {
                name: get("DeviceName").await?.unwrap_or_default(),
                product_type: get("ProductType").await?.unwrap_or_default(),
                ios_version,
                udid: get("UniqueDeviceID").await?.unwrap_or_default(),
            })
        })
        .await
    }

    async fn status(&self) -> Result<bool, BackendError> {
        self.with_timeout(async {
            amfi()
                .await?
                .get_developer_mode_status()
                .await
                .map_err(map_err)
        })
        .await
    }

    async fn reveal(&self) -> Result<(), BackendError> {
        self.with_timeout(async {
            amfi()
                .await?
                .reveal_developer_mode_option_in_ui()
                .await
                .map_err(map_err)
        })
        .await
    }

    async fn enable(&self) -> Result<(), BackendError> {
        // ponytail: spike did not verify `enable`; if it errors with a disconnect on success
        // (device reboots), map that specific error to Ok(()) here once observed.
        self.with_timeout(async { amfi().await?.enable_developer_mode().await.map_err(map_err) })
            .await
    }

    async fn accept(&self) -> Result<(), BackendError> {
        self.with_timeout(async { amfi().await?.accept_developer_mode().await.map_err(map_err) })
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map_err_groups() {
        assert!(matches!(
            map_err(IdeviceError::InvalidHostID),
            BackendError::NotTrusted(_)
        ));
        assert_eq!(map_err(IdeviceError::DeviceLocked), BackendError::Locked);
        assert_eq!(
            map_err(IdeviceError::DeviceNotFound),
            BackendError::NoDevice
        );
        assert_eq!(map_err(IdeviceError::Timeout), BackendError::Timeout);
        assert!(matches!(
            map_err(IdeviceError::MalformedCommand),
            BackendError::Other(_)
        ));
    }

    #[test]
    fn classify_messages() {
        let c = |m: &str| classify(m.into());
        assert_eq!(
            c("unknown error 'PasswordProtected' returned from device"),
            BackendError::Locked
        );
        assert!(matches!(
            c("UnexpectedResponse PairRecord"),
            BackendError::NotTrusted(_)
        ));
        assert!(matches!(c("device is locked"), BackendError::NotTrusted(_)));
        assert!(matches!(c("block device"), BackendError::Other(_)));
    }

    #[tokio::test(start_paused = true)]
    async fn with_timeout_elapses() {
        let b = IdeviceBackend::new();
        let r: Result<(), _> = b.with_timeout(std::future::pending()).await;
        assert_eq!(r, Err(BackendError::Timeout));
    }
}
