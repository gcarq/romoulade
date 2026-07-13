use crate::gb::cartridge::Cartridge;
use crate::gb::joypad::JoypadInputState;
use crate::gb::ppu::buffer::FrameBuffer;
use crate::gb::{Emulator, EmulatorConfig, EmulatorMessage, FrontendMessage};
use crate::gui::debugger::DebuggerFrontend;
use crate::perf::PerformanceCounter;
use eframe::egui::load::SizedTexture;
use eframe::egui::{self, Image};
use eframe::egui::{Color32, Key, TextureHandle, Ui, Vec2, ViewportBuilder, ViewportId};
use eframe::epaint::ColorImage;
use eframe::epaint::textures::TextureOptions;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::thread::JoinHandle;

pub const SCREEN_WIDTH: usize = 160;
pub const SCREEN_HEIGHT: usize = 144;

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

/// The emulator frontend is responsible for handling the emulator instance,
/// it runs in a separate thread and communicates with the emulator using a channel.
pub struct EmulatorFrontend {
    thread: JoinHandle<()>,
    channel: EmulatorChannel,
    frame: TextureHandle,
    debugger: Option<DebuggerFrontend>,
    last_input: JoypadInputState,
    pub fps_counter: PerformanceCounter,
}

impl EmulatorFrontend {
    /// Starts the emulator with the given cartridge.
    pub fn start(ctx: &egui::Context, cartridge: &Cartridge, config: EmulatorConfig) -> Self {
        let (emulator_sender, emulator_receiver) = mpsc::sync_channel(2);
        let (frontend_sender, frontend_receiver) = mpsc::channel();
        let cartridge = cartridge.clone();
        let repaint_ctx = ctx.clone();

        let thread = thread::spawn(move || {
            let mut emulator = Emulator::with_display(
                emulator_sender,
                frontend_receiver,
                cartridge,
                repaint_ctx,
                config,
            );
            if let Err(msg) = emulator.run() {
                eprintln!("Emulator error: {msg}");
            }
        });

        // Create initial TextureHandle for the frame buffer
        let image = ColorImage::new([1, 1], vec![Color32::TRANSPARENT]);
        let frame = ctx.load_texture("frame", image, TextureOptions::NEAREST);

        Self {
            channel: EmulatorChannel::new(frontend_sender, emulator_receiver),
            debugger: None,
            last_input: JoypadInputState::default(),
            fps_counter: PerformanceCounter::default(),
            frame,
            thread,
        }
    }

    /// Updates the emulator frontend Ui.
    pub fn update(&mut self, ui: &mut Ui, display_size: Vec2) {
        self.handle_user_input(ui);
        self.recv_message();
        self.draw_emulator_frame(ui, display_size);
        if !self.update_debugger(ui.ctx()) {
            self.detach_debugger();
        }
    }

    /// Stops the emulator by sending a reset message and waiting for it to finish.
    pub fn stop(&self) {
        println!("Stopping emulator ...");
        self.send_message(FrontendMessage::Stop);
        // Wait for the emulator to finish
        while !self.thread.is_finished() {
            let _ = self.channel.receiver.try_recv();
            spin_sleep::sleep(std::time::Duration::from_millis(15));
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

    /// Sends a message to the emulator.
    #[inline]
    pub fn send_message(&self, message: FrontendMessage) {
        if let Err(msg) = self.channel.sender.send(message) {
            eprintln!("Emulator isn't running: {msg}");
        }
    }

    /// Updates the debugger frontend.
    /// Returns false if the debugger was closed and should be detached.
    fn update_debugger(&mut self, ctx: &egui::Context) -> bool {
        let mut is_running = true;
        if let Some(debugger) = &mut self.debugger {
            ctx.show_viewport_immediate(
                ViewportId::from_hash_of("debugger"),
                ViewportBuilder::default()
                    .with_title("Debugger")
                    .with_inner_size(Vec2::new(1130.0, 790.0))
                    .with_resizable(false),
                |ctx, _| {
                    debugger.update(ctx);
                    // Check if the debugger window is closed
                    if ctx.input(|i| i.viewport().close_requested()) {
                        is_running = false;
                    }
                },
            );
        }
        is_running
    }

    /// Closes the debugger frontend and sends a `DetachDebugger` message to the emulator.
    #[inline]
    fn detach_debugger(&mut self) {
        println!("Detaching debugger ...");
        self.debugger = None;
        self.send_message(FrontendMessage::DetachDebugger);
    }

    /// Draws the latest frame from the emulator to the screen
    fn draw_emulator_frame(&self, ui: &mut Ui, display_size: Vec2) {
        let image = Image::from_texture(SizedTexture::from_handle(&self.frame));
        ui.add(image.fit_to_exact_size(display_size));
    }

    /// Sets the frame texture to the given `FrameBuffer`.
    fn set_frame_texture(&mut self, frame: FrameBuffer) {
        let image = ColorImage::new([frame.width(), frame.height()], frame.into_vec());
        self.frame.set(image, TextureOptions::NEAREST);
    }

    /// Checks for messages from the emulator and updates the state if necessary.
    fn recv_message(&mut self) {
        if let Ok(msg) = self.channel.receiver.try_recv() {
            match msg {
                EmulatorMessage::Frame(frame) => {
                    self.set_frame_texture(frame);
                    self.fps_counter.update();
                }
                EmulatorMessage::Debug(message) => match &mut self.debugger {
                    Some(dbg) => dbg.handle_message(*message),
                    None => eprintln!("Got debug message, but debugger is not attached"),
                },
            }
        }
    }

    /// Handles user input and sends it to the emulator.
    fn handle_user_input(&mut self, ui: &mut Ui) {
        let input = ui.input(|i| {
            let mut input = JoypadInputState::default();
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
            input
        });

        if input != self.last_input {
            self.send_message(FrontendMessage::Input(input));
            self.last_input = input;
        }
    }
}
