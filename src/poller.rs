use crate::backend::DevModeBackend;
use crate::model::{AppState, BackendError, Command, ErrorKind, Event, Phase, Step};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::time::{Instant, sleep};

pub struct PollConfig {
    /// Poll period while watching the phone.
    pub tick: Duration,
    /// Min time between automatic `accept` attempts.
    pub accept_every: Duration,
    /// Total time before giving up on the watching phase.
    pub watch_cap: Duration,
}

impl Default for PollConfig {
    fn default() -> Self {
        Self {
            tick: Duration::from_secs(2),
            accept_every: Duration::from_secs(10),
            watch_cap: Duration::from_secs(15 * 60),
        }
    }
}

pub type Emit<'a> = &'a (dyn Fn(Event) + Sync);

fn log(emit: Emit, step: Step, res: &dyn std::fmt::Debug) {
    emit(Event::Log(format!("{step:?}: {res:?}")));
}

fn state(emit: Emit, s: AppState) {
    emit(Event::State(s));
}

/// Logs `msg` only when it differs from the previous one (no spam every tick).
fn note(emit: Emit, last: &mut String, msg: String) {
    if *last != msg {
        emit(Event::Log(msg.clone()));
        *last = msg;
    }
}

/// Sleeps one tick, or returns early on a command. Err(next) = leave the loop.
async fn pause(
    rx: &mut UnboundedReceiver<Command>,
    tick: Duration,
    force: &mut bool,
) -> Result<(), Option<Command>> {
    tokio::select! {
        c = rx.recv() => match c {
            None => Err(None),
            Some(Command::Verify) => { *force = true; Ok(()) }
            Some(Command::Activate) => Ok(()),
            Some(c @ Command::Check) => Err(Some(c)),
        },
        _ = sleep(tick) => Ok(()),
    }
}

fn check_error(e: &BackendError) -> AppState {
    match e {
        BackendError::NoUsbmuxd | BackendError::NoDevice | BackendError::NotTrusted(_) => {
            AppState::from_error(e)
        }
        other => AppState::Error(ErrorKind::ReadFailed(other.to_string())),
    }
}

/// Detect + status. While the phone is locked (always right after a reboot) shows the unlock
/// screen and retries every tick until it answers.
pub async fn check(
    b: &dyn DevModeBackend,
    cfg: &PollConfig,
    rx: &mut UnboundedReceiver<Command>,
    emit: Emit<'_>,
) -> Option<Command> {
    state(emit, AppState::Working(Step::Detect));
    let end = Instant::now() + cfg.watch_cap;
    let mut last_err = String::new();
    let mut last_shown: Option<BackendError> = None;
    loop {
        let det = b.detect().await;
        let res = match det {
            Err(e) => Err(e),
            Ok(d) if (1..16).contains(&d.ios_major()) => {
                state(emit, AppState::NotNeeded(d.ios_version));
                return None;
            }
            Ok(d) => b.status().await.map(|st| (d, st)),
        };
        match res {
            Ok((_, true)) => {
                state(emit, AppState::Success);
                return None;
            }
            Ok((d, false)) => {
                state(emit, AppState::Ready(d));
                return None;
            }
            Err(BackendError::Locked) => {
                note(
                    emit,
                    &mut last_err,
                    "Check: locked (PasswordProtected)".into(),
                );
                state(emit, AppState::Waiting(Phase::Locked));
                last_shown = None;
                if Instant::now() >= end {
                    state(emit, AppState::Error(ErrorKind::WatchTimeout));
                    return None;
                }
                if let Err(next) = pause(rx, cfg.tick, &mut false).await {
                    return next;
                }
            }
            // Daemon/phone absent: show the screen once, keep retrying until it returns.
            Err(e @ (BackendError::NoUsbmuxd | BackendError::NoDevice)) => {
                note(emit, &mut last_err, format!("Check: {e}"));
                if last_shown.as_ref() != Some(&e) {
                    state(emit, check_error(&e));
                    last_shown = Some(e);
                }
                if let Err(next) = pause(rx, cfg.tick.max(Duration::from_secs(3)), &mut false).await
                {
                    return next;
                }
            }
            Err(e) => {
                emit(Event::Log(format!("Check: {e}")));
                state(emit, check_error(&e));
                return None;
            }
        }
    }
}

