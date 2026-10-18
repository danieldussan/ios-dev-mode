#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
mod app;
mod backend;
mod i18n;
mod model;
mod poller;
mod ui;

use backend::{DevModeBackend, idevice_impl::IdeviceBackend, mock::MockBackend};
use std::sync::Arc;

fn main() -> eframe::Result {
    let mock = std::env::args().any(|a| a == "--mock");
    let backend: Arc<dyn DevModeBackend> = if mock {
        Arc::new(MockBackend::demo())
    } else {
        Arc::new(IdeviceBackend::new())
    };
    let rt = tokio::runtime::Runtime::new().expect("tokio runtime");
    let handle = rt.handle().clone();

    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_title(i18n::t(i18n::Lang::from_env(), i18n::Key::WindowTitle))
            .with_inner_size([480.0, 560.0])
            .with_resizable(false)
            .with_icon(eframe::egui::IconData {
                rgba: include_bytes!("../assets/icon-128.rgba").to_vec(),
                width: 128,
                height: 128,
            }),
        ..Default::default()
    };
    eframe::run_native(
        "Modo desarrollador iOS",
        options,
        Box::new(move |cc| {
            let ctx = cc.egui_ctx.clone();
            egui_extras::install_image_loaders(&ctx);
            let (ev_tx, ev_rx) = std::sync::mpsc::channel();
            let (cmd_tx, cmd_rx) = tokio::sync::mpsc::unbounded_channel();
            handle.spawn(poller::run_worker(
                backend,
                cmd_rx,
                poller::PollConfig::default(),
                move |ev| {
                    let _ = ev_tx.send(ev);
                    ctx.request_repaint();
                },
            ));
            let _ = cmd_tx.send(model::Command::Check);
            Ok(Box::new(app::App::new(cmd_tx, ev_rx)))
        }),
    )
}
