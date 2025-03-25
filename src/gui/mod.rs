mod state;

use crate::gb::cartridge::Cartridge;
use crate::gb::ppu::buffer::FrameBuffer;
use crate::gb::{Emulator, EmulatorMessage};
use crate::gui::state::EmulatorState;
use eframe::egui;
use eframe::egui::{ColorImage, TextureOptions, Ui, Vec2};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;

pub const SCREEN_WIDTH: usize = 160;
pub const SCREEN_HEIGHT: usize = 144;

// TODO: make this configurable
pub const UPSCALE: usize = 3;

pub enum FrontendMessage {
    Reset,
}

/// A channel to communicate between the emulator and the frontend.
/// The frontend can send messages with `sender`
/// and receive messages from the emulator with `receiver`.
struct EmulatorChannel {
    pub sender: Sender<FrontendMessage>,
    pub receiver: Receiver<EmulatorMessage>,
}

impl EmulatorChannel {
    pub fn new(sender: Sender<FrontendMessage>, receiver: Receiver<EmulatorMessage>) -> Self {
        Self { sender, receiver }
    }
}

#[derive(Default)]
pub struct Romoulade {
    state: Option<EmulatorState>,
}

impl Romoulade {
    fn load_rom(&mut self) {
        let path = match rfd::FileDialog::new().pick_file() {
            Some(path) => path,
            None => return,
        };
        println!("Loading ROM: {}", path.display());
        let cartridge = Cartridge::from_path(&path).expect("Unable to load cartridge from path");
        self.start_emulator(cartridge);
    }

    /// Starts the emulator with the given cartridge.
    fn start_emulator(&mut self, cartridge: Cartridge) {
        let (emulator_sender, emulator_receiver) = mpsc::channel();
        let (frontend_sender, frontend_receiver) = mpsc::channel();
        let used_cartridge = cartridge.clone();
        let thread = thread::spawn(move || {
            let mut emulator =
                Emulator::new(emulator_sender, frontend_receiver, used_cartridge, UPSCALE)
                    .expect("Unable to create GameBoy instance");
            emulator.run();
        });
        self.state = Some(EmulatorState::new(
            thread,
            EmulatorChannel::new(frontend_sender, emulator_receiver),
            cartridge,
        ));
    }

    /// Sends a reset message to the emulator.
    fn reset(&mut self) {
        let state = match &self.state {
            Some(state) => state,
            None => return,
        };

        match state.channel.sender.send(FrontendMessage::Reset) {
            Ok(_) => println!("Resetting emulator ..."),
            Err(_) => eprintln!("Emulator is not running"),
        }

        self.start_emulator(state.cartridge.clone());
    }

    /// Checks the channel for new messages and updates the emulation screen
    /// if a new frame is available.
    fn update_emulation_screen(&self, ctx: &egui::Context, ui: &mut Ui) {
        let receiver = match &self.state {
            Some(state) => &state.channel.receiver,
            None => return,
        };

        if let Ok(message) = receiver.recv() {
            match message {
                EmulatorMessage::Frame(frame) => self.update_emulation_texture(frame, ctx, ui),
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
                    self.reset();
                }
            });
        });
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| match self.state {
                Some(_) => self.update_emulation_screen(ctx, ui),
                None => {
                    ui.label("Emulator stopped");
                }
            });
        });
    }
}
