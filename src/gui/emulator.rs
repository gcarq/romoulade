use crate::gb::cartridge::Cartridge;
use crate::gb::joypad::{ActionInput, DPadInput, JoypadInput};
use crate::gb::ppu::buffer::FrameBuffer;
use crate::gb::{Emulator, EmulatorMessage};
use crate::gui::View;
use eframe::epaint::ColorImage;
use eframe::epaint::textures::TextureOptions;
use egui::{Key, Ui, Vec2};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::thread::{JoinHandle, sleep};

pub const SCREEN_WIDTH: usize = 160;
pub const SCREEN_HEIGHT: usize = 144;

// TODO: make this configurable
pub const UPSCALE: usize = 3;

pub const UPSCALED_WIDTH: usize = SCREEN_WIDTH * UPSCALE;
pub const UPSCALED_HEIGHT: usize = SCREEN_HEIGHT * UPSCALE;

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

/// Holds the emulation frontend that is responsible for the interaction with the emulation backend.
pub struct EmulatorFrontend {
    thread: JoinHandle<()>,
    cartridge: Cartridge,
    channel: EmulatorChannel,
    latest_frame: FrameBuffer,
}

impl EmulatorFrontend {
    /// Starts the emulator with the given cartridge.
    pub fn start(cartridge: Cartridge) -> Self {
        let (emulator_sender, emulator_receiver) = mpsc::channel();
        let (frontend_sender, frontend_receiver) = mpsc::channel();
        let used_cartridge = cartridge.clone();
        let thread = thread::spawn(move || {
            // TODO: implement proper debug initialization
            let mut emulator =
                Emulator::new(emulator_sender, frontend_receiver, used_cartridge, UPSCALE)
                    .expect("Unable to create GameBoy instance");
            emulator.run();
        });
        Self {
            thread,
            cartridge,
            channel: EmulatorChannel::new(frontend_sender, emulator_receiver),
            latest_frame: FrameBuffer::new(UPSCALE),
        }
    }

    /// Restarts the emulator with the same cartridge.
    #[inline]
    pub fn restart(&mut self) {
        self.shutdown();
        *self = Self::start(self.cartridge.clone());
    }

    /// Shuts the emulator down by sending a reset message and waiting for it to finish.
    pub fn shutdown(&self) {
        match self.channel.sender.send(FrontendMessage::Stop) {
            Ok(_) => println!("Stopping emulator ..."),
            Err(_) => eprintln!("Emulator is not running"),
        }
        while !self.thread.is_finished() {
            // Wait for the emulator to finish
            sleep(std::time::Duration::from_millis(15));
        }
    }

    /// Translates the passed key and sends it as input to the emulator.
    fn send_user_input(&self, key: &Key) {
        let message = match key {
            Key::A => JoypadInput::DPad(DPadInput::Left),
            Key::D => JoypadInput::DPad(DPadInput::Right),
            Key::W => JoypadInput::DPad(DPadInput::Up),
            Key::S => JoypadInput::DPad(DPadInput::Down),
            Key::ArrowRight => JoypadInput::Action(ActionInput::A),
            Key::ArrowLeft => JoypadInput::Action(ActionInput::B),
            Key::Enter => JoypadInput::Action(ActionInput::Start),
            Key::Backspace => JoypadInput::Action(ActionInput::Select),
            _ => return,
        };
        self.channel
            .sender
            .send(FrontendMessage::Input(message))
            .expect("Unable to send user input message");
    }

    /// Handles user input and sends it to the emulator.
    #[inline]
    fn handle_user_input(&self, ui: &mut Ui) {
        ui.input(|i| {
            for key in &i.keys_down {
                self.send_user_input(key);
            }
        });
    }

    /// Updates the state by checking for new messages from the emulator.
    pub fn update(&mut self) {
        if let Ok(msg) = self.channel.receiver.try_recv() {
            match msg {
                EmulatorMessage::Frame(frame) => {
                    self.latest_frame = frame;
                }
            }
        }
    }

    /// Updates the emulation screen with the latest frame.
    /// TODO: this is a bit hacky, we should probably use a texture cache
    fn update_screen(&self, ctx: &egui::Context, ui: &mut Ui) {
        let image = ColorImage {
            size: [SCREEN_WIDTH * UPSCALE, SCREEN_HEIGHT * UPSCALE],
            pixels: self.latest_frame.buffer.clone(),
        };
        let texture = ctx.load_texture("frame", image, TextureOptions::LINEAR);
        let size = Vec2::new(UPSCALED_WIDTH as f32, UPSCALED_HEIGHT as f32);
        ui.image((texture.id(), size));
        ctx.request_repaint();
    }
}

impl View for EmulatorFrontend {
    fn ui(&self, ctx: &egui::Context, ui: &mut Ui) {
        self.handle_user_input(ui);
        let size = Vec2::new(UPSCALED_WIDTH as f32, UPSCALED_HEIGHT as f32);
        ui.allocate_ui(size, |ui| self.update_screen(ctx, ui));
    }
}
