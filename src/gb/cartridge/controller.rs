use crate::gb::cartridge::mbc1::MBC1;
use crate::gb::cartridge::mbc3::MBC3;
use crate::gb::cartridge::mbc5::MBC5;
use crate::gb::cartridge::nombc::NoMBC;
use crate::gb::cartridge::{CartridgeConfig, ControllerType};
use dyn_clone::DynClone;
use std::sync::Arc;

/// This trait defines the interface for a bank controller.
/// It allows reading and writing to different banks of the cartridge.
pub trait BankController: DynClone + Send {
    fn read(&mut self, address: u16) -> u8;

    fn write(&mut self, address: u16, value: u8);
}

dyn_clone::clone_trait_object!(BankController);

/// Creates a new `BankController` with the given ROM and `CartridgeConfig`.
pub fn new(config: CartridgeConfig, rom: Arc<[u8]>) -> Box<dyn BankController> {
    match config.controller {
        ControllerType::NoMBC => Box::new(NoMBC::new(config, rom)),
        ControllerType::MBC1 => Box::new(MBC1::new(config, rom)),
        ControllerType::MBC2 => todo!("MBC2 is not implemented"),
        ControllerType::MBC3 => Box::new(MBC3::new(config, rom)),
        ControllerType::MBC5 => Box::new(MBC5::new(config, rom)),
        ControllerType::MBC6 => todo!("MBC6 is not implemented"),
        ControllerType::MBC7 => todo!("MBC7 is not implemented"),
    }
}
