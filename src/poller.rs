use crate::backend::DevModeBackend;
use crate::model::{AppState, BackendError, Command, Event, Step};
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::time::{Instant, sleep};

pub struct PollConfig {
    pub gone_cap: Duration,
    pub back_cap: Duration,
    pub ready_cap: Duration,
    pub status_cap: Duration,
    pub manual_cap: Duration,
    pub fast: Duration,
    pub slow: Duration,
}

impl Default for PollConfig {
    fn default() -> Self {
        Self {
            gone_cap: Duration::from_secs(20),
            back_cap: Duration::from_secs(90),
            ready_cap: Duration::from_secs(120),
            status_cap: Duration::from_secs(180),
            manual_cap: Duration::from_secs(300),
            fast: Duration::from_secs(1),
            slow: Duration::from_secs(3),
        }
    }
}

pub type Emit<'a> = &'a (dyn Fn(Event) + Sync);

const MANUAL_HELP: &str = "Abre Ajustes > Privacidad y seguridad > Modo de desarrollador, actívalo y reinicia el iPhone. Luego toca Reintentar.";

fn log(emit: Emit, step: Step, res: &dyn std::fmt::Debug) {
    emit(Event::Log(format!("{step:?}: {res:?}")));
}

fn state(emit: Emit, s: AppState) {
    emit(Event::State(s));
}

pub async fn check(b: &dyn DevModeBackend, emit: Emit<'_>) {
    state(emit, AppState::Working(Step::Detect));
    let det = b.detect().await;
    log(emit, Step::Detect, &det);
    let d = match det {
        Ok(d) => d,
        Err(e) => return state(emit, AppState::from_error(&e)),
    };
    if (1..16).contains(&d.ios_major()) {
        return state(emit, AppState::NotNeeded(d.ios_version));
    }
    let st = b.status().await;
    log(emit, Step::Confirm, &st);
    state(
        emit,
        match st {
            Ok(true) => AppState::Success,
            Ok(false) => AppState::Ready(d),
            Err(e) => AppState::from_error(&e),
        },
    );
}

/// Calls `f` every `every` until it yields Some or `cap` elapses.
async fn poll_until<T, Fut: Future<Output = Option<T>>>(
    cap: Duration,
    every: Duration,
    mut f: impl FnMut() -> Fut,
) -> Option<T> {
    let end = Instant::now() + cap;
    loop {
        if let Some(v) = f().await {
            return Some(v);
        }
        if Instant::now() >= end {
            return None;
        }
        sleep(every).await;
    }
}

async fn wait_status(b: &dyn DevModeBackend, cap: Duration, every: Duration) -> bool {
    poll_until(cap, every, || async {
        matches!(b.status().await, Ok(true)).then_some(())
    })
    .await
    .is_some()
}

async fn manual(b: &dyn DevModeBackend, cfg: &PollConfig, emit: Emit<'_>) {
    state(emit, AppState::WaitingUser { manual: true });
    let ok = wait_status(b, cfg.manual_cap, cfg.slow).await;
    state(
        emit,
        if ok {
            AppState::Success
        } else {
            AppState::Error(MANUAL_HELP.into())
        },
    );
}

fn fail(emit: Emit, msg: &str) {
    state(emit, AppState::Error(msg.into()));
}

pub async fn activate(b: &dyn DevModeBackend, cfg: &PollConfig, emit: Emit<'_>) {
    state(emit, AppState::Working(Step::Reveal));
    let r = b.reveal().await;
    log(emit, Step::Reveal, &r);
    if r.is_err() {
        return manual(b, cfg, emit).await;
    }

    state(emit, AppState::Working(Step::Enable));
    let r = b.enable().await;
    log(emit, Step::Enable, &r);
    if r.is_err() {
        return manual(b, cfg, emit).await;
    }

    state(emit, AppState::Working(Step::Reboot));
    let gone = poll_until(cfg.gone_cap, cfg.fast, || async {
        let p = b.present().await;
        matches!(p, Ok(false) | Err(BackendError::NoDevice)).then_some(())
    })
    .await;
    log(emit, Step::Reboot, &gone);
    if gone.is_none() {
        return fail(
            emit,
            "El iPhone no se reinició. Actívalo a mano: Ajustes > Privacidad y seguridad > Modo de desarrollador.",
        );
    }

    state(emit, AppState::Working(Step::Reconnect));
    let back = poll_until(cfg.back_cap, cfg.fast, || async {
        matches!(b.present().await, Ok(true)).then_some(())
    })
    .await;
    let ready = match back {
        // Any Err = still locked/booting, keep going.
        Some(()) => poll_until(cfg.ready_cap, cfg.slow, || async { b.detect().await.ok() }).await,
        None => None,
    };
    log(emit, Step::Reconnect, &ready.is_some());
    if ready.is_none() {
        return fail(
            emit,
            "El iPhone no volvió a conectarse. Desbloquéalo, revisa el cable y toca Reintentar.",
        );
    }

    state(emit, AppState::Working(Step::Accept));
    let r = b.accept().await;
    log(emit, Step::Accept, &r);
    if let Err(e) = r {
        return state(emit, AppState::Error(e.to_string()));
    }

    state(emit, AppState::WaitingUser { manual: false });
    if wait_status(b, cfg.status_cap, cfg.slow).await {
        state(emit, AppState::Success);
    } else {
        fail(
            emit,
            "No se confirmó a tiempo. En el iPhone toca Activar e ingresa tu código, luego toca Reintentar.",
        );
    }
}

