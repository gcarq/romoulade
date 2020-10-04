use crate::gb::cartridge::Cartridge;
use crate::gb::cpu::CPU;
use crate::gb::display::Display;
use crate::gb::interrupt::IRQHandler;
use crate::gb::memory::MemoryBus;
use crate::gb::ppu::PPU;
use crate::gb::timer::Timer;
use crate::gb::{debugger, AddressSpace, DISPLAY_REFRESH_RATE};
use clap::{App, Arg, ArgMatches};
use std::cell::RefCell;
use std::error::Error;
use std::path::Path;

#[macro_use]
extern crate clap;
#[macro_use]
extern crate bitflags;

mod gb;
mod utils;

fn main() -> Result<(), Box<dyn Error>> {
    let matches = parse_args();
    let path = Path::new(matches.value_of("rom").unwrap());

    let fps_limit = match matches.is_present("no-fps-limit") {
        true => 0,
        false => DISPLAY_REFRESH_RATE,
    };
    let debug = matches.is_present("debug");

    println!("Loading cartridge {}...", &path.display());
    let cartridge = Cartridge::from_path(&path).expect("Unable to load cartridge from path");
    println!("  -> {}", &cartridge.meta);

    let bus = RefCell::new(MemoryBus::new(cartridge));
    let mut display = Display::new(2, fps_limit).expect("Unable to create sdl2 Display");
    let mut ppu = PPU::new(&bus, &mut display);
    let cpu = RefCell::new(CPU::new(&bus));
    let mut irq_handler = IRQHandler::new(&cpu, &bus);
    let mut timer = Timer::new(&bus);

    match debug {
        true => debugger::emulate(&cpu, &bus, &mut ppu, &mut timer, &mut irq_handler)?,
        false => emulate(&cpu, &mut ppu, &mut timer, &mut irq_handler),
    }
    Ok(())
}

/// Starts the emulating loop
fn emulate<T: AddressSpace>(
    cpu: &RefCell<CPU<T>>,
    ppu: &mut PPU,
    timer: &mut Timer,
    irq_handler: &mut IRQHandler<T>,
) {
    loop {
        let cycles = cpu.borrow_mut().step();
        timer.step(cycles);
        ppu.step(cycles);
        irq_handler.handle();
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
