use crate::domain::models::{AppSettings, HistoryEntry, OutputFormat, VoiceInfo};
use crate::services::{history, paths, speech};
use eframe::egui;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

const SAMPLE_TEXT: &str = "Olá! Esta é uma demonstração do Rust Voice Studio, um aplicativo de síntese de voz que utiliza as vozes instaladas no Windows.";

#[derive(Debug)]
enum UiMessage {
    VoicesLoaded(Result<Vec<VoiceInfo>, String>),
    PreviewFinished(Result<(), String>),
    SpeakFinished(Result<(), String>),
    ExportFinished(Result<HistoryEntry, String>),
}

pub struct App {
    text: String,
    voices: Vec<VoiceInfo>,
    selected_voice: usize,
    settings: AppSettings,
    history: Vec<HistoryEntry>,
    status: String,
    error: Option<String>,
    busy: bool,
    loading_voices: bool,
    tx: Sender<UiMessage>,
    rx: Receiver<UiMessage>,
}

impl App {
    pub fn new(_creation_context: &eframe::CreationContext<'_>) -> Self {
        let (tx, rx) = mpsc::channel();
        let settings = history::load_settings();

        let app = Self {
            text: String::new(),
            voices: Vec::new(),
            selected_voice: 0,
            settings,
            history: history::load_history(),
            status: "Inicializando mecanismo de voz do Windows...".to_string(),
            error: None,
            busy: false,
            loading_voices: true,
            tx,
            rx,
        };

        app.load_voices();
        app
    }

    fn load_voices(&self) {
        let sender = self.tx.clone();
        thread::spawn(move || {
            let result = speech::list_voices();
            let _ = sender.send(UiMessage::VoicesLoaded(result));
        });
    }

    fn handle_messages(&mut self) {
        while let Ok(message) = self.rx.try_recv() {
            match message {
                UiMessage::VoicesLoaded(result) => {
                    self.loading_voices = false;
                    match result {
                        Ok(voices) => {
                            let preferred = self.settings.selected_voice.clone();
                            self.voices = voices;
                            self.selected_voice = preferred
                                .and_then(|name| self.voices.iter().position(|voice| voice.name == name))
                                .unwrap_or(0);
                            if self.voices.is_empty() {
                                self.status = "Nenhuma voz SAPI foi encontrada neste computador.".to_string();
                            } else {
                                self.status = format!("{} voz(es) SAPI encontrada(s).", self.voices.len());
                                self.update_selected_voice_setting();
                            }
                        }
                        Err(error) => {
                            self.error = Some(error.clone());
                            self.status = "Não foi possível carregar as vozes.".to_string();
                        }
                    }
                }
                UiMessage::PreviewFinished(result) => {
                    self.busy = false;
                    match result {
                        Ok(()) => {
                            self.error = None;
                            self.status = "Prévia concluída.".to_string();
                        }
                        Err(error) => {
                            self.error = Some(error.clone());
                            self.status = "Falha ao reproduzir a prévia.".to_string();
                        }
                    }
                }
                UiMessage::SpeakFinished(result) => {
                    self.busy = false;
                    match result {
                        Ok(()) => {
                            self.error = None;
                            self.status = "Leitura do texto concluída.".to_string();
                        }
                        Err(error) => {
                            self.error = Some(error);
                            self.status = "Não foi possível reproduzir o texto.".to_string();
                        }
                    }
                }
                UiMessage::ExportFinished(result) => {
                    self.busy = false;
                    match result {
                        Ok(entry) => {
                            self.error = None;
                            self.status = format!("Arquivo gerado: {}", entry.title);
                            self.history.insert(0, entry);
                            self.history.truncate(50);
                            if let Err(error) = history::save_history(&self.history) {
                                self.error = Some(error);
                            }
                        }
                        Err(error) => {
                            self.error = Some(error.clone());
                            self.status = "Não foi possível gerar o áudio.".to_string();
                        }
                    }
                }
            }
        }
    }

    fn selected_voice_info(&self) -> Option<&VoiceInfo> {
        self.voices.get(self.selected_voice)
    }

    fn update_selected_voice_setting(&mut self) {
        self.settings.selected_voice = self.selected_voice_info().map(|voice| voice.name.clone());
        if let Err(error) = history::save_settings(&self.settings) {
            self.error = Some(error);
        }
    }

    fn save_current_settings(&mut self) {
        if let Some(voice) = self.selected_voice_info() {
            self.settings.selected_voice = Some(voice.name.clone());
        }
        if let Err(error) = history::save_settings(&self.settings) {
            self.error = Some(error);
        }
    }

