mod debugger;
pub mod emulator;
#[macro_use]
mod macros;

use crate::gb::cartridge::Cartridge;
use crate::gb::{EmulatorConfig, FrontendMessage, GBResult};
use crate::gui::emulator::{EmulatorFrontend, SCREEN_HEIGHT, SCREEN_WIDTH};
use eframe::egui;
use eframe::egui::{RichText, Vec2, menu};
use egui::{CentralPanel, Color32, Label, TopBottomPanel, Ui, Widget};
use std::fs;
use std::path::PathBuf;
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
            frontend.stop();
        }
        let dialog = rfd::FileDialog::new().add_filter("Game Boy ROM", &["gb"]);
        if let Some(path) = dialog.pick_file() {
            println!("Loading Cartridge: {}", path.display());
            let rom = fs::read(path)?;
            let cartridge = Cartridge::try_from(Arc::from(rom.into_boxed_slice()))?;

            // Set savefile if autosave is enabled
            if self.config.autosave && cartridge.header.config.is_savable() {
                self.config.savefile = Some(PathBuf::from(cartridge.autosave_filename()));
            }
            self.cartridge = Some(cartridge);
        }
        Ok(())
    }

    /// Loads a save file using a file dialog.
    fn choose_savefile(&mut self) {
        let dialog = rfd::FileDialog::new().add_filter("Save Files", &["sav"]);
        if let Some(path) = dialog.pick_file() {
            self.config.savefile = Some(path);
        }
    }

    /// Starts the `Emulator` with the loaded `Cartridge`.
    fn start_emulator(&mut self, ui: &Ui) {
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
    fn stop_emulator(&mut self) {
        if let Some(frontend) = self.frontend.take() {
            frontend.stop();
        }
    }

    /// Updates the top panel of the main window.
    fn update_top_panel(&mut self, ui: &mut Ui) {
        menu::bar(ui, |ui| {
            self.update_emulator_menu(ui);
            ui.separator();
            self.update_savegame_menu(ui);
            ui.separator();
            self.update_settings_menu(ui);
            ui.separator();
            self.update_tools_menu(ui);
        });
    }

    /// Updates the emulator section in the menu bar.
    fn update_emulator_menu(&mut self, ui: &mut Ui) {
        ui.menu_button(menu_text!("Emulator"), |ui| {
            // Load ROM button
            if ui.button(menu_text!("📁 Load ROM...")).clicked() {
                self.stop_emulator();
                if let Err(msg) = self.choose_cartridge() {
                    eprintln!("Error loading ROM: {msg}");
                }
            }
            ui.separator();
            // Run or Reset button
            ui.add_enabled_ui(self.cartridge.is_some(), |ui| {
                let text = match self.frontend {
                    Some(_) => "◀ Reset",
                    None => "▶ Run",
                };
                if ui.button(menu_text!(text)).clicked() {
                    self.stop_emulator();
                    self.start_emulator(ui);
                }
            });
            // Stop button
            ui.add_enabled_ui(self.frontend.is_some(), |ui| {
                if ui.button(menu_text!("⏹ Stop")).clicked() {
                    self.stop_emulator();
                }
            });
        });
    }

    /// Updates the savegame section in the menu bar.
    fn update_savegame_menu(&mut self, ui: &mut Ui) {
        ui.menu_button(menu_text!("Savegames"), |ui| {
            let supports_saves = self
                .cartridge
                .as_ref()
                .is_some_and(|c| c.header.config.is_savable());
            // Load button
            ui.add_enabled_ui(
                supports_saves && self.cartridge.is_some() && self.frontend.is_none(),
                |ui| {
                    if ui.button(menu_text!("📁 Load...")).clicked() {
                        self.choose_savefile();
                    }
                },
            );
            ui.separator();
            // Save button
            ui.add_enabled_ui(
                supports_saves && self.config.savefile.is_some() && self.frontend.is_some(),
                |ui| {
                    if ui.button(menu_text!("💾 Save")).clicked() {
                        if let Some(frontend) = &mut self.frontend {
                            frontend.send_message(FrontendMessage::WriteSaveFile);
                        }
                    }
                },
            );
            // Save As button
            ui.add_enabled_ui(self.frontend.is_some() && supports_saves, |ui| {
                if ui.button(menu_text!("💾 Save As...")).clicked() {
                    if let Some(frontend) = &mut self.frontend {
                        let dialog = rfd::FileDialog::new().add_filter("Save Files", &["sav"]);
                        if let Some(path) = dialog.save_file() {
                            println!("Writing save file to: {}", path.display());
                            self.config.savefile = Some(path);
                            frontend
                                .send_message(FrontendMessage::UpdateConfig(self.config.clone()));
                            frontend.send_message(FrontendMessage::WriteSaveFile);
                        }
                    }
                }
            });
        });
    }

    /// Updates the settings menu in the menu bar.
    fn update_settings_menu(&mut self, ui: &mut Ui) {
        ui.menu_button(menu_text!("Settings"), |ui| {
            if ui
                .checkbox(&mut self.config.autosave, menu_text!("Enable Autosave"))
                .on_hover_text(
                    "Automatically saves the game every 60 seconds and on emulator shutdown.\n\
                    Also loads the last save file on startup.",
                )
                .clicked()
            {
                if let Some(frontend) = &mut self.frontend {
                    frontend.send_message(FrontendMessage::UpdateConfig(self.config.clone()));
                }
            }
        });
    }

    /// Updates the tools menu in the menu bar.
    fn update_tools_menu(&mut self, ui: &mut Ui) {
        ui.menu_button(menu_text!("Tools"), |ui| {
            // Attach Debugger button
            ui.add_enabled_ui(self.cartridge.is_some(), |ui| {
                if ui.button(menu_text!("🔧 Attach Debugger")).clicked() {
                    if self.frontend.is_none() {
                        self.start_emulator(ui);
                    }
                    if let Some(frontend) = &mut self.frontend {
                        frontend.attach_debugger();
                    }
                }
            });
        });
    }

    /// Displays the cartridge information in the bottom panel.
    fn draw_cartridge_info(&self, ui: &mut Ui) {
        let label = if let Some(cartridge) = &self.cartridge {
            Label::new(RichText::new(cartridge.to_string()).color(Color32::ORANGE))
        } else {
            Label::new("No ROM loaded")
        };
        label.selectable(false).ui(ui);
    }

    /// Displays the savefile information in the bottom panel.
    fn draw_savefile_info(&self, ui: &mut Ui) {
        if let Some(ref savefile) = self.config.savefile {
            // TODO: get rid of unwrap
            Label::new(savefile.file_name().unwrap().display().to_string())
                .selectable(false)
                .ui(ui);
        }
    }

    /// Returns the dimensions of the frame layout in pixels.
    const fn frame_layout_dimensions(&self) -> Vec2 {
        Vec2 {
            x: (SCREEN_WIDTH * self.config.upscale) as f32,
            y: (SCREEN_HEIGHT * self.config.upscale) as f32,
        }
    }
}

impl eframe::App for Romoulade {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            self.update_top_panel(ui);
        });
        TopBottomPanel::bottom("status_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                self.draw_cartridge_info(ui);
                ui.separator();
                self.draw_savefile_info(ui);
            });
        });
        CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                ui.allocate_ui(self.frame_layout_dimensions(), |ui| {
                    if let Some(emulator) = &mut self.frontend {
                        emulator.update(ctx, ui);
                    }
                });
            });
        if ctx.input(|i| i.viewport().close_requested()) {
            self.stop_emulator();
        }
    }
}
