#![allow(clippy::upper_case_acronyms)]

#[macro_use]
extern crate bitflags;
extern crate clap;
use crate::gui::Romoulade;
use backtrace::Backtrace;
use clap::Parser;
use eframe::{HardwareAcceleration, egui};
use std::error::Error;
use std::panic;
use std::panic::PanicHookInfo;

mod gb;
mod gui;
mod utils;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Enable debugger
    #[arg(short, long)]
    debug: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    panic::set_hook(Box::new(|info| {
        panic_hook(info);
    }));

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

fn panic_hook(info: &PanicHookInfo<'_>) {
    if cfg!(debug_assertions) {
        let location = info.location().unwrap();

        let msg = match info.payload().downcast_ref::<&'static str>() {
            Some(s) => *s,
            None => match info.payload().downcast_ref::<String>() {
                Some(s) => &s[..],
                None => "Box<Any>",
            },
        };

        let stacktrace: String = format!("{:?}", Backtrace::new()).replace('\n', "\n\r");

        println!(
            "{}thread '<unnamed>' panicked at '{}', {}\n\r{}",
            termion::screen::ToMainScreen,
            msg,
            location,
            stacktrace
        );
    }
}
