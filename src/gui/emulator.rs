use crate::gb::cartridge::Cartridge;
use crate::gb::joypad::JoypadInput;
use crate::gb::ppu::buffer::FrameBuffer;
use crate::gb::{Emulator, EmulatorConfig, EmulatorMessage, FrontendMessage};
use crate::gui::debugger::DebuggerFrontend;
use eframe::epaint::ColorImage;
use eframe::epaint::textures::TextureOptions;
use egui::{Key, TextureHandle, Ui, Vec2, ViewportBuilder, ViewportId};
use spin_sleep::sleep;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::thread::JoinHandle;

pub const SCREEN_WIDTH: usize = 160;
pub const SCREEN_HEIGHT: usize = 144;

// TODO: make this configurable
pub const UPSCALE: usize = 3;

/// A channel to communicate between the emulator and the frontend.
/// The frontend can send messages with `sender`
/// and receive messages from the emulator with `receiver`.
struct EmulatorChannel {
    pub sender: Sender<FrontendMessage>,
    pub receiver: Receiver<EmulatorMessage>,
}

impl EmulatorChannel {
    #[inline]
    pub const fn new(sender: Sender<FrontendMessage>, receiver: Receiver<EmulatorMessage>) -> Self {
        Self { sender, receiver }
    }
}

/// The emulator frontend is responsible for handling the emulator instance,
/// it runs in a separate thread and communicates with the emulator using a channel.
pub struct EmulatorFrontend {
    thread: JoinHandle<()>,
    channel: EmulatorChannel,
    frame: Option<TextureHandle>,
    debugger: Option<DebuggerFrontend>,
}

impl EmulatorFrontend {
    pub fn update(&mut self, ctx: &egui::Context, ui: &mut Ui) {
        self.handle_user_input(ui);
        self.recv_message(ctx);
        self.draw_emulator_frame(ctx, ui);

        let mut stop_debugger = false;
        if let Some(debugger) = &mut self.debugger {
            ctx.show_viewport_immediate(
                ViewportId::from_hash_of("debugger"),
                ViewportBuilder::default().with_title("Debugger"),
                |ctx, _| {
                    debugger.update(ctx);

                    // Check if the debugger window is closed
                    if ctx.input(|i| i.viewport().close_requested()) {
                        stop_debugger = true;
                    }
                },
            );
        }
        if stop_debugger {
            self.detach_debugger();
        }
    }

    /// Starts the emulator with the given cartridge.
    pub fn start(cartridge: &Cartridge, config: EmulatorConfig) -> Self {
        let (emulator_sender, emulator_receiver) = mpsc::channel();
        let (frontend_sender, frontend_receiver) = mpsc::channel();
        let used_cartridge = cartridge.clone();

        let debugger = match config.debug {
            true => Some(DebuggerFrontend::new(frontend_sender.clone())),
            false => None,
        };

        let thread = thread::spawn(move || {
            let mut emulator =
                Emulator::new(emulator_sender, frontend_receiver, used_cartridge, config);
            emulator.run();
        });
        Self {
            channel: EmulatorChannel::new(frontend_sender, emulator_receiver),
            frame: None,
            thread,
            debugger,
        }
    }

    /// Shuts the emulator down by sending a reset message and waiting for it to finish.
    pub fn shutdown(&self) {
        println!("Stopping emulator ...");
        self.send_message(FrontendMessage::Stop);
        while !self.thread.is_finished() {
            // Wait for the emulator to finish
            sleep(std::time::Duration::from_millis(15));
        }
    }

    /// Attaches a debugger to the frontend and sends a `AttachDebugger` to the emulator
    /// if `send_msg` is true.
    #[inline]
    pub fn attach_debugger(&mut self) {
        if self.debugger.is_some() {
            println!("Debugger already attached");
            return;
        }
        println!("Attaching debugger ...");
        self.debugger = Some(DebuggerFrontend::new(self.channel.sender.clone()));
        self.send_message(FrontendMessage::AttachDebugger);
    }

    /// Closes the debugger frontend and sends a `DetachDebugger` message to the emulator.
    #[inline]
    fn detach_debugger(&mut self) {
        println!("Detaching debugger ...");
        self.debugger = None;
        self.send_message(FrontendMessage::DetachDebugger);
    }

    /// Draws the latest frame from the emulator to the screen
    #[inline]
    fn draw_emulator_frame(&self, ctx: &egui::Context, ui: &mut Ui) {
        if let Some(frame) = &self.frame {
            let size = Vec2::new(
                (SCREEN_WIDTH * UPSCALE) as f32,
                (SCREEN_HEIGHT * UPSCALE) as f32,
            );
            ui.image((frame.id(), size));
            ctx.request_repaint();
        }
    }

    /// Sets the frame texture to the given `FrameBuffer`.
    fn set_frame_texture(&mut self, frame: FrameBuffer, ctx: &egui::Context) {
        let image = ColorImage {
            size: [SCREEN_WIDTH * UPSCALE, SCREEN_HEIGHT * UPSCALE],
            pixels: frame.buffer,
        };
        let options = TextureOptions::LINEAR;

        // Set the new frame to the texture or create a new one if it doesn't exist
        if let Some(frame) = &mut self.frame {
            frame.set(image, options);
        } else {
            self.frame = Some(ctx.load_texture("frame", image, options));
        }
    }

    /// Checks for messages from the emulator and updates the state if necessary.
    fn recv_message(&mut self, ctx: &egui::Context) {
        // TODO: consider checking if there are multiple messages
        if let Ok(msg) = self.channel.receiver.try_recv() {
            match msg {
                EmulatorMessage::Frame(frame) => self.set_frame_texture(frame, ctx),
                EmulatorMessage::Debug(message) => {
                    if let Some(debugger) = &mut self.debugger {
                        debugger.handle_message(*message);
                    }
                }
            }
        }
    }

    /// Sends a message to the emulator.
    #[inline]
    fn send_message(&self, message: FrontendMessage) {
        if let Err(msg) = self.channel.sender.send(message) {
            eprintln!("Emulator isn't running: {msg}");
        }
    }

    /// Handles user input and sends it to the emulator.
    fn handle_user_input(&self, ui: &mut Ui) {
        ui.input(|i| {
            let mut input = JoypadInput::default();
            for key in &i.keys_down {
                match key {
                    Key::A => input.left = true,
                    Key::D => input.right = true,
                    Key::W => input.up = true,
                    Key::S => input.down = true,
                    Key::ArrowRight => input.a = true,
                    Key::ArrowLeft => input.b = true,
                    Key::Enter => input.start = true,
                    Key::Backspace => input.select = true,
                    _ => {}
                }
            }
            if input.is_pressed() {
                self.send_message(FrontendMessage::Input(input));
            }
        });
    }
}
