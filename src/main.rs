#![allow(clippy::upper_case_acronyms)]

#[macro_use]
extern crate bitflags;
#[macro_use]
extern crate clap;
use crate::gb::bus::Bus;
use crate::gb::cartridge::Cartridge;
use crate::gb::cpu::CPU;
use crate::gb::display::Display;
use crate::gb::ppu::PPU;
use crate::gb::timer::Timer;
use crate::gb::{interrupt, DISPLAY_REFRESH_RATE};
use backtrace::Backtrace;
use clap::{App, Arg, ArgMatches};
use std::error::Error;
use std::panic;
use std::panic::PanicHookInfo;
use std::path::Path;

mod gb;
mod utils;

fn main() -> Result<(), Box<dyn Error>> {
    panic::set_hook(Box::new(|info| {
        panic_hook(info);
    }));

    let matches = parse_args();
    let path = Path::new(matches.value_of("rom").unwrap());

    let fps_limit = match matches.is_present("no-fps-limit") {
        true => 0,
        false => DISPLAY_REFRESH_RATE,
    };
    let debug = matches.is_present("debug");

    println!("Loading cartridge {}...", &path.display());
    let cartridge = Cartridge::from_path(path).expect("Unable to load cartridge from path");
    println!("  -> {}", &cartridge.meta);

    let mut cpu = CPU::new();
    let mut bus = Bus::new(cartridge);
    let mut display = Display::new(2, fps_limit).expect("Unable to create sdl2 Display");
    let mut ppu = PPU::new(&mut display);
    let mut timer = Timer::new();

    match debug {
        true => {
            // TODO: Adapt debugger for new architecture
            //let mut debugger = Debugger::new(&cpu, &bus, &mut ppu, &mut timer, &mut irq_handler);
            //debugger.emulate()?
        }
        false => emulate(&mut cpu, &mut bus, &mut ppu, &mut timer),
    }
    Ok(())
}

/// Starts the emulating loop
fn emulate(
    cpu: &mut CPU,
    bus: &mut Bus,
    ppu: &mut PPU,
    timer: &mut Timer,
) {
    loop {
        let cycles = cpu.step(bus);
        timer.step(bus, cycles);
        ppu.step(bus, cycles);
        interrupt::handle(cpu, bus);
    }
}

fn parse_args() -> ArgMatches<'static> {
    App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about("Experimental GB Emulator")
        .arg(
            Arg::with_name("rom")
                .help("Path of the ROM to load")
                .index(1)
                .required(true)
                .value_name("ROM")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("debug")
                .help("Enable debugger")
                .long("debug"),
        )
        .arg(
            Arg::with_name("no-fps-limit")
                .help("Disable fps limit for debugging purposes")
                .long("no-fps-limit"),
        )
        .get_matches()
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
