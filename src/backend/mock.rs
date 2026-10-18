use super::DevModeBackend;
use crate::model::{BackendError, DeviceInfo};
use std::collections::VecDeque;
use std::sync::Mutex;
use std::time::Duration;

pub struct Script {
    pub present: VecDeque<Result<bool, BackendError>>,
    pub detect: VecDeque<Result<DeviceInfo, BackendError>>,
    pub status: VecDeque<Result<bool, BackendError>>,
    pub reveal: Result<(), BackendError>,
    pub enable: Result<(), BackendError>,
    pub accept: Result<(), BackendError>,
}

impl Default for Script {
    fn default() -> Self {
        Self {
            present: VecDeque::new(),
            detect: VecDeque::new(),
            status: VecDeque::new(),
            reveal: Ok(()),
            enable: Ok(()),
            accept: Ok(()),
        }
    }
}

pub struct MockBackend {
    script: Mutex<Script>,
    calls: Mutex<Vec<&'static str>>,
    delay: Duration,
}

/// Pops while more than one entry remains; the last entry repeats forever.
fn next<T: Clone>(q: &mut VecDeque<Result<T, BackendError>>) -> Result<T, BackendError> {
    match q.len() {
        0 => Err(BackendError::Other("mock: empty script".into())),
        1 => q[0].clone(),
        _ => q.pop_front().unwrap(),
    }
}

impl MockBackend {
    pub fn new(script: Script) -> Self {
        Self {
            script: Mutex::new(script),
            calls: Mutex::new(Vec::new()),
            delay: Duration::ZERO,
        }
    }

    pub fn with_delay(mut self, d: Duration) -> Self {
        self.delay = d;
        self
    }

    #[cfg(test)]
    pub fn calls(&self) -> Vec<&'static str> {
        self.calls.lock().unwrap().clone()
    }

    pub fn demo() -> Self {
        let dev = DeviceInfo {
            name: "iPhone de prueba".into(),
            product_type: "iPhone15,2".into(),
            ios_version: "18.1".into(),
            udid: "0000-DEMO".into(),
        };
        Self::new(Script {
            present: [Ok(false), Ok(true)].into(),
            detect: [Ok(dev)].into(),
            status: [Ok(false), Ok(false), Ok(true)].into(),
            ..Script::default()
        })
        .with_delay(Duration::from_millis(600))
    }

    async fn enter(&self, name: &'static str) {
        if !self.delay.is_zero() {
            tokio::time::sleep(self.delay).await;
        }
        self.calls.lock().unwrap().push(name);
    }
}

#[async_trait::async_trait]
impl DevModeBackend for MockBackend {
    async fn present(&self) -> Result<bool, BackendError> {
        self.enter("present").await;
        next(&mut self.script.lock().unwrap().present)
    }
    async fn detect(&self) -> Result<DeviceInfo, BackendError> {
        self.enter("detect").await;
        next(&mut self.script.lock().unwrap().detect)
    }
    async fn status(&self) -> Result<bool, BackendError> {
        self.enter("status").await;
        next(&mut self.script.lock().unwrap().status)
    }
    async fn reveal(&self) -> Result<(), BackendError> {
        self.enter("reveal").await;
        self.script.lock().unwrap().reveal.clone()
    }
    async fn enable(&self) -> Result<(), BackendError> {
        self.enter("enable").await;
        self.script.lock().unwrap().enable.clone()
    }
    async fn accept(&self) -> Result<(), BackendError> {
        self.enter("accept").await;
        self.script.lock().unwrap().accept.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock(status: Vec<Result<bool, BackendError>>) -> MockBackend {
        MockBackend::new(Script {
            status: status.into(),
            ..Script::default()
        })
    }

    #[tokio::test]
    async fn last_entry_repeats() {
        let m = mock(vec![Ok(false), Ok(true)]);
        assert_eq!(m.status().await, Ok(false));
        assert_eq!(m.status().await, Ok(true));
        assert_eq!(m.status().await, Ok(true));
    }

    #[tokio::test]
    async fn empty_script_errs() {
        let m = mock(vec![]);
        assert_eq!(
            m.status().await,
            Err(BackendError::Other("mock: empty script".into()))
        );
    }

    #[tokio::test]
    async fn calls_recorded_in_order() {
        let m = mock(vec![Ok(true)]);
        m.reveal().await.unwrap();
        m.status().await.unwrap();
        m.enable().await.unwrap();
        assert_eq!(m.calls(), vec!["reveal", "status", "enable"]);
    }

    #[tokio::test]
    async fn actions_default_ok() {
        let m = mock(vec![]);
        assert_eq!(m.reveal().await, Ok(()));
        assert_eq!(m.accept().await, Ok(()));
    }
}
