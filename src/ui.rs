use crate::model::{AppState, Command, Phase, Step};
use eframe::egui::{self, Color32, ProgressBar, RichText};

const GREEN: Color32 = Color32::from_rgb(46, 160, 67);
const RED: Color32 = Color32::from_rgb(214, 59, 59);
const AMBER: Color32 = Color32::from_rgb(214, 148, 24);

pub fn main_button(s: &AppState) -> (&'static str, Option<Command>) {
    match s {
        AppState::Idle | AppState::Success | AppState::NotNeeded(_) => {
            ("Verificar", Some(Command::Check))
        }
        AppState::NoUsbmuxd | AppState::NoDevice | AppState::NotTrusted(_) | AppState::Error(_) => {
            ("Reintentar", Some(Command::Check))
        }
        AppState::Ready(_) => ("Activar", Some(Command::Activate)),
        AppState::Working(_) => ("Trabajando…", None),
        AppState::Waiting(_) => ("Ya reinicié / Verificar ahora", Some(Command::Verify)),
    }
}

pub fn step_label(s: Step) -> &'static str {
    match s {
        Step::Detect => "Detectar iPhone",
        Step::Reveal => "Mostrar opción en Ajustes",
        Step::Enable => "Activar modo desarrollador",
        Step::Reboot => "Reinicio del iPhone",
        Step::Reconnect => "Reconexión",
        Step::Accept => "Enviar confirmación",
        Step::Confirm => "Confirmar en el iPhone",
    }
}

/// (icon, title, color, body)
fn header(s: &AppState) -> (&'static str, &'static str, Color32, String) {
    let gray = Color32::GRAY;
    match s {
        AppState::Idle => (
            "📱",
            "Listo para verificar",
            gray,
            "Conecta tu iPhone por USB y toca Verificar.".into(),
        ),
        AppState::NoUsbmuxd => (
            "⚠",
            "Sin conexión con el iPhone",
            RED,
            "Conecta el cable y desbloquea el iPhone con tu código (tras un reinicio no responde hasta entonces). Si sigue igual, instala iTunes o Apple Devices desde la web de Apple. Se reintenta solo."
                .into(),
        ),
        AppState::NoDevice => (
            "🔌",
            "No hay iPhone",
            AMBER,
            "Conecta tu iPhone por USB, desbloquéalo y toca Confiar".into(),
        ),
        AppState::NotTrusted(_) => (
            "🔒",
            "iPhone sin confianza",
            AMBER,
            "Toca Confiar en el iPhone y vuelve a intentar".into(),
        ),
        AppState::NotNeeded(v) => (
            "✔",
            "No es necesario",
            gray,
            format!("Este iPhone (iOS {v}) no necesita modo desarrollador"),
        ),
        AppState::Ready(d) => (
            "📱",
            "iPhone detectado",
            AMBER,
            format!(
                "{} · {} · iOS {}\nEl modo desarrollador está desactivado.",
                d.name, d.product_type, d.ios_version
            ),
        ),
        AppState::Working(Step::Reconnect) => {
            ("⏳", "Trabajando…", AMBER, "Desbloquea tu iPhone".into())
        }
        AppState::Working(_) => (
            "⏳",
            "Trabajando…",
            AMBER,
            "No desconectes el cable.".into(),
        ),
        AppState::Waiting(Phase::Settings) => (
            "👆",
            "Actívalo en Ajustes",
            AMBER,
            "Abre Ajustes > Privacidad y seguridad > Modo de desarrollador, actívalo y reinicia el iPhone."
                .into(),
        ),
        AppState::Waiting(Phase::Rebooting) => (
            "⏳",
            "Reiniciando...",
            AMBER,
            "Espera a que el iPhone vuelva a encender. La conexión regresa sola.".into(),
        ),
        AppState::Waiting(Phase::Locked) => (
            "🔒",
            "Desbloquea el iPhone",
            AMBER,
            "Después de reiniciar, el iPhone necesita tu código. Ingrésalo en el iPhone y esta pantalla seguirá sola.".into(),
        ),
        AppState::Waiting(Phase::Confirm) => (
            "👆",
            "Falta tu confirmación",
            AMBER,
            "Confirma en el iPhone: toca Activar e ingresa tu código".into(),
        ),
        AppState::Success => (
            "✔",
            "Modo desarrollador activo",
            GREEN,
            "Todo listo.".into(),
        ),
        AppState::Error(m) => ("✖", "Algo salió mal", RED, m.clone()),
    }
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

pub fn show(ui: &mut egui::Ui, state: &AppState, log: &[String]) -> Option<Command> {
    let (icon, title, color, body) = header(state);
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
            ui.label(RichText::new(format!("{mark}  {}", step_label(st))).color(c));
        }
        ui.add_space(6.0);
        ui.add(ProgressBar::new(frac));
    }

    ui.add_space(12.0);
    let (label, cmd) = main_button(state);
    let mut out = None;
    ui.vertical_centered(|ui| {
        let main = ui.add_enabled(
            cmd.is_some(),
            egui::Button::new(RichText::new(label).size(18.0)).min_size(egui::vec2(200.0, 40.0)),
        );
        if main.clicked() {
            out = cmd;
        }
        if matches!(state, AppState::Waiting(_)) && ui.button("Cancelar").clicked() {
            out = Some(Command::Check);
        }
    });

    ui.add_space(8.0);
    egui::CollapsingHeader::new("Detalles").show(ui, |ui| {
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
    fn main_button_per_state() {
        let d = DeviceInfo {
            name: "".into(),
            product_type: "".into(),
            ios_version: "18".into(),
            udid: "".into(),
        };
        assert_eq!(
            main_button(&AppState::Idle),
            ("Verificar", Some(Command::Check))
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
            AppState::Error("e".into()),
        ] {
            assert_eq!(main_button(&s), ("Reintentar", Some(Command::Check)));
        }
        assert_eq!(
            main_button(&AppState::Ready(d)),
            ("Activar", Some(Command::Activate))
        );
        assert_eq!(
            main_button(&AppState::Working(Step::Enable)),
            ("Trabajando…", None)
        );
        assert_eq!(
            main_button(&AppState::Waiting(Phase::Settings)),
            ("Ya reinicié / Verificar ahora", Some(Command::Verify))
        );
    }
}
