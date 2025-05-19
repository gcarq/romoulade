mod debugger;
pub mod emulator;

use crate::gb::cartridge::Cartridge;
use crate::gb::{EmulatorConfig, GBResult};
use crate::gui::emulator::EmulatorFrontend;
use eframe::egui;
use egui::{CentralPanel, Color32, Label, RichText, TopBottomPanel, Ui, Widget};
use std::fs;
use std::sync::Arc;

pub struct Romoulade {
    frontend: Option<EmulatorFrontend>,
    cartridge: Option<Cartridge>,
    config: EmulatorConfig,
}

impl Romoulade {
    pub fn new(config: EmulatorConfig) -> GBResult<Self> {
        let cartridge = match &config.rom {
            Some(rom) => Some(Cartridge::try_from(rom.as_path())?),
            None => None,
        };

        // Initialize the emulator frontend if a cartridge is loaded
        let frontend = cartridge
            .as_ref()
            .map(|cartridge| EmulatorFrontend::start(cartridge, config.clone()));

        Ok(Self {
            frontend,
            cartridge,
            config,
        })
    }

    /// Loads a cartridge using a file dialog.
    fn choose_cartridge(&mut self) -> GBResult<()> {
        if let Some(frontend) = &self.frontend {
            frontend.shutdown();
        }
        let dialog = rfd::FileDialog::new().add_filter("Game Boy ROM", &["gb"]);
        if let Some(path) = dialog.pick_file() {
            println!("Loading Cartridge: {}", path.display());
            let rom = fs::read(path)?;
            self.cartridge = Some(Cartridge::try_from(Arc::from(rom.into_boxed_slice()))?);
        }
        Ok(())
    }

    /// Starts the `Emulator` with the loaded `Cartridge`.
    #[inline]
    fn run(&mut self) {
        if let Some(cartridge) = &self.cartridge {
            self.frontend = Some(EmulatorFrontend::start(cartridge, self.config.clone()));
        }
    }

    /// Shuts down the `Emulator` and cleans up resources.
    #[inline]
    fn shutdown(&mut self) {
        if let Some(frontend) = self.frontend.take() {
            frontend.shutdown();
        }
    }

    /// Draws the top panel of the main window.
    fn draw_top_panel(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if ui.button("Load ROM").clicked() {
                self.shutdown();
                if let Err(msg) = self.choose_cartridge() {
                    eprintln!("Error loading ROM: {msg}");
                }
            }
            ui.separator();
            if ui.button("Run").clicked() {
                self.shutdown();
                self.config.debug = false;
                self.run();
            }
            if ui.button("Attach Debugger").clicked() {
                if let Some(frontend) = &mut self.frontend {
                    frontend.attach_debugger();
                } else {
                    self.config.debug = true;
                    self.run();
                }
            }
            if ui.button("Stop").clicked() {
                self.shutdown();
            }
            ui.separator();
            if let Some(cartridge) = &self.cartridge {
                Label::new(RichText::new(format!("{cartridge}")).color(Color32::ORANGE))
                    .selectable(false)
                    .ui(ui);
            } else {
                Label::new("No ROM loaded").selectable(false).ui(ui);
            }
        });
    }
}

impl eframe::App for Romoulade {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            self.draw_top_panel(ui);
        });
        CentralPanel::default().show(ctx, |ui| {
            if let Some(emulator) = &mut self.frontend {
                emulator.update(ctx, ui);
            }
        });
    }
}
