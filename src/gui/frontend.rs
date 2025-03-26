use crate::gb::cartridge::Cartridge;
use crate::gb::joypad::{ActionInput, DPadInput, JoypadInput};
use crate::gb::ppu::buffer::FrameBuffer;
use crate::gb::{Emulator, EmulatorMessage};
use crate::gui::{EmulatorChannel, FrontendMessage, UPSCALE};
use egui::Key;
use std::sync::mpsc;
use std::thread;
use std::thread::{JoinHandle, sleep};

/// Holds the emulation frontend that is responsible for the interaction with the emulation backend.
pub struct EmulatorFrontend {
    pub thread: JoinHandle<()>,
    pub channel: EmulatorChannel,
    pub cartridge: Cartridge,
}

impl EmulatorFrontend {
    /// Starts the emulator with the given cartridge.
    pub fn start(cartridge: Cartridge) -> Self {
        let (emulator_sender, emulator_receiver) = mpsc::channel();
        let (frontend_sender, frontend_receiver) = mpsc::channel();
        let used_cartridge = cartridge.clone();
        let thread = thread::spawn(move || {
            let mut emulator =
                Emulator::new(emulator_sender, frontend_receiver, used_cartridge, UPSCALE)
                    .expect("Unable to create GameBoy instance");
            emulator.run();
        });

        let channel = EmulatorChannel::new(frontend_sender, emulator_receiver);
        Self {
            thread,
            channel,
            cartridge,
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
    pub fn send_user_input(&self, key: &Key) {
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

    /// Receives and returns the current frame from the emulator.
    #[inline]
    pub fn recv_frame(&self) -> Option<FrameBuffer> {
        match self.channel.receiver.recv() {
            Ok(EmulatorMessage::Frame(frame)) => Some(frame),
            _ => None,
        }
    }
}