/// Watches the phone until Developer Mode is on, whatever path the user takes
/// (Settings only, reboot, auto flow). `force` makes the first tick run detect + accept.
async fn watch(
    b: &dyn DevModeBackend,
    cfg: &PollConfig,
    rx: &mut UnboundedReceiver<Command>,
    emit: Emit<'_>,
    mut force: bool,
) -> Option<Command> {
    let end = Instant::now() + cfg.watch_cap;
    let mut phase = Phase::Settings;
    state(emit, AppState::Waiting(phase));
    let (mut gone, mut last_err) = (false, String::new());
    let mut last_accept: Option<Instant> = None;
    loop {
        if force {
            let d = b.detect().await;
            log(emit, Step::Detect, &d);
        }
        let mut next = phase;
        match b.present().await {
            // usbmuxd also dies while the phone reboots (Linux): same as "phone gone".
            Ok(false) | Err(BackendError::NoDevice | BackendError::NoUsbmuxd) => {
                gone = true;
                next = Phase::Rebooting;
            }
            Err(e) => note(emit, &mut last_err, format!("present: {e}")),
            Ok(true) => match b.status().await {
                Ok(true) => {
                    state(emit, AppState::Success);
                    return None;
                }
                Ok(false) => {
                    next = if gone || force {
                        Phase::Confirm
                    } else {
                        Phase::Settings
                    };
                    let due = last_accept.is_none_or(|t| t.elapsed() >= cfg.accept_every);
                    if next == Phase::Confirm && (force || due) {
                        last_accept = Some(Instant::now());
                        let r = b.accept().await;
                        log(emit, Step::Accept, &r);
                    }
                }
                Err(BackendError::Locked) => next = Phase::Locked,
                Err(e) => note(emit, &mut last_err, format!("status: {e}")),
            },
        }
        if next != phase {
            phase = next;
            state(emit, AppState::Waiting(phase));
        }
        force = false;
        if Instant::now() >= end {
            state(emit, AppState::Error(ErrorKind::WatchTimeout));
            return None;
        }
        if let Err(next) = pause(rx, cfg.tick, &mut force).await {
            return next;
        }
    }
}

pub async fn activate(
    b: &dyn DevModeBackend,
    cfg: &PollConfig,
    rx: &mut UnboundedReceiver<Command>,
    emit: Emit<'_>,
) -> Option<Command> {
    // Lost device is fatal; anything else (enable may drop the connection, usbmuxd may die
    // with the reboot) -> watch.
    let fatal = |r: &Result<(), BackendError>| matches!(r, Err(BackendError::NoDevice));
    state(emit, AppState::Working(Step::Reveal));
    let r = b.reveal().await;
    log(emit, Step::Reveal, &r);
    if fatal(&r) {
        state(emit, AppState::from_error(&r.unwrap_err()));
        return None;
    }
    if r.is_ok() {
        state(emit, AppState::Working(Step::Enable));
        let r = b.enable().await;
        log(emit, Step::Enable, &r);
        if fatal(&r) {
            state(emit, AppState::from_error(&r.unwrap_err()));
            return None;
        }
    }
    watch(b, cfg, rx, emit, false).await
}