    fn start_preview(&mut self) {
        if self.voices.is_empty() {
            self.error = Some("Nenhuma voz SAPI está disponível para prévia.".to_string());
            return;
        }

        self.busy = true;
        self.error = None;
        self.status = "Reproduzindo prévia no dispositivo de áudio padrão...".to_string();
        self.save_current_settings();

        let sender = self.tx.clone();
        let voice_index = self
            .selected_voice_info()
            .map(|voice| voice.index)
            .unwrap_or(self.selected_voice);
        let rate = self.settings.rate;
        let volume = self.settings.volume;

        thread::spawn(move || {
            let result = speech::test_voice(voice_index, rate, volume);
            let _ = sender.send(UiMessage::PreviewFinished(result));
        });
    }

    fn start_speak_text(&mut self) {
        if self.text.trim().is_empty() {
            self.error = Some("Digite ou carregue um texto antes de iniciar a leitura.".to_string());
            return;
        }
        if self.voices.is_empty() {
            self.error = Some("Não há vozes SAPI disponíveis. Verifique as vozes instaladas no Windows.".to_string());
            return;
        }

        self.busy = true;
        self.error = None;
        self.status = "Lendo o texto no dispositivo de áudio padrão...".to_string();
        self.save_current_settings();

        // A thread de trabalho mantém a interface responsiva durante a leitura.
        let sender = self.tx.clone();
        let text = self.text.clone();
        let voice_index = self
            .selected_voice_info()
            .map(|voice| voice.index)
            .unwrap_or(self.selected_voice);
        let rate = self.settings.rate;
        let volume = self.settings.volume;

        thread::spawn(move || {
            let result = speech::speak_text(&text, voice_index, rate, volume);
            let _ = sender.send(UiMessage::SpeakFinished(result));
        });
    }

    fn choose_export_path(&self, format: OutputFormat) -> Option<PathBuf> {
        let export_dir = paths::default_export_dir();
        let _ = std::fs::create_dir_all(&export_dir);
        let suggested_name = format!("voz-gerada.{}", format.extension());
        let dialog = rfd::FileDialog::new()
            .set_directory(export_dir)
            .set_file_name(&suggested_name)
            .add_filter(format.label(), &[format.extension()]);
        dialog.save_file()
    }

    fn start_export(&mut self) {
        if self.text.trim().is_empty() {
            self.error = Some("Digite ou carregue um texto antes de gerar o áudio.".to_string());
            return;
        }
        if self.voices.is_empty() {
            self.error = Some("Não há vozes SAPI disponíveis. Verifique as vozes instaladas no Windows.".to_string());
            return;
        }

        let format = OutputFormat::from(self.settings.output_format.as_str());
        let Some(destination) = self.choose_export_path(format) else {
            self.status = "Exportação cancelada.".to_string();
            return;
        };

        self.busy = true;
        self.error = None;
        self.status = format!("Gerando áudio {}...", format.label());
        self.save_current_settings();

        let sender = self.tx.clone();
        let text = self.text.clone();
        let voice_index = self
            .selected_voice_info()
            .map(|voice| voice.index)
            .unwrap_or(self.selected_voice);
        let rate = self.settings.rate;
        let volume = self.settings.volume;
        let bitrate = self.settings.bitrate_kbps;

        thread::spawn(move || {
            let result = speech::export_audio(
                text,
                voice_index,
                rate,
                volume,
                destination,
                format,
                bitrate,
            );
            let _ = sender.send(UiMessage::ExportFinished(result));
        });
    }

