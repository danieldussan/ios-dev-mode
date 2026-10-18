pub mod idevice_impl;
pub mod mock;
use crate::model::{BackendError, DeviceInfo};

#[async_trait::async_trait]
pub trait DevModeBackend: Send + Sync {
    /// Ok(true) if usbmuxd lists >= 1 device, Ok(false) if none, Err(NoUsbmuxd) if daemon down.
    async fn present(&self) -> Result<bool, BackendError>;
    /// First device + lockdown session + DeviceName/ProductType/ProductVersion/UniqueDeviceID.
    async fn detect(&self) -> Result<DeviceInfo, BackendError>;
    async fn status(&self) -> Result<bool, BackendError>;
    async fn reveal(&self) -> Result<(), BackendError>;
    /// Reboots the device on success.
    async fn enable(&self) -> Result<(), BackendError>;
    /// Shows the confirm dialog on the device.
    async fn accept(&self) -> Result<(), BackendError>;
}
