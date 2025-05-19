use crate::gb::cartridge::{
    CARTRIDGE_GLOBAL_CHECKSUM1, CARTRIDGE_GLOBAL_CHECKSUM2, CartridgeConfig, ControllerType,
    calculate_global_checksum, rom_bank_mask, verify_checksum,
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
fn test_verify_checksum_ok() {
    let mut buf = (0..=CARTRIDGE_GLOBAL_CHECKSUM2)
        .map(|i| i as u8)
        .collect::<Vec<u8>>();
    buf[CARTRIDGE_GLOBAL_CHECKSUM1 as usize] = 0x8B;
    buf[CARTRIDGE_GLOBAL_CHECKSUM2 as usize] = 0x3B;
    assert!(verify_checksum(&buf).is_ok());
}

#[test]
fn test_verify_checksum_buffer_too_small() {
    let buf = (0..=10).map(|i| i as u8).collect::<Vec<u8>>();
    assert!(verify_checksum(&buf).is_err());
}

#[test]
fn test_verify_checksum_buffer_invalid_checksum() {
    let mut buf = (0..=CARTRIDGE_GLOBAL_CHECKSUM2)
        .map(|i| i as u8)
        .collect::<Vec<u8>>();
    buf[CARTRIDGE_GLOBAL_CHECKSUM1 as usize] = 0x00;
    buf[CARTRIDGE_GLOBAL_CHECKSUM2 as usize] = 0x00;
    assert!(verify_checksum(&buf).is_err());
}

#[test]
fn test_cartridge_config() {
    let config = CartridgeConfig::new(ControllerType::MBC1, 0x02, 0x03).unwrap();
    assert_eq!(config.controller, ControllerType::MBC1);
    assert_eq!(config.rom_banks, 8);
    assert_eq!(config.ram_size, 32768);
    assert_eq!(config.ram_banks, 4);
}

#[test]
fn test_rom_bank_mask() {
    assert_eq!(rom_bank_mask(2), 0b11);
    assert_eq!(rom_bank_mask(4), 0b111);
    assert_eq!(rom_bank_mask(8), 0b1111);
    assert_eq!(rom_bank_mask(16), 0b11111);
    assert_eq!(rom_bank_mask(32), 0b111111);
    assert_eq!(rom_bank_mask(64), 0b1111111);
    assert_eq!(rom_bank_mask(128), 0b11111111);
}