    fn load_text_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Arquivos de texto", &["txt", "md"])
            .pick_file()
        {
            match std::fs::read_to_string(&path) {
                Ok(contents) => {
                    self.text = contents;
                    self.status = format!("Texto carregado: {}", path.display());
                    self.error = None;
                }
                Err(error) => self.error = Some(format!("Não foi possível ler o arquivo: {error}")),
            }
        }
    }

    fn save_text_file(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .set_file_name("texto-para-narracao.txt")
            .add_filter("Arquivo de texto", &["txt"])
            .save_file()
        {
            match std::fs::write(&path, self.text.as_bytes()) {
                Ok(()) => {
                    self.status = format!("Texto salvo: {}", path.display());
                    self.error = None;
                }
                Err(error) => self.error = Some(format!("Não foi possível salvar o texto: {error}")),
            }
        }
    }

    fn open_export_folder(&mut self) {
        let directory = paths::default_export_dir();
        if let Err(error) = std::fs::create_dir_all(&directory) {
            self.error = Some(format!("Não foi possível criar a pasta: {error}"));
            return;
        }
        match std::process::Command::new("explorer.exe").arg(&directory).spawn() {
            Ok(_) => {
                self.status = format!("Pasta aberta: {}", directory.display());
                self.error = None;
            }
            Err(error) => self.error = Some(format!("Não foi possível abrir a pasta: {error}")),
        }
    }

    fn open_history_file(path: &PathBuf) -> Result<(), String> {
        if !path.exists() {
            return Err("O arquivo não existe mais no caminho registrado.".to_string());
        }
        std::process::Command::new("cmd")
            .args(["/C", "start", ""])
            .arg(path)
            .spawn()
            .map(|_| ())
            .map_err(|error| format!("Não foi possível abrir o arquivo: {error}"))
    }

    fn render_header(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.heading(
                    egui::RichText::new("RUST VOICE STUDIO")
                        .size(25.0)
                        .strong()
                        .color(egui::Color32::from_rgb(103, 232, 249)),
                );
                ui.label(
                    egui::RichText::new("TEXT-TO-SPEECH DESKTOP · SAPI NATIVA · OFFLINE")
                        .size(10.5)
                        .color(egui::Color32::from_rgb(160, 174, 192)),
                );
            });
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    egui::RichText::new("● OFFLINE")
                        .strong()
                        .color(egui::Color32::from_rgb(74, 222, 128)),
                );
                if ui.button("Atualizar vozes").clicked() && !self.loading_voices && !self.busy {
                    self.loading_voices = true;
                    self.status = "Atualizando a lista de vozes...".to_string();
                    self.load_voices();
                }
            });
        });
        ui.add_space(6.0);
        ui.separator();
    }

    fn render_editor(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("Texto para conversão");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    egui::RichText::new(format!("{} caracteres", self.text.chars().count()))
                        .small()
                        .color(egui::Color32::from_rgb(148, 163, 184)),
                );
            });
        });

        ui.add(
            egui::TextEdit::multiline(&mut self.text)
                .desired_rows(14)
                .desired_width(f32::INFINITY)
                .hint_text("Digite ou cole aqui o texto que deseja transformar em fala..."),
        );

        ui.horizontal(|ui| {
            if ui.button("Carregar TXT").clicked() && !self.busy {
                self.load_text_file();
            }
            if ui.button("Salvar TXT").clicked() && !self.busy {
                self.save_text_file();
            }
            if ui.button("Inserir exemplo").clicked() && !self.busy {
                self.text = SAMPLE_TEXT.to_string();
            }
            if ui.button("Limpar texto").clicked() && !self.busy {
                self.text.clear();
            }
        });
    }

    fn render_settings(&mut self, ui: &mut egui::Ui) {
        ui.heading("Configurações de voz");
        ui.add_space(4.0);

        let mut changed = false;
        ui.label("Voz instalada no Windows");
        let selected_label = self
            .selected_voice_info()
            .map(|voice| format!("{}  ·  {}", voice.name, voice.language))
            .unwrap_or_else(|| {
                if self.loading_voices {
                    "Carregando vozes...".to_string()
                } else {
                    "Nenhuma voz disponível".to_string()
                }
            });

        egui::ComboBox::from_id_salt("voice_selection")
            .width(ui.available_width())
            .selected_text(selected_label)
            .show_ui(ui, |ui| {
                for (position, voice) in self.voices.iter().enumerate() {
                    let label = format!("{}  ·  {}", voice.name, voice.language);
                    if ui
                        .selectable_value(&mut self.selected_voice, position, label)
                        .changed()
                    {
                        changed = true;
                    }
                }
            });

        ui.add_space(8.0);
        ui.horizontal(|ui| {
            ui.label("Velocidade");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(format!("{:+}", self.settings.rate));
            });
        });
        changed |= ui
            .add(egui::Slider::new(&mut self.settings.rate, -10..=10).show_value(false))
            .changed();
        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Mais lenta").small().weak());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(egui::RichText::new("Mais rápida").small().weak());
            });
        });

        ui.add_space(7.0);
        ui.horizontal(|ui| {
            ui.label("Volume da fala");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(format!("{}%", self.settings.volume));
            });
        });
        changed |= ui
            .add(egui::Slider::new(&mut self.settings.volume, 0..=100).show_value(false))
            .changed();

        ui.add_space(10.0);
        ui.separator();
        ui.heading("Formato de saída");
        ui.horizontal(|ui| {
            ui.label("Formato");
            egui::ComboBox::from_id_salt("output_format")
                .selected_text(self.settings.output_format.clone())
                .show_ui(ui, |ui| {
                    changed |= ui
                        .selectable_value(&mut self.settings.output_format, "MP3".to_string(), "MP3")
                        .changed();
                    changed |= ui
                        .selectable_value(&mut self.settings.output_format, "WAV".to_string(), "WAV")
                        .changed();
                });
        });

        if OutputFormat::from(self.settings.output_format.as_str()) == OutputFormat::Mp3 {
            ui.horizontal(|ui| {
                ui.label("Taxa MP3");
                egui::ComboBox::from_id_salt("mp3_bitrate")
                    .selected_text(format!("{} kbps", self.settings.bitrate_kbps))
                    .show_ui(ui, |ui| {
                        for bitrate in [96_u32, 128, 160, 192, 256, 320] {
                            changed |= ui
                                .selectable_value(
                                    &mut self.settings.bitrate_kbps,
                                    bitrate,
                                    format!("{bitrate} kbps"),
                                )
                                .changed();
                        }
                    });
            });
        }

        if changed {
            self.save_current_settings();
        }

        ui.add_space(10.0);
        let no_voices = self.voices.is_empty() || self.loading_voices;
        ui.horizontal_wrapped(|ui| {
            if ui
                .add_enabled(
                    !self.busy && !no_voices && !self.text.trim().is_empty(),
                    egui::Button::new("▶  Ouvir texto"),
                )
                .on_hover_text("Lê todo o texto do editor usando a voz selecionada.")
                .clicked()
            {
                self.start_speak_text();
            }
            if ui
                .add_enabled(!self.busy && !no_voices, egui::Button::new("♪  Testar voz"))
                .clicked()
            {
                self.start_preview();
            }
            if ui
                .add_enabled(
                    !self.busy && !no_voices && !self.text.trim().is_empty(),
                    egui::Button::new(
                        egui::RichText::new("GERAR ÁUDIO")
                            .strong()
                            .color(egui::Color32::from_rgb(8, 15, 25)),
                    )
                    .fill(egui::Color32::from_rgb(103, 232, 249)),
                )
                .clicked()
            {
                self.start_export();
            }
        });
        ui.add_space(4.0);
        ui.label(
            egui::RichText::new("Ouvir texto reproduz o conteúdo completo sem criar arquivo. A saída usa o dispositivo de áudio padrão do Windows.")
                .small()
                .weak(),
        );
    }

    fn render_history(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("Histórico recente");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("Abrir pasta de exportação").clicked() {
                    self.open_export_folder();
                }
            });
        });

        if self.history.is_empty() {
            ui.label(egui::RichText::new("Os arquivos gerados aparecerão aqui.").weak());
            return;
        }

        let recent_entries: Vec<HistoryEntry> = self.history.iter().take(8).cloned().collect();
        egui::ScrollArea::vertical()
            .max_height(150.0)
            .auto_shrink([false, true])
            .show(ui, |ui| {
                for entry in &recent_entries {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label(egui::RichText::new(&entry.title).strong());
                            ui.label(
                                egui::RichText::new(format!(
                                    "{}  ·  {} caracteres  ·  {}",
                                    entry.format, entry.characters, entry.created_at
                                ))
                                .small()
                                .weak(),
                            );
                        });
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.small_button("Ouvir arquivo").clicked() {
                                if let Err(error) = Self::open_history_file(&entry.path) {
                                    self.error = Some(error);
                                }
                            }
                        });
                    });
                    ui.separator();
                }
            });
    }
}

impl eframe::App for App {
    fn update(&mut self, context: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_messages();

        egui::CentralPanel::default().show(context, |ui| {
            self.render_header(ui);

            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.columns(2, |columns| {
                        columns[0].vertical(|ui| self.render_editor(ui));
                        columns[1].vertical(|ui| self.render_settings(ui));
                    });

                    ui.add_space(12.0);
                    ui.separator();
                    self.render_history(ui);
                });

            ui.add_space(5.0);
            ui.separator();
            ui.horizontal(|ui| {
                if self.busy {
                    ui.spinner();
                }
                ui.label(&self.status);
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new("Rust · Windows SAPI").small().weak());
                });
            });
            if let Some(error) = &self.error {
                ui.colored_label(egui::Color32::from_rgb(248, 113, 113), error);
            }
        });

        if self.busy || self.loading_voices {
            context.request_repaint_after(Duration::from_millis(120));
        }
    }
}
