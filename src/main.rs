#![allow(clippy::upper_case_acronyms)]
#![warn(clippy::semicolon_if_nothing_returned)]
#![warn(clippy::unnecessary_semicolon)]
#![warn(clippy::missing_const_for_fn)]

#[macro_use]
extern crate bitflags;
extern crate clap;

use crate::gb::cartridge::Cartridge;
use crate::gb::{Emulator, EmulatorConfig};
use crate::gui::Romoulade;
use crate::gui::emulator::UPSCALE;
use clap::Parser;
use eframe::{HardwareAcceleration, egui};
use std::path::PathBuf;
use std::sync::mpsc;

mod gb;
mod gui;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to the Game Boy ROM
    #[arg(short, long)]
    rom: Option<PathBuf>,

    /// Enable the debugger immediately
    #[arg(short, long)]
    debug: bool,

    /// Start immediately and skip boot ROM
    #[arg(short, long)]
    fastboot: bool,

    /// Print serial data to stdout
    #[arg(short, long)]
    print_serial: bool,

    /// Start the emulator in headless mode
    #[arg(long)]
    headless: bool,
}

fn main() {
    let args = Args::parse();

    let config = EmulatorConfig {
        rom: args.rom,
        upscale: UPSCALE, // TODO: make this configurable
        fastboot: args.fastboot,
        print_serial: args.print_serial,
        headless: args.headless,
    };

    if config.headless {
        headless_mode(config);
    } else {
        gui_mode(config);
    }
}

/// Starts the emulator with an `egui` frontend.
fn gui_mode(config: EmulatorConfig) {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_resizable(false)
            .with_inner_size([503.0, 475.0]),
        hardware_acceleration: HardwareAcceleration::Preferred,
        ..Default::default()
    };

    let app = Romoulade::new(config).expect("Failed to create Romoulade instance");
    eframe::run_native("Romoulade", options, Box::new(|_| Ok(Box::new(app))))
        .expect("Unable to run egui app");
}

/// Starts the emulator in headless mode.
fn headless_mode(config: EmulatorConfig) {
    let rom = config
        .rom
        .as_ref()
        .expect("No ROM path provided for headless mode");
    let cartridge = Cartridge::try_from(rom.as_path()).expect("Failed to load cartridge");
    let (emulator_sender, _) = mpsc::channel();
    let (_, frontend_receiver) = mpsc::channel();
    let mut emulator = Emulator::new(emulator_sender, frontend_receiver, cartridge, config);
    emulator.run();
}
