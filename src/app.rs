use crate::i18n::{Key, Lang, t};
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
    pub lang: Lang,
}

impl App {
    pub fn new(cmd_tx: UnboundedSender<Command>, ev_rx: Receiver<Event>) -> Self {
        Self {
            state: AppState::Idle,
            log: Vec::new(),
            cmd_tx,
            ev_rx,
            warned_usbmuxd: false,
            lang: Lang::from_env(),
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
            cmd = ui::show(ui, &self.state, &self.log, self.lang);
        });
        if let Some(c) = cmd {
            let _ = self.cmd_tx.send(c);
        }

        if self.state == AppState::NoUsbmuxd && !self.warned_usbmuxd {
            self.warned_usbmuxd = true;
            let (yes, no) = (
                t(self.lang, Key::PopupOpen).to_string(),
                t(self.lang, Key::PopupClose).to_string(),
            );
            let open = rfd::MessageDialog::new()
                .set_level(rfd::MessageLevel::Warning)
                .set_title(t(self.lang, Key::TitleNoUsbmuxd))
                .set_description(t(self.lang, Key::PopupDesc))
                .set_buttons(rfd::MessageButtons::OkCancelCustom(yes.clone(), no))
                .show();
            if matches!(open, rfd::MessageDialogResult::Custom(s) if s == yes) {
                ui.ctx().open_url(egui::OpenUrl::new_tab(ITUNES_URL));
            }
        }
    }
}
