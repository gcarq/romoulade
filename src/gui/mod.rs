mod frontend;

use crate::gb::EmulatorMessage;
use crate::gb::cartridge::Cartridge;
use crate::gb::joypad::JoypadInput;
use crate::gb::ppu::buffer::FrameBuffer;
use crate::gui::frontend::EmulatorFrontend;
use eframe::egui;
use eframe::egui::{ColorImage, TextureOptions, Ui, Vec2};
use std::sync::mpsc::{Receiver, Sender};

pub const SCREEN_WIDTH: usize = 160;
pub const SCREEN_HEIGHT: usize = 144;

// TODO: make this configurable
pub const UPSCALE: usize = 3;

pub enum FrontendMessage {
    Stop,
    Input(JoypadInput),
}

/// A channel to communicate between the emulator and the frontend.
/// The frontend can send messages with `sender`
/// and receive messages from the emulator with `receiver`.
struct EmulatorChannel {
    pub sender: Sender<FrontendMessage>,
    pub receiver: Receiver<EmulatorMessage>,
}

impl EmulatorChannel {
    #[inline]
    pub fn new(sender: Sender<FrontendMessage>, receiver: Receiver<EmulatorMessage>) -> Self {
        Self { sender, receiver }
    }
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

        let path = match rfd::FileDialog::new().pick_file() {
            Some(path) => path,
            None => return,
        };
        println!("Loading ROM: {}", path.display());
        let cartridge = Cartridge::from_path(&path).expect("Unable to load cartridge from path");
        self.frontend = Some(EmulatorFrontend::start(cartridge));
    }

    /// Receive the current frame from the frontend and update the screen
    fn update_emulation_screen(&self, ctx: &egui::Context, ui: &mut Ui) {
        if let Some(frontend) = &self.frontend {
            if let Some(frame) = frontend.recv_frame() {
                self.update_emulation_texture(frame, ctx, ui)
            }
        }
    }

    /// Updates the emulation screen with the given frame buffer.
    /// TODO: this is a bit hacky, we should probably use a texture cache
    fn update_emulation_texture(&self, frame: FrameBuffer, ctx: &egui::Context, ui: &mut Ui) {
        let image = ColorImage {
            size: [SCREEN_WIDTH * UPSCALE, SCREEN_HEIGHT * UPSCALE],
            pixels: frame.buffer,
        };
        let texture = ctx.load_texture("frame", image, TextureOptions::LINEAR);
        ui.image((
            texture.id(),
            Vec2::new(
                (SCREEN_WIDTH * UPSCALE) as f32,
                (SCREEN_HEIGHT * UPSCALE) as f32,
            ),
        ));
        ctx.request_repaint();
    }

    /// Handles user input and sends it to the frontend.
    fn handle_user_input(&self, ui: &mut Ui) {
        if let Some(frontend) = &self.frontend {
            ui.input(|i| {
                for key in &i.keys_down {
                    frontend.send_user_input(key);
                }
            });
        }
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
            self.handle_user_input(ui);
            ui.horizontal(|ui| match self.frontend {
                Some(_) => self.update_emulation_screen(ctx, ui),
                None => {
                    ui.label("Emulator stopped");
                }
            });
        });
    }
}
