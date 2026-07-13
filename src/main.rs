#[macro_use]
extern crate bitflags;
extern crate clap;

use crate::gb::cartridge::Cartridge;
use crate::gb::{Emulator, EmulatorConfig, SCREEN_HEIGHT};
use crate::gui::emulator::SCREEN_WIDTH;
use crate::gui::{PANEL_HEIGHT, Romoulade};
use anyhow::{Context, Result};
use clap::Parser;
use eframe::egui;
use std::path::PathBuf;
use std::sync::mpsc;

mod gb;
mod gui;
mod perf;

const DEFAULT_UPSCALE: usize = 3; // Default upscale factor for the display

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

fn main() -> Result<()> {
    let args = Args::parse();

    let config = EmulatorConfig {
        rom: args.rom,
        upscale: DEFAULT_UPSCALE,
        fastboot: args.fastboot,
        print_serial: args.print_serial,
        headless: args.headless,
        autosave: true, // TODO: make this configurable
        savefile: None, // TODO: make this configurable
    };

    if config.headless {
        headless_mode(config)
    } else {
        gui_mode(config)
    }
}

/// Starts the emulator with an `egui` frontend.
fn gui_mode(config: EmulatorConfig) -> Result<()> {
    let size = egui::vec2(
        SCREEN_WIDTH as f32 * DEFAULT_UPSCALE as f32,
        SCREEN_HEIGHT as f32 * DEFAULT_UPSCALE as f32 + PANEL_HEIGHT * 2.0,
    );
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_resizable(false)
            .with_inner_size(size),
        ..Default::default()
    };

    let app = Romoulade::new(config).with_context(|| "Unable to start emulator")?;
    eframe::run_native("Romoulade", options, Box::new(|_| Ok(Box::new(app))))
        .with_context(|| "Failed to run egui app")?;
    Ok(())
}

/// Starts the emulator in headless mode.
fn headless_mode(config: EmulatorConfig) -> Result<()> {
    let rom = config
        .rom
        .as_ref()
        .with_context(|| "No ROM path provided for headless mode")?;
    let cartridge = Cartridge::try_from(rom.as_path())?;
    let (emulator_sender, _) = mpsc::sync_channel(2);
    let (_, frontend_receiver) = mpsc::channel();
    let mut emulator = Emulator::headless(emulator_sender, frontend_receiver, cartridge, config);
    emulator
        .run()
        .with_context(|| "Failed to run emulator in headless mode")
}
