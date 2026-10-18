use crate::i18n::{Key, Lang, t};
use crate::model::{AppState, Command, ErrorKind, Phase, Step};
use eframe::egui::{self, Align, Color32, Layout, ProgressBar, RichText};

const GREEN: Color32 = Color32::from_rgb(46, 160, 67);
const RED: Color32 = Color32::from_rgb(214, 59, 59);
const AMBER: Color32 = Color32::from_rgb(214, 148, 24);

pub fn main_button(s: &AppState) -> (Key, Option<Command>) {
    match s {
        AppState::Idle | AppState::Success | AppState::NotNeeded(_) => {
            (Key::BtnVerify, Some(Command::Check))
        }
        AppState::NoUsbmuxd | AppState::NoDevice | AppState::NotTrusted(_) | AppState::Error(_) => {
            (Key::BtnRetry, Some(Command::Check))
        }
        AppState::Ready(_) => (Key::BtnActivate, Some(Command::Activate)),
        AppState::Working(_) => (Key::BtnWorking, None),
        AppState::Waiting(_) => (Key::BtnVerifyNow, Some(Command::Verify)),
    }
}

pub fn step_label(s: Step) -> Key {
    match s {
        Step::Detect => Key::StepDetect,
        Step::Reveal => Key::StepReveal,
        Step::Enable => Key::StepEnable,
        Step::Reboot => Key::StepReboot,
        Step::Reconnect => Key::StepReconnect,
        Step::Accept => Key::StepAccept,
        Step::Confirm => Key::StepConfirm,
    }
}

/// (icon, title, color, body)
fn header(s: &AppState, lang: Lang) -> (&'static str, &'static str, Color32, String) {
    let gray = Color32::GRAY;
    let tr = |k| t(lang, k);
    let (icon, title, color, body) = match s {
        AppState::Idle => ("📱", Key::TitleIdle, gray, tr(Key::BodyIdle).to_string()),
        AppState::NoUsbmuxd => (
            "⚠",
            Key::TitleNoUsbmuxd,
            RED,
            tr(Key::BodyNoUsbmuxd).to_string(),
        ),
        AppState::NoDevice => (
            "🔌",
            Key::TitleNoDevice,
            AMBER,
            tr(Key::BodyNoDevice).to_string(),
        ),
        AppState::NotTrusted(_) => (
            "🔒",
            Key::TitleNotTrusted,
            AMBER,
            tr(Key::BodyNotTrusted).to_string(),
        ),
        AppState::NotNeeded(v) => (
            "✔",
            Key::TitleNotNeeded,
            gray,
            tr(Key::BodyNotNeeded).replace("{v}", v),
        ),
        AppState::Ready(d) => (
            "📱",
            Key::TitleReady,
            AMBER,
            tr(Key::BodyReady)
                .replace("{name}", &d.name)
                .replace("{model}", &d.product_type)
                .replace("{v}", &d.ios_version),
        ),
        AppState::Working(Step::Reconnect) => (
            "⏳",
            Key::TitleWorking,
            AMBER,
            tr(Key::BodyUnlockPhone).to_string(),
        ),
        AppState::Working(_) => (
            "⏳",
            Key::TitleWorking,
            AMBER,
            tr(Key::BodyDontUnplug).to_string(),
        ),
        AppState::Waiting(Phase::Settings) => (
            "👆",
            Key::TitleSettings,
            AMBER,
            tr(Key::BodySettings).to_string(),
        ),
        AppState::Waiting(Phase::Rebooting) => (
            "⏳",
            Key::TitleRebooting,
            AMBER,
            tr(Key::BodyRebooting).to_string(),
        ),
        AppState::Waiting(Phase::Locked) => (
            "🔒",
            Key::TitleLocked,
            AMBER,
            tr(Key::BodyLocked).to_string(),
        ),
        AppState::Waiting(Phase::Confirm) => (
            "👆",
            Key::TitleConfirm,
            AMBER,
            tr(Key::BodyConfirm).to_string(),
        ),
        AppState::Success => (
            "✔",
            Key::TitleSuccess,
            GREEN,
            tr(Key::BodySuccess).to_string(),
        ),
        AppState::Error(kind) => {
            let body = match kind {
                ErrorKind::Locked => tr(Key::ErrLocked).to_string(),
                ErrorKind::ReadFailed(d) => format!(
                    "{} {}",
                    tr(Key::ErrReadFailed).replace("{detail}", d),
                    tr(Key::ManualHelp)
                ),
                ErrorKind::WatchTimeout => tr(Key::ManualHelp).to_string(),
                ErrorKind::Backend(d) => d.clone(),
            };
            ("✖", Key::ErrTitle, RED, body)
        }
    };
    (icon, t(lang, title), color, body)
}

/// Fraction of steps done, for the progress bar. None = hide the bar.
fn progress(s: &AppState) -> Option<(usize, f32)> {
    let cur = match s {
        AppState::Working(Step::Detect) => return None,
        AppState::Working(st) => st.index(),
        AppState::Waiting(p) => p.step().index(),
        AppState::Success => Step::ALL.len(),
        _ => return None,
    };
    Some((cur, cur as f32 / Step::ALL.len() as f32))
}

