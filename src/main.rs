use crate::gb::cpu::CPU;
use crate::gb::display::Display;
use crate::gb::interrupt::IRQHandler;
use crate::gb::memory::MemoryBus;
use crate::gb::ppu::PPU;
use crate::gb::timings::Timer;
use clap::{App, Arg, ArgMatches};
use std::cell::RefCell;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::{fs, io};

#[macro_use]
extern crate clap;
#[macro_use]
extern crate bitflags;

mod gb;
mod utils;

struct ROMLoader<'a> {
    path: &'a Path,
}

impl<'a> ROMLoader<'a> {
    pub fn new(path: &'a Path) -> Self {
        Self { path }
    }

    pub fn load(&self) -> io::Result<Vec<u8>> {
        let mut file = File::open(&self.path)?;
        let metadata = fs::metadata(&self.path)?;
        let mut buffer = vec![0; metadata.len() as usize];
        file.read(&mut buffer)?;

        println!(
            "Loaded rom '{}' with {} bytes",
            self.path.display(),
            buffer.len()
        );
        Ok(buffer)
    }
}

fn main() {
    let matches = parse_args();
    let path = matches.value_of("rom").unwrap();

    let rom = ROMLoader::new(Path::new(&path)).load().unwrap();
    let bus = RefCell::new(MemoryBus::new(rom));

    let mut display = Display::new(2);
    let mut timer = Timer::new(&bus);
    let mut ppu = PPU::new(&bus, &mut display);
    let cpu = RefCell::new(CPU::new(&bus));
    let mut irq_handler = IRQHandler::new(&cpu);

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
        .get_matches()
}
