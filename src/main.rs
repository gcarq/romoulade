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
use fern::colors::{Color, ColoredLevelConfig};
use log::LevelFilter;
use std::io;
use std::path::PathBuf;
use std::sync::{LazyLock, mpsc};

mod gb;
mod gui;
mod perf;

const DEFAULT_UPSCALE: usize = 3; // Default upscale factor for the display

/// Colors for log levels.
static COLORS: LazyLock<ColoredLevelConfig> = LazyLock::new(|| {
    ColoredLevelConfig::new()
        .error(Color::Red)
        .warn(Color::Yellow)
        .info(Color::Green)
        .debug(Color::Cyan)
        .trace(Color::BrightCyan)
});

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
    setup_logger(LevelFilter::Info)?;
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

/// Sets up application logger with the given `log_level`.
fn setup_logger(level_filter: log::LevelFilter) -> Result<()> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}] {} - {message}",
                COLORS.color(record.level()),
                record.target()
            ));
        })
        .level(level_filter)
        .chain(io::stdout())
        .apply()
        .context("failed to initialize logging")?;

    Ok(())
}