const GITHUB_URL: &str = "https://github.com/danieldussan";

pub fn footer(ui: &mut egui::Ui, lang: &mut Lang) {
    ui.add_space(4.0);
    ui.horizontal(|ui| {
        ui.hyperlink_to(t(*lang, Key::MadeBy), GITHUB_URL);
        let icon = ui
            .add(
                egui::Image::new(egui::include_image!("../assets/github.svg"))
                    .fit_to_exact_size(egui::vec2(18.0, 18.0))
                    .tint(ui.visuals().text_color())
                    .sense(egui::Sense::click()),
            )
            .on_hover_cursor(egui::CursorIcon::PointingHand);
        if icon.clicked() {
            ui.ctx().open_url(egui::OpenUrl::new_tab(GITHUB_URL));
        }
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            egui::ComboBox::from_id_salt("lang")
                .selected_text(lang.name())
                .show_ui(ui, |ui| {
                    for l in Lang::ALL {
                        ui.selectable_value(lang, l, l.name());
                    }
                })
                .response
                .on_hover_text(t(*lang, Key::Language));
        });
    });
    ui.add_space(4.0);
}

pub fn show(ui: &mut egui::Ui, state: &AppState, log: &[String], lang: Lang) -> Option<Command> {
    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| show_inner(ui, state, log, lang))
        .inner
}

fn show_inner(ui: &mut egui::Ui, state: &AppState, log: &[String], lang: Lang) -> Option<Command> {
    let (icon, title, color, body) = header(state, lang);
    ui.add_space(12.0);
    ui.vertical_centered(|ui| {
        ui.label(RichText::new(icon).size(48.0).color(color));
        ui.label(RichText::new(title).size(24.0).strong().color(color));
        ui.add_space(6.0);
        ui.label(RichText::new(body).size(15.0));
    });
    ui.add_space(12.0);

    if let Some((cur, frac)) = progress(state) {
        for st in Step::ALL {
            let (mark, c) = match st.index().cmp(&cur) {
                std::cmp::Ordering::Less => ("✔", GREEN),
                std::cmp::Ordering::Equal if *state != AppState::Success => ("▶", AMBER),
                _ => ("○", Color32::GRAY),
            };
            ui.label(RichText::new(format!("{mark}  {}", t(lang, step_label(st)))).color(c));
        }
        ui.add_space(6.0);
        ui.add(ProgressBar::new(frac));
    }

    ui.add_space(12.0);
    let (label, cmd) = main_button(state);
    let label = t(lang, label);
    let mut out = None;
    ui.vertical_centered(|ui| {
        let main = ui.add_enabled(
            cmd.is_some(),
            egui::Button::new(RichText::new(label).size(18.0)).min_size(egui::vec2(200.0, 40.0)),
        );
        if main.clicked() {
            out = cmd;
        }
        if matches!(state, AppState::Waiting(_)) && ui.button(t(lang, Key::Cancel)).clicked() {
            out = Some(Command::Check);
        }
    });

    ui.add_space(8.0);
    egui::CollapsingHeader::new(t(lang, Key::Details))
        .id_salt("details")
        .show(ui, |ui| {
            egui::ScrollArea::vertical()
                .max_height(120.0)
                .stick_to_bottom(true)
                .show(ui, |ui| {
                    for l in log {
                        ui.monospace(l);
                    }
                });
        });

    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::DeviceInfo;

    #[test]
    fn github_svg_embedded() {
        let svg = include_str!("../assets/github.svg");
        assert!(!svg.is_empty() && svg.contains("<path"));
    }

    #[test]
    fn main_button_per_state() {
        let d = DeviceInfo {
            name: "".into(),
            product_type: "".into(),
            ios_version: "18".into(),
            udid: "".into(),
        };
        assert_eq!(
            main_button(&AppState::Idle),
            (Key::BtnVerify, Some(Command::Check))
        );
        assert_eq!(main_button(&AppState::Success).1, Some(Command::Check));
        assert_eq!(
            main_button(&AppState::NotNeeded("15".into())).1,
            Some(Command::Check)
        );
        for s in [
            AppState::NoUsbmuxd,
            AppState::NoDevice,
            AppState::NotTrusted("x".into()),
            AppState::Error(ErrorKind::Locked),
        ] {
            assert_eq!(main_button(&s), (Key::BtnRetry, Some(Command::Check)));
        }
        assert_eq!(
            main_button(&AppState::Ready(d)),
            (Key::BtnActivate, Some(Command::Activate))
        );
        assert_eq!(
            main_button(&AppState::Working(Step::Enable)),
            (Key::BtnWorking, None)
        );
        assert_eq!(
            main_button(&AppState::Waiting(Phase::Settings)),
            (Key::BtnVerifyNow, Some(Command::Verify))
        );
    }
}
