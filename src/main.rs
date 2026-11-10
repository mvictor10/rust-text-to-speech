#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(target_os = "windows")]
mod app;
#[cfg(target_os = "windows")]
mod domain;
#[cfg(target_os = "windows")]
mod services;

#[cfg(target_os = "windows")]
fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1120.0, 820.0])
            .with_min_inner_size([900.0, 680.0])
            .with_title("Rust Voice Studio"),
        ..Default::default()
    };

    eframe::run_native(
        "Rust Voice Studio",
        options,
        Box::new(|creation_context| Ok(Box::new(app::App::new(creation_context)))),
    )
}

#[cfg(not(target_os = "windows"))]
fn main() {
    eprintln!("Rust Voice Studio foi projetado para Windows 10/11 e utiliza a API SAPI.");
}
