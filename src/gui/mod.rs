mod debugger;
pub mod emulator;

use crate::gb::GBResult;
use crate::gb::cartridge::Cartridge;
use crate::gui::emulator::EmulatorFrontend;
use eframe::egui;
use egui::{CentralPanel, Color32, Label, RichText, TopBottomPanel, Ui, Widget};
use std::fs;
use std::path::Path;
use std::sync::Arc;

#[derive(Default)]
pub struct Romoulade {
    frontend: Option<EmulatorFrontend>,
    cartridge: Option<Cartridge>,
}

impl Romoulade {
    /// Loads a cartridge from the given `Path`.
    pub fn load_cartridge(&mut self, path: &Path) -> GBResult<()> {
        println!("Loading Cartridge: {}", path.display());
        let rom = fs::read(path)?;
        self.cartridge = Some(Cartridge::try_from(Arc::from(rom.into_boxed_slice()))?);
        Ok(())
    }

    /// Loads a cartridge using a file dialog.
    fn choose_cartridge(&mut self) {
        if let Some(frontend) = &self.frontend {
            frontend.shutdown();
        }

        let dialog = rfd::FileDialog::new().add_filter("Game Boy ROM", &["gb"]);
        if let Some(path) = dialog.pick_file() {
            self.load_cartridge(&path)
                .expect("Failed to load cartridge");
        }
    }

    /// Starts the emulator with the loaded cartridge.
    #[inline]
    fn run(&mut self, debug: bool) {
        if let Some(cartridge) = &self.cartridge {
            self.frontend = Some(EmulatorFrontend::start(cartridge, debug));
        }
    }

    /// Shuts down the emulator and cleans up resources.
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
                self.choose_cartridge();
            }
            ui.separator();
            if ui.button("Run").clicked() {
                self.shutdown();
                self.run(false);
            }
            if ui.button("Attach Debugger").clicked() {
                if let Some(frontend) = &mut self.frontend {
                    frontend.attach_debugger();
                } else {
                    self.run(true);
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