pub async fn run_worker(
    b: Arc<dyn DevModeBackend>,
    mut rx: UnboundedReceiver<Command>,
    cfg: PollConfig,
    emit: impl Fn(Event) + Send + Sync + 'static,
) {
    while let Some(cmd) = rx.recv().await {
        match cmd {
            Command::Check => check(&*b, &emit).await,
            Command::Activate => activate(&*b, &cfg, &emit).await,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::mock::{MockBackend, Script};
    use crate::model::DeviceInfo;
    use std::sync::Mutex;

    fn dev(v: &str) -> DeviceInfo {
        DeviceInfo {
            name: "iPhone".into(),
            product_type: "iPhone15,2".into(),
            ios_version: v.into(),
            udid: "U".into(),
        }
    }

    type Sink = Mutex<Vec<Event>>;

    fn states(sink: &Sink) -> Vec<AppState> {
        sink.lock()
            .unwrap()
            .iter()
            .filter_map(|e| match e {
                Event::State(s) => Some(s.clone()),
                _ => None,
            })
            .collect()
    }

    async fn run_check(b: &MockBackend) -> Vec<AppState> {
        let sink = Sink::default();
        check(b, &|e| sink.lock().unwrap().push(e)).await;
        states(&sink)
    }

    async fn run_activate(b: &MockBackend) -> Vec<AppState> {
        let sink = Sink::default();
        activate(b, &PollConfig::default(), &|e| sink.lock().unwrap().push(e)).await;
        states(&sink)
    }

    #[tokio::test(start_paused = true)]
    async fn check_success_when_status_true() {
        let b = MockBackend::new(Script {
            detect: [Ok(dev("18.1"))].into(),
            status: [Ok(true)].into(),
            ..Script::default()
        });
        assert_eq!(run_check(&b).await.last(), Some(&AppState::Success));
    }

    #[tokio::test(start_paused = true)]
    async fn check_no_usbmuxd() {
        let b = MockBackend::new(Script {
            detect: [Err(BackendError::NoUsbmuxd)].into(),
            ..Script::default()
        });
        assert_eq!(run_check(&b).await.last(), Some(&AppState::NoUsbmuxd));
    }

    #[tokio::test(start_paused = true)]
    async fn check_old_ios_not_needed() {
        let b = MockBackend::new(Script {
            detect: [Ok(dev("15.7"))].into(),
            ..Script::default()
        });
        assert_eq!(
            run_check(&b).await.last(),
            Some(&AppState::NotNeeded("15.7".into()))
        );
    }

    #[tokio::test(start_paused = true)]
    async fn activate_happy_path() {
        let b = MockBackend::new(Script {
            present: [Ok(true), Ok(false), Ok(true)].into(),
            detect: [
                Err(BackendError::NotTrusted("locked".into())),
                Ok(dev("18.1")),
            ]
            .into(),
            status: [Ok(false), Ok(false), Ok(true)].into(),
            ..Script::default()
        });
        assert_eq!(run_activate(&b).await.last(), Some(&AppState::Success));
        let calls = b.calls();
        let pos = |n: &str| calls.iter().position(|c| *c == n).unwrap();
        assert!(pos("reveal") < pos("enable"));
        assert!(pos("enable") < pos("present"));
        assert!(pos("detect") < pos("accept"));
        assert!(pos("accept") < pos("status"));
        assert_eq!(calls.iter().filter(|c| **c == "detect").count(), 2);
    }

    #[tokio::test(start_paused = true)]
    async fn activate_never_reboots_errors() {
        let b = MockBackend::new(Script {
            present: [Ok(true)].into(),
            ..Script::default()
        });
        let s = run_activate(&b).await;
        assert!(matches!(s.last(), Some(AppState::Error(m)) if m.contains("no se reinició")));
    }

    #[tokio::test(start_paused = true)]
    async fn activate_status_never_true_errors() {
        let b = MockBackend::new(Script {
            present: [Ok(false), Ok(true)].into(),
            detect: [Ok(dev("18.1"))].into(),
            status: [Ok(false)].into(),
            ..Script::default()
        });
        let s = run_activate(&b).await;
        assert!(matches!(s.last(), Some(AppState::Error(m)) if m.contains("No se confirmó")));
    }

    #[tokio::test(start_paused = true)]
    async fn reveal_error_falls_back_to_manual() {
        let b = MockBackend::new(Script {
            reveal: Err(BackendError::Other("nope".into())),
            status: [Ok(false), Ok(true)].into(),
            ..Script::default()
        });
        let s = run_activate(&b).await;
        assert!(s.contains(&AppState::WaitingUser { manual: true }));
        assert_eq!(s.last(), Some(&AppState::Success));
    }
}
