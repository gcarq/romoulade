#![allow(clippy::upper_case_acronyms)]
#![warn(clippy::semicolon_if_nothing_returned)]
#![warn(clippy::unnecessary_semicolon)]
#![warn(clippy::missing_const_for_fn)]
#![warn(clippy::doc_markdown)]

#[macro_use]
extern crate bitflags;
extern crate clap;
use crate::gui::emulator::UPSCALE;
use crate::gui::Romoulade;
use clap::Parser;
use eframe::{egui, HardwareAcceleration};
use std::path::PathBuf;

mod gb;
mod gui;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to the Game Boy ROM
    #[arg(short, long)]
    rom: Option<PathBuf>,

    /// Starts and skips the boot ROM
    #[arg(short, long)]
    fastboot: bool,
}

fn main() {
    let args = Args::parse();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_resizable(false)
            .with_inner_size([503.0, 475.0]),
        hardware_acceleration: HardwareAcceleration::Preferred,
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };

    let config = gb::EmulatorConfig {
        upscale: UPSCALE, // TODO: make this configurable
        debug: false,
        fastboot: args.fastboot,
    };

    let mut app = Romoulade::new(config);
    if let Some(rom) = args.rom {
        app.load_cartridge(&rom).expect("Failed to load cartridge");
    }

    eframe::run_native("Romoulade", options, Box::new(|_| Ok(Box::new(app))))
        .expect("Unable to run egui app");
}