pub async fn run_worker(
    b: Arc<dyn DevModeBackend>,
    mut rx: UnboundedReceiver<Command>,
    cfg: PollConfig,
    emit: impl Fn(Event) + Send + Sync + 'static,
) {
    let mut pending = None;
    loop {
        let cmd = match pending.take() {
            Some(c) => c,
            None => match rx.recv().await {
                Some(c) => c,
                None => return,
            },
        };
        pending = match cmd {
            Command::Check => check(&*b, &cfg, &mut rx, &emit).await,
            Command::Activate => activate(&*b, &cfg, &mut rx, &emit).await,
            Command::Verify => watch(&*b, &cfg, &mut rx, &emit, true).await,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::mock::{MockBackend, Script};
    use crate::model::DeviceInfo;
    use std::sync::Mutex;
    use tokio::sync::mpsc::unbounded_channel;

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
        let (_tx, mut rx) = unbounded_channel();
        check(b, &PollConfig::default(), &mut rx, &|e| {
            sink.lock().unwrap().push(e)
        })
        .await;
        states(&sink)
    }

    async fn run_activate(b: &MockBackend) -> Vec<AppState> {
        let sink = Sink::default();
        let (_tx, mut rx) = unbounded_channel();
        activate(b, &PollConfig::default(), &mut rx, &|e| {
            sink.lock().unwrap().push(e)
        })
        .await;
        states(&sink)
    }

    fn w(p: Phase) -> AppState {
        AppState::Waiting(p)
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
    async fn check_locked_retries_until_unlocked() {
        let b = MockBackend::new(Script {
            detect: [Ok(dev("18.1"))].into(),
            status: [
                Err(BackendError::Locked),
                Err(BackendError::Locked),
                Ok(true),
            ]
            .into(),
            ..Script::default()
        });
        let s = run_check(&b).await;
        assert!(s.contains(&w(Phase::Locked)));
        assert!(!s.iter().any(|x| matches!(x, AppState::Error(_))));
        assert_eq!(s.last(), Some(&AppState::Success));
    }

    #[tokio::test(start_paused = true)]
    async fn check_no_usbmuxd_shown_once_then_recovers() {
        let b = MockBackend::new(Script {
            detect: [
                Err(BackendError::NoUsbmuxd),
                Err(BackendError::NoUsbmuxd),
                Ok(dev("18.1")),
            ]
            .into(),
            status: [Ok(false)].into(),
            ..Script::default()
        });
        let s = run_check(&b).await;
        let n = s.iter().filter(|x| **x == AppState::NoUsbmuxd).count();
        assert_eq!(n, 1, "popup state emitted once");
        assert!(matches!(s.last(), Some(AppState::Ready(_))));
    }

    #[tokio::test(start_paused = true)]
    async fn usbmuxd_gap_during_reboot_never_errors() {
        let d = BackendError::NoUsbmuxd;
        let b = MockBackend::new(Script {
            enable: Err(BackendError::Timeout),
            present: [Ok(true), Err(d.clone()), Err(d.clone()), Err(d), Ok(true)].into(),
            status: [Err(BackendError::Locked), Ok(false), Ok(true)].into(),
            ..Script::default()
        });
        let s = run_activate(&b).await;
        assert!(s.contains(&w(Phase::Rebooting)));
        assert!(s.contains(&w(Phase::Locked)));
        assert!(
            !s.iter()
                .any(|x| matches!(x, AppState::Error(_) | AppState::NoUsbmuxd))
        );
        assert_eq!(s.last(), Some(&AppState::Success));
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
    async fn check_other_error_is_read_failed() {
        let b = MockBackend::new(Script {
            detect: [Err(BackendError::Other("boom".into()))].into(),
            ..Script::default()
        });
        let s = run_check(&b).await;
        assert!(
            matches!(s.last(), Some(AppState::Error(ErrorKind::ReadFailed(d))) if d.contains("boom"))
        );
    }

    #[tokio::test(start_paused = true)]
    async fn reboot_then_locked_then_confirm_then_success() {
        let b = MockBackend::new(Script {
            enable: Err(BackendError::Timeout),
            present: [Ok(true), Ok(false), Ok(true)].into(),
            status: [Err(BackendError::Locked), Ok(false), Ok(true)].into(),
            ..Script::default()
        });
        let s = run_activate(&b).await;
        for p in [Phase::Rebooting, Phase::Locked, Phase::Confirm] {
            assert!(s.contains(&w(p)), "missing {p:?}");
        }
        assert_eq!(s.last(), Some(&AppState::Success));
        assert!(b.calls().contains(&"accept"));
    }

    #[tokio::test(start_paused = true)]
    async fn settings_only_no_reboot_succeeds() {
        let b = MockBackend::new(Script {
            present: [Ok(true)].into(),
            status: [Ok(false), Ok(false), Ok(true)].into(),
            ..Script::default()
        });
        let s = run_activate(&b).await;
        assert!(!s.contains(&w(Phase::Rebooting)));
        assert_eq!(s.last(), Some(&AppState::Success));
        assert!(!b.calls().contains(&"accept"));
    }

    #[tokio::test(start_paused = true)]
    async fn locked_never_errors_before_cap() {
        let b = MockBackend::new(Script {
            present: [Ok(true)].into(),
            status: [Err(BackendError::Locked)].into(),
            ..Script::default()
        });
        let sink = Sink::default();
        let (_tx, mut rx) = unbounded_channel();
        let r = tokio::time::timeout(
            Duration::from_secs(14 * 60),
            activate(&b, &PollConfig::default(), &mut rx, &|e| {
                sink.lock().unwrap().push(e)
            }),
        )
        .await;
        assert!(r.is_err(), "should still be watching");
        assert!(
            !states(&sink)
                .iter()
                .any(|x| matches!(x, AppState::Error(_)))
        );
    }

    #[tokio::test(start_paused = true)]
    async fn cap_errors_with_help() {
        let b = MockBackend::new(Script {
            present: [Ok(true)].into(),
            status: [Ok(false)].into(),
            ..Script::default()
        });
        let s = run_activate(&b).await;
        assert_eq!(s.last(), Some(&AppState::Error(ErrorKind::WatchTimeout)));
    }

    #[tokio::test(start_paused = true)]
    async fn reveal_error_falls_back_to_watch() {
        let b = MockBackend::new(Script {
            reveal: Err(BackendError::Other("nope".into())),
            present: [Ok(true)].into(),
            status: [Ok(false), Ok(true)].into(),
            ..Script::default()
        });
        let s = run_activate(&b).await;
        assert!(s.contains(&w(Phase::Settings)));
        assert_eq!(s.last(), Some(&AppState::Success));
        assert!(!b.calls().contains(&"enable"));
    }

    #[tokio::test(start_paused = true)]
    async fn enable_no_device_is_fatal() {
        let b = MockBackend::new(Script {
            enable: Err(BackendError::NoDevice),
            ..Script::default()
        });
        assert_eq!(run_activate(&b).await.last(), Some(&AppState::NoDevice));
    }

    async fn run_cmds(b: MockBackend, cmds: &[Command]) -> (Vec<AppState>, Vec<&'static str>) {
        let b = Arc::new(b);
        let sink = Arc::new(Sink::default());
        let (tx, rx) = unbounded_channel();
        for c in cmds {
            tx.send(*c).unwrap();
        }
        drop(tx);
        let s2 = sink.clone();
        run_worker(b.clone(), rx, PollConfig::default(), move |e| {
            s2.lock().unwrap().push(e)
        })
        .await;
        (states(&sink), b.calls())
    }

    #[tokio::test(start_paused = true)]
    async fn verify_forces_detect_and_accept() {
        let b = MockBackend::new(Script {
            present: [Ok(true)].into(),
            status: [Ok(false)].into(),
            ..Script::default()
        });
        // Channel closes after the forced first tick, ending the watcher.
        let (s, calls) = run_cmds(b, &[Command::Verify]).await;
        assert_eq!(s.last(), Some(&w(Phase::Confirm)));
        assert!(calls.contains(&"detect") && calls.contains(&"accept"));
    }

    #[tokio::test(start_paused = true)]
    async fn verify_works_after_error_state() {
        let b = MockBackend::new(Script {
            detect: [Err(BackendError::Other("x".into()))].into(),
            present: [Ok(true)].into(),
            status: [Ok(true)].into(),
            ..Script::default()
        });
        let (s, _) = run_cmds(b, &[Command::Check, Command::Verify]).await;
        assert!(s.iter().any(|x| matches!(x, AppState::Error(_))));
        assert_eq!(s.last(), Some(&AppState::Success));
    }

    #[tokio::test(start_paused = true)]
    async fn check_while_watching_cancels() {
        let b = MockBackend::new(Script {
            detect: [Ok(dev("18.1"))].into(),
            present: [Ok(true)].into(),
            status: [Ok(false)].into(),
            ..Script::default()
        });
        let (s, _) = run_cmds(b, &[Command::Activate, Command::Check]).await;
        assert!(matches!(s.last(), Some(AppState::Ready(_))));
    }
}
