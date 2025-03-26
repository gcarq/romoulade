#![allow(clippy::upper_case_acronyms)]

#[macro_use]
extern crate bitflags;
extern crate clap;
use crate::gui::Romoulade;
use clap::Parser;
use eframe::{HardwareAcceleration, egui};
use std::error::Error;

mod gb;
mod gui;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Enable debugger
    #[arg(short, long)]
    debug: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    if args.debug {
        // TODO: Implement debugger
        println!("Debugger enabled");
    }
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_resizable(false)
            .with_inner_size([503.0, 475.0]),
        hardware_acceleration: HardwareAcceleration::Preferred,
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };

    let app = Romoulade::default();

    eframe::run_native("Romoulade", options, Box::new(|_| Ok(Box::new(app))))
        .expect("Unable to run egui app");
    Ok(())
}
