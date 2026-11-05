#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui::{self, Color32, RichText, Vec2};
use std::{path::PathBuf, process::{Command, Stdio}, sync::mpsc::{self, Receiver}, thread};

#[derive(Default)]
struct CrocApp {
    files: Vec<PathBuf>,
    code: String,
    croc_path: String,
    destination: Option<PathBuf>,
    status: String,
    running: bool,
    log: String,
    events: Option<Receiver<String>>,
}

impl CrocApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        Self { croc_path: "croc".into(), status: "Listo para transferir".into(), ..Default::default() }
    }

    fn start_send(&mut self) {
        if self.files.is_empty() || self.running { return; }
        let paths = self.files.clone();
        self.launch(move |c| {
            c.arg("send");
            for path in paths { c.arg(path); }
        }, "Enviando archivos…");
    }

    fn start_receive(&mut self) {
        let code = self.code.trim().to_owned();
        if code.is_empty() || self.running { return; }
        let destination = self.destination.clone();
        self.launch(move |c| {
            c.arg(code);
            if let Some(dir) = destination { c.arg("--out").arg(dir); }
        }, "Recibiendo archivos…");
    }

    fn launch<F>(&mut self, configure: F, label: &str)
    where F: FnOnce(&mut Command) + Send + 'static {
        let executable = if self.croc_path.trim().is_empty() { "croc".into() } else { self.croc_path.trim().into() };
        let (tx, rx) = mpsc::channel();
        self.events = Some(rx);
        self.running = true;
        self.status = label.into();
        self.log.clear();
        thread::spawn(move || {
            let mut command = Command::new(executable);
            configure(&mut command);
            command.stdout(Stdio::piped()).stderr(Stdio::piped());
            let result = command.output();
            let message = match result {
                Ok(output) => format!("{}\n{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr)),
                Err(error) => format!("No se pudo iniciar croc: {error}"),
            };
            let _ = tx.send(message);
        });
    }
}

impl eframe::App for CrocApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Some(rx) = self.events.take() {
            if let Ok(message) = rx.try_recv() {
                self.log = message;
                self.running = false;
                self.status = "Transferencia finalizada".into();
            } else {
                self.events = Some(rx);
            }
            ctx.request_repaint();
        }
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.add_space(24.0);
            ui.horizontal(|ui| {
                ui.heading(RichText::new("croc").size(30.0).strong().color(Color32::from_rgb(104, 135, 255)));
                ui.label(RichText::new("transferencias simples y privadas").color(Color32::GRAY));
            });
            ui.add_space(18.0);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 12.0;
                if ui.selectable_label(self.files.len() > 0, "↑  Enviar").clicked() { self.code.clear(); }
                if ui.selectable_label(!self.code.is_empty(), "↓  Recibir").clicked() { self.files.clear(); }
            });
            ui.add_space(12.0);
            ui.columns(2, |columns| {
                let left = &mut columns[0];
                left.group(|ui| {
                    ui.heading("Enviar archivos");
                    ui.label("Selecciona uno o varios archivos para generar un código.");
                    ui.add_space(10.0);
                    if ui.button("＋  Elegir archivos").clicked() {
                        if let Some(picked) = rfd::FileDialog::new().pick_files() { self.files = picked; }
                    }
                    for file in &self.files { ui.label(format!("• {}", file.file_name().unwrap_or_default().to_string_lossy())); }
                    ui.add_space(8.0);
                    if ui.add_enabled(!self.files.is_empty() && !self.running, egui::Button::new("Enviar ahora  →")).clicked() { self.start_send(); }
                });
                let right = &mut columns[1];
                right.group(|ui| {
                    ui.heading("Recibir archivos");
                    ui.label("Pega el código que te compartieron.");
                    ui.add_space(10.0);
                    ui.add(egui::TextEdit::singleline(&mut self.code).hint_text("ej. 1234-5678-9012"));
                    if ui.button("⌂  Elegir carpeta").clicked() { self.destination = rfd::FileDialog::new().pick_folder(); }
                    if let Some(dir) = &self.destination { ui.label(dir.to_string_lossy().to_string()); }
                    ui.add_space(8.0);
                    if ui.add_enabled(!self.code.trim().is_empty() && !self.running, egui::Button::new("Recibir ahora  →")).clicked() { self.start_receive(); }
                });
            });
            ui.add_space(16.0);
            ui.horizontal(|ui| { ui.label(if self.running { "●" } else { "○" }); ui.label(&self.status); });
            if !self.log.is_empty() { ui.add_space(8.0); ui.add(egui::TextEdit::multiline(&mut self.log).desired_rows(5).interactive(false)); }
            ui.add_space(12.0);
            egui::CollapsingHeader::new("Configuración avanzada").show(ui, |ui| {
                ui.horizontal(|ui| { ui.label("Ejecutable croc:"); ui.text_edit_singleline(&mut self.croc_path); });
            });
        });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions { viewport: egui::ViewportBuilder::default().with_inner_size(Vec2::new(760.0, 620.0)).with_min_inner_size(Vec2::new(600.0, 500.0)), ..Default::default() };
    eframe::run_native("Croc UI", options, Box::new(|cc| Ok(Box::new(CrocApp::new(cc)))))
}
