#![allow(clippy::upper_case_acronyms)]

#[macro_use]
extern crate bitflags;
extern crate clap;
use crate::gb::bus::Bus;
use crate::gb::cartridge::Cartridge;
use crate::gb::cpu::CPU;
use crate::gb::display::Display;
use crate::gb::ppu::PPU;
use crate::gb::{interrupt, DISPLAY_REFRESH_RATE};
use backtrace::Backtrace;
use clap::Parser;
use std::error::Error;
use std::panic;
use std::panic::PanicHookInfo;
use std::path::Path;

mod gb;
mod utils;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path of the ROM to load
    #[arg(value_name = "FILE")]
    rom: String,

    /// Disable fps limit for debugging purposes
    #[arg(short, long)]
    no_fps_limit: bool,

    /// Enable debugger
    #[arg(short, long)]
    debug: bool,
}

fn main() -> Result<(), Box<dyn Error>> {
    panic::set_hook(Box::new(|info| {
        panic_hook(info);
    }));

    let args = Args::parse();
    let rom_path = Path::new(&args.rom);
    let fps_limit = if args.no_fps_limit {
        0
    } else {
        DISPLAY_REFRESH_RATE
    };

    println!("Loading cartridge {}...", &rom_path.display());
    let cartridge = Cartridge::from_path(rom_path).expect("Unable to load cartridge from path");
    println!("  -> {}", &cartridge.meta);

    let mut cpu = CPU::new();
    let mut bus = Bus::new(cartridge);
    let mut display = Display::new(2, fps_limit).expect("Unable to create sdl2 Display");
    let mut ppu = PPU::new(&mut display);

    match args.debug {
        true => {
            // TODO: Adapt debugger for new architecture
            //let mut debugger = Debugger::new(&cpu, &bus, &mut ppu, &mut timer, &mut irq_handler);
            //debugger.emulate()?
        }
        false => emulate(&mut cpu, &mut bus, &mut ppu),
    }
    Ok(())
}

/// Starts the emulating loop
fn emulate(cpu: &mut CPU, bus: &mut Bus, ppu: &mut PPU) {
    loop {
        let cycles = cpu.step(bus);
        bus.step(cycles);
        ppu.step(bus, cycles);
        interrupt::handle(cpu, bus);
    }
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
