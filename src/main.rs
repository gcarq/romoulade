#![allow(clippy::upper_case_acronyms)]

#[macro_use]
extern crate bitflags;
extern crate clap;
use crate::gui::Romoulade;
use crate::gui::emulator::UPSCALE;
use clap::Parser;
use eframe::{HardwareAcceleration, egui};
use std::error::Error;
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

fn main() -> Result<(), Box<dyn Error>> {
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
    Ok(())
}
