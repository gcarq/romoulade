use crate::gb::cartridge::{
    CARTRIDGE_GLOBAL_CHECKSUM1, CARTRIDGE_GLOBAL_CHECKSUM2, CartridgeConfig, ControllerType,
    calculate_global_checksum, verify_checksum,
};

#[test]
fn test_calculate_global_checksum() {
    let buf = (0..CARTRIDGE_GLOBAL_CHECKSUM2)
        .map(|i| i as u8)
        .collect::<Vec<u8>>();
    let checksum = calculate_global_checksum(&buf);
    assert_eq!(checksum, 0x8B3B);
}

#[test]
fn test_verify_checksum() {
    let mut buf = (0..=CARTRIDGE_GLOBAL_CHECKSUM2)
        .map(|i| i as u8)
        .collect::<Vec<u8>>();
    buf[CARTRIDGE_GLOBAL_CHECKSUM1 as usize] = 0x8B;
    buf[CARTRIDGE_GLOBAL_CHECKSUM2 as usize] = 0x3B;
    assert!(verify_checksum(&buf).is_ok());
}

#[test]
fn test_cartridge_config() {
    let config = CartridgeConfig::new(ControllerType::MBC1WithRAM, 0x02, 0x03).unwrap();
    assert_eq!(config.controller, ControllerType::MBC1WithRAM);
    assert_eq!(config.rom_size, 131072);
    assert_eq!(config.rom_banks, 8);
    assert_eq!(config.ram_size, 32768);
    assert_eq!(config.ram_banks, 4);
}
