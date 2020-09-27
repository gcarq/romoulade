use crate::gb::cartridge::Cartridge;
use crate::gb::cpu::CPU;
use crate::gb::display::Display;
use crate::gb::interrupt::IRQHandler;
use crate::gb::memory::MemoryBus;
use crate::gb::ppu::PPU;
use crate::gb::timer::Timer;
use crate::gb::DISPLAY_REFRESH_RATE;
use clap::{App, Arg, ArgMatches};
use std::cell::RefCell;
use std::path::Path;

#[macro_use]
extern crate clap;
#[macro_use]
extern crate bitflags;

mod gb;
mod utils;

fn main() {
    let matches = parse_args();
    let path = Path::new(matches.value_of("rom").unwrap());

    let fps_limit = match matches.is_present("no-fps-limit") {
        true => 0,
        false => DISPLAY_REFRESH_RATE,
    };

    println!("Loading cartridge {}...", &path.display());
    let cartridge = Cartridge::from_path(&path).expect("Unable to load cartridge from path");
    println!("  -> {}", cartridge);

    let bus = RefCell::new(MemoryBus::new(cartridge));
    let mut display = Display::new(2, fps_limit).expect("Unable to create sdl2 Display");
    let mut ppu = PPU::new(&bus, &mut display);
    let cpu = RefCell::new(CPU::new(&bus));
    let mut irq_handler = IRQHandler::new(&cpu);
    let mut timer = Timer::new(&bus);

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
            Arg::with_name("no-fps-limit")
                .help("Disable fps limit for debugging purposes")
                .long("no-fps-limit"),
        )
        .get_matches()
}
