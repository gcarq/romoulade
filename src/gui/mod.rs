mod debugger;
pub mod emulator;
#[macro_use]
mod macros;

use crate::gb::cartridge::Cartridge;
use crate::gb::{EmulatorConfig, GBResult};
use crate::gui::emulator::EmulatorFrontend;
use eframe::egui;
use eframe::egui::menu;
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
        Ok(Self {
            frontend: None,
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

    /// Loads a save file using a file dialog.
    fn choose_savefile(&mut self) -> GBResult<()> {
        let dialog = rfd::FileDialog::new().add_filter("Save Files", &["sav"]);
        if let Some(path) = dialog.pick_file() {
            println!("Loading save file: {}", path.display());
            if let Some(cartridge) = &mut self.cartridge {
                let mut data = fs::read(&path)?;
                let checksum = match data.pop() {
                    Some(checksum) => checksum,
                    None => return Err("Save file is empty or invalid.".into()),
                };
                if checksum != cartridge.header.header_checksum {
                    return Err(format!(
                        "Unable to load Savegame - Checksum mismatch: expected {}, got {}",
                        cartridge.header.header_checksum, checksum
                    )
                    .into());
                }
                cartridge.controller.load_ram(data);
                println!("Save file loaded successfully.");
            } else {
                return Err("No cartridge loaded to apply the save file.".into());
            }
        }
        Ok(())
    }

    /// Starts the `Emulator` with the loaded `Cartridge`.
    fn run(&mut self, ui: &Ui) {
        if let Some(cartridge) = &self.cartridge {
            self.frontend = Some(EmulatorFrontend::start(
                ui.ctx(),
                cartridge,
                self.config.clone(),
            ));
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
        menu::bar(ui, |ui| {
            // Main emulator menu
            ui.menu_button(menu_text!("Emulator"), |ui| {
                // Load ROM button
                if ui.button(menu_text!("Load ROM...")).clicked() {
                    self.shutdown();
                    if let Err(msg) = self.choose_cartridge() {
                        eprintln!("Error loading ROM: {msg}");
                    }
                }
                // Run or Reset button
                ui.add_enabled_ui(self.cartridge.is_some(), |ui| {
                    let text = match self.frontend {
                        Some(_) => "Reset",
                        None => "Run",
                    };
                    if ui.button(menu_text!(text)).clicked() {
                        self.shutdown();
                        self.run(ui);
                    }
                });
                // Stop button
                ui.add_enabled_ui(self.frontend.is_some(), |ui| {
                    if ui.button(menu_text!("Stop")).clicked() {
                        self.shutdown();
                    }
                });
                // Exit button
                if ui.button(menu_text!("Exit")).clicked() {
                    self.shutdown();
                    ui.ctx().request_repaint();
                    std::process::exit(0);
                }
            });
            ui.separator();
            // Savegames menu
            ui.menu_button(menu_text!("Savegames"), |ui| {
                let supports_saves = self
                    .cartridge
                    .as_ref()
                    .is_some_and(|c| c.header.has_persistent_data());
                // Load button
                ui.add_enabled_ui(self.cartridge.is_some() && supports_saves, |ui| {
                    if ui.button(menu_text!("Load...")).clicked() {
                        match self.choose_savefile() {
                            Ok(_) => {
                                self.shutdown();
                                self.run(ui);
                            }
                            Err(msg) => eprintln!("Error loading save file: {msg}"),
                        }
                    }
                });
                // Save button
                ui.add_enabled_ui(self.frontend.is_some() && supports_saves, |ui| {
                    if ui.button(menu_text!("Save...")).clicked() {
                        if let Some(frontend) = &mut self.frontend {
                            let dialog = rfd::FileDialog::new().add_filter("Save Files", &["sav"]);
                            if let Some(path) = dialog.save_file() {
                                frontend.write_savefile(path);
                            }
                        }
                    }
                });
            });
            ui.separator();
            // Settings menu
            ui.menu_button(menu_text!("Settings"), |ui| {
                ui.checkbox(&mut self.config.autosave, menu_text!("Enable Autosave"))
                    .on_hover_text("Automatically saves the game every 30 seconds and on exit.");
            });
            ui.separator();
            // Debug menu
            ui.menu_button(menu_text!("Debug"), |ui| {
                // Attach Debugger button
                ui.add_enabled_ui(self.cartridge.is_some(), |ui| {
                    if ui.button(menu_text!("Attach Debugger")).clicked() {
                        if self.frontend.is_none() {
                            self.run(ui);
                        }
                        if let Some(frontend) = &mut self.frontend {
                            frontend.attach_debugger();
                        }
                    }
                });
            });
            ui.separator();
            self.draw_cartridge_info(ui);
        });
    }

    /// Displays the cartridge information in the top panel.
    fn draw_cartridge_info(&self, ui: &mut Ui) {
        let text = if let Some(cartridge) = &self.cartridge {
            menu_text!(cartridge.to_string()).color(Color32::ORANGE)
        } else {
            menu_text!("No ROM loaded")
        };
        Label::new(text).selectable(false).ui(ui);
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
