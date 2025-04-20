pub mod emulator;

use crate::gb::cartridge::Cartridge;
use crate::gui::emulator::EmulatorFrontend;
use eframe::egui;
use eframe::egui::Ui;

pub trait View {
    fn ui(&self, ctx: &egui::Context, ui: &mut Ui);
}

#[derive(Default)]
pub struct Romoulade {
    frontend: Option<EmulatorFrontend>,
}

impl Romoulade {
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
        let cartridge = Cartridge::from_path(&path).expect("Unable to load cartridge from path");
        self.frontend = Some(EmulatorFrontend::start(cartridge));
    }
}

impl eframe::App for Romoulade {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Load ROM").clicked() {
                    self.load_rom();
                }
                ui.separator();
                if ui.button("Reset").clicked() {
                    if let Some(frontend) = &mut self.frontend {
                        frontend.restart();
                    }
                }
            });
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(frontend) = &mut self.frontend {
                frontend.update();
                frontend.ui(ctx, ui);
            } else {
                ui.label("No ROM loaded");
            }
        });
    }
}
