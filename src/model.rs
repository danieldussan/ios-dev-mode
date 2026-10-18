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
    #[error("device locked")]
    Locked,
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

/// What the user-facing watcher currently believes is happening on the phone.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    /// User must enable Developer Mode in Settings and restart.
    Settings,
    Rebooting,
    /// Phone is back but locked.
    Locked,
    /// Phone shows the "Turn On" dialog.
    Confirm,
}

impl Phase {
    /// Wizard step highlighted for this phase.
    pub fn step(self) -> Step {
        match self {
            Phase::Settings => Step::Enable,
            Phase::Rebooting => Step::Reboot,
            Phase::Locked => Step::Reconnect,
            Phase::Confirm => Step::Confirm,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorKind {
    Locked,
    ReadFailed(String),
    WatchTimeout,
    Backend(String),
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
    /// Watching the phone while the user acts on it.
    Waiting(Phase),
    Success,
    Error(ErrorKind),
}

impl AppState {
    pub fn from_error(e: &BackendError) -> Self {
        match e {
            BackendError::NoUsbmuxd => AppState::NoUsbmuxd,
            BackendError::NoDevice => AppState::NoDevice,
            BackendError::NotTrusted(m) => AppState::NotTrusted(m.clone()),
            BackendError::Locked => AppState::Error(ErrorKind::Locked),
            other => AppState::Error(ErrorKind::Backend(other.to_string())),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Check,
    Activate,
    /// Check now while watching: detect + status (+ accept).
    Verify,
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
            AppState::from_error(&Locked),
            AppState::Error(ErrorKind::Locked)
        );
        assert_eq!(
            AppState::from_error(&Timeout),
            AppState::Error(ErrorKind::Backend("timed out".into()))
        );
        assert_eq!(
            AppState::from_error(&Other("boom".into())),
            AppState::Error(ErrorKind::Backend("boom".into()))
        );
    }
}
