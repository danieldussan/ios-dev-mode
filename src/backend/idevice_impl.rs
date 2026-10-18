use super::DevModeBackend;
use crate::model::{BackendError, DeviceInfo};

pub struct IdeviceBackend;

impl IdeviceBackend {
    pub fn new() -> Self {
        Self
    }
}

impl Default for IdeviceBackend {
    fn default() -> Self {
        Self::new()
    }
}

fn nope<T>() -> Result<T, BackendError> {
    Err(BackendError::Other("not implemented".into()))
}

#[async_trait::async_trait]
impl DevModeBackend for IdeviceBackend {
    async fn present(&self) -> Result<bool, BackendError> {
        nope()
    }
    async fn detect(&self) -> Result<DeviceInfo, BackendError> {
        nope()
    }
    async fn status(&self) -> Result<bool, BackendError> {
        nope()
    }
    async fn reveal(&self) -> Result<(), BackendError> {
        nope()
    }
    async fn enable(&self) -> Result<(), BackendError> {
        nope()
    }
    async fn accept(&self) -> Result<(), BackendError> {
        nope()
    }
}
