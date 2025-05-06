mod debugger;
pub mod emulator;

use crate::gb::cartridge::Cartridge;
use crate::gui::emulator::EmulatorFrontend;
use eframe::egui;
use egui::{CentralPanel, Color32, Label, RichText, TopBottomPanel, Ui, Widget};
use std::fs;
use std::sync::Arc;

#[derive(Default)]
pub struct Romoulade {
    frontend: Option<EmulatorFrontend>,
    cartridge: Option<Cartridge>,
}

impl Romoulade {
    /// Loads a ROM file using a file dialog.
    fn load_rom(&mut self) {
        if let Some(frontend) = &self.frontend {
            frontend.shutdown();
        }

        let path = match rfd::FileDialog::new()
            .add_filter("Game Boy ROM", &["gb"])
            .pick_file()
        {
            Some(path) => path,
            None => return,
        };
        println!("Loading ROM: {}", path.display());
        let rom = fs::read(path).expect("Unable to load cartridge from path");
        self.cartridge = Some(
            Cartridge::try_from(Arc::from(rom.into_boxed_slice()))
                .expect("Cartridge is corrupt or unsupported"),
        );
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
                self.load_rom();
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
