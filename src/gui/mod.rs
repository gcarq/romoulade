mod debugger;
pub mod emulator;

use crate::gb::cartridge::Cartridge;
use crate::gui::emulator::EmulatorFrontend;
use eframe::egui;
use egui::{CentralPanel, Color32, TopBottomPanel, Ui};

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
        self.cartridge =
            Some(Cartridge::from_path(&path).expect("Unable to load cartridge from path"));
    }

    /// Starts the emulator with the loaded cartridge.
    #[inline]
    fn run(&mut self, debug: bool) {
        if let Some(cartridge) = &self.cartridge {
            self.frontend = Some(EmulatorFrontend::start(cartridge, debug));
        }
    }

    /// Draws the top panel of the main window.
    fn draw_top_panel(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if ui.button("Load ROM").clicked() {
                self.load_rom();
            }
            ui.separator();
            if ui.button("Run").clicked() {
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
                if let Some(frontend) = &self.frontend {
                    frontend.shutdown();
                    self.frontend = None;
                } else {
                    println!("Emulator is alredy stopped");
                }
            }
            ui.separator();
            if let Some(cartridge) = &self.cartridge {
                ui.colored_label(Color32::ORANGE, format!("{}", cartridge));
            } else {
                ui.label("No ROM loaded");
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
