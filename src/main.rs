#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
// Real backend (task 02) constructs the error variants; remove once wired.
#![allow(dead_code)]
mod app;
mod backend;
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
            .with_title("Modo desarrollador iOS")
            .with_inner_size([480.0, 560.0])
            .with_resizable(false),
        ..Default::default()
    };
    eframe::run_native(
        "Modo desarrollador iOS",
        options,
        Box::new(move |cc| {
            let ctx = cc.egui_ctx.clone();
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
            Ok(Box::new(app::App::new(cmd_tx, ev_rx)))
        }),
    )
}
