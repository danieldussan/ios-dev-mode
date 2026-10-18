use crate::model::{AppState, Command, Event};
use crate::ui;
use eframe::egui;
use std::sync::mpsc::Receiver;
use tokio::sync::mpsc::UnboundedSender;

const ITUNES_URL: &str = "https://www.apple.com/itunes/";

pub struct App {
    pub state: AppState,
    pub log: Vec<String>,
    pub cmd_tx: UnboundedSender<Command>,
    pub ev_rx: Receiver<Event>,
    pub warned_usbmuxd: bool,
}

impl App {
    pub fn new(cmd_tx: UnboundedSender<Command>, ev_rx: Receiver<Event>) -> Self {
        Self {
            state: AppState::Idle,
            log: Vec::new(),
            cmd_tx,
            ev_rx,
            warned_usbmuxd: false,
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        for ev in self.ev_rx.try_iter() {
            match ev {
                Event::State(s) => self.state = s,
                Event::Log(l) => self.log.push(l),
            }
        }

        let mut cmd = None;
        egui::CentralPanel::default().show(ui, |ui| {
            cmd = ui::show(ui, &self.state, &self.log);
        });
        if let Some(c) = cmd {
            let _ = self.cmd_tx.send(c);
        }

        if self.state == AppState::NoUsbmuxd && !self.warned_usbmuxd {
            self.warned_usbmuxd = true;
            let open = rfd::MessageDialog::new()
                .set_level(rfd::MessageLevel::Warning)
                .set_title("Sin conexión con el iPhone")
                .set_description("Conecta el cable y desbloquea el iPhone con tu código (tras un reinicio no responde hasta entonces). Si sigue igual, instala iTunes o Apple Devices desde la web de Apple. La app reintenta sola.")
                .set_buttons(rfd::MessageButtons::OkCancelCustom("Abrir página de Apple".into(), "Cerrar".into()))
                .show();
            if matches!(open, rfd::MessageDialogResult::Custom(s) if s == "Abrir página de Apple")
            {
                ui.ctx().open_url(egui::OpenUrl::new_tab(ITUNES_URL));
            }
        }
    }
}
