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

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {}

fn main() -> Result<(), Box<dyn Error>> {
    let _args = Args::parse();
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
