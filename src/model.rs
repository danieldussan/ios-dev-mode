#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceInfo {
    pub name: String,
    pub product_type: String,
    pub ios_version: String,
    pub udid: String,
}

impl DeviceInfo {
    /// Major iOS version; 0 when unparsable (treated as "supported").
    pub fn ios_major(&self) -> u32 {
        self.ios_version
            .split('.')
            .next()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum BackendError {
    #[error("usbmuxd not reachable")]
    NoUsbmuxd,
    #[error("no device connected")]
    NoDevice,
    #[error("device not trusted: {0}")]
    NotTrusted(String),
    #[error("timed out")]
    Timeout,
    #[error("{0}")]
    Other(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Step {
    Detect,
    Reveal,
    Enable,
    Reboot,
    Reconnect,
    Accept,
    Confirm,
}

impl Step {
    pub const ALL: [Step; 7] = [
        Step::Detect,
        Step::Reveal,
        Step::Enable,
        Step::Reboot,
        Step::Reconnect,
        Step::Accept,
        Step::Confirm,
    ];

    pub fn index(self) -> usize {
        self as usize
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppState {
    Idle,
    NoUsbmuxd,
    NoDevice,
    NotTrusted(String),
    /// iOS version string, major < 16
    NotNeeded(String),
    /// Detected, Developer Mode OFF
    Ready(DeviceInfo),
    Working(Step),
    /// manual=true: user toggles in Settings
    WaitingUser {
        manual: bool,
    },
    Success,
    Error(String),
}

impl AppState {
    pub fn from_error(e: &BackendError) -> Self {
        match e {
            BackendError::NoUsbmuxd => AppState::NoUsbmuxd,
            BackendError::NoDevice => AppState::NoDevice,
            BackendError::NotTrusted(m) => AppState::NotTrusted(m.clone()),
            other => AppState::Error(other.to_string()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Check,
    Activate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    State(AppState),
    Log(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dev(v: &str) -> DeviceInfo {
        DeviceInfo {
            name: String::new(),
            product_type: String::new(),
            ios_version: v.into(),
            udid: String::new(),
        }
    }

    #[test]
    fn ios_major_parses() {
        assert_eq!(dev("17.5.1").ios_major(), 17);
        assert_eq!(dev("16").ios_major(), 16);
        assert_eq!(dev("").ios_major(), 0);
    }

    #[test]
    fn from_error_maps_all_variants() {
        use BackendError::*;
        assert_eq!(AppState::from_error(&NoUsbmuxd), AppState::NoUsbmuxd);
        assert_eq!(AppState::from_error(&NoDevice), AppState::NoDevice);
        assert_eq!(
            AppState::from_error(&NotTrusted("x".into())),
            AppState::NotTrusted("x".into())
        );
        assert_eq!(
            AppState::from_error(&Timeout),
            AppState::Error("timed out".into())
        );
        assert_eq!(
            AppState::from_error(&Other("boom".into())),
            AppState::Error("boom".into())
        );
    }
}
