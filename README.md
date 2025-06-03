# Romoulade

A Game Boy Emulator written in Rust.

At this point, a lot of tested games are playable; however, there are still some bugs.

<img src="https://github.com/user-attachments/assets/cb0598a6-6fa8-4653-b134-b25d93558e55" width="240" height="240">
<img src="https://github.com/user-attachments/assets/e621bef7-866a-4224-8f6f-bdc08ce19d61" width="240" height="240">
<img src="https://github.com/user-attachments/assets/b209c4ea-c4c2-47a6-85f0-6522ed166ba2" width="240" height="240">
<img src="https://github.com/user-attachments/assets/e721c8ca-d585-46e2-a268-eec63f7e92cd" width="240" height="240">

## Quick Start

Clone this repository, build and run it with cargo:

```
git clone https://github.com/gcarq/romoulade.git
cd romoulade/
cargo run --release
```

## Key Bindings

| Keyboard    | Emulator |
|-------------|----------|
| Arrow Right | A        |
| Arrow Left  | B        |
| Enter       | Start    |
| Backspace   | Select   |
| W           | Up       |
| A           | Left     |
| S           | Down     |
| D           | Right    |

## State

This project started as an exploratory project and aims to have an accurate
DMG emulator with cross-platform support and good documentation.
The Frontend has been built with [egui.rs](https://github.com/emilk/egui), but the emulator can also be started in
headless mode (see usage below).

#### Known Issues and TODOs

- Implement remaining MBCs (MBC2, MBC6, MBC7, ...)
- Implement Sound Processing Unit
- Pass acceptance tests from [the Mooneye Test Suite](https://github.com/Gekkio/mooneye-test-suite)
- Improve debugger UI
- Add option for dynamic screen upscaling

### Blargg's Test ROMs

| Test No.     | Result | Remark  |
|--------------|--------|---------|
| cpu_instrs   | ✅      | Passed  |
| instr_timing | ✅      | Passed  |
| mem_timing   | ✅      | Passed  |
| mem_timing-2 | ✅      | Passed  |
| oam_bug      | ❌      | Failed  |
| halt_bug     | ✅      | Passed  |
| dmg_sound    | ❌      | Missing |

### Mooneye Acceptance Tests

The test ROMs are taken from the
commit [a1adfe2](https://github.com/Gekkio/mooneye-test-suite/commit/a1adfe27ba6517d8f4d14d16088e23ce6bbf4d55).

| Test Name  | Passed | Failing Tests                                                                                                                     |
|------------|--------|-----------------------------------------------------------------------------------------------------------------------------------|
| common     | 26/31  | <sub>boot_div, boot_hwio, di_timing, halt_ime0_nointr_timing, halt_ime1_timing2</sub>                                             |
| bits       | 3/3    | ✅                                                                                                                                 |
| instr      | 1/1    | ✅                                                                                                                                 |
| interrupts | 1/1    | ✅                                                                                                                                 |
| oam_dma    | 2/3    | <sub>sources</sub>                                                                                                                |
| ppu        | 6/12   | <sub>intr_2_mode0_timing_sprites, hblank_ly_scx_timing, lcdon_timing, lcdon_write_timing, stat_irq_blocking, stat_lyc_onoff</sub> |
| serial     | 0/1    | <sub>boot_sclk_align</sub>                                                                                                        |
| timer      | 10/13  | <sub>rapid_toggle, tima_write_reloading, tma_write_reloading</sub>                                                                |

## Debugger

This emulator comes with a visual debugger built using [egui.rs](https://github.com/emilk/egui).
The debugger can be attached and detached at any time without a performance penalty.
![Screenshot_20250526_152543](https://github.com/user-attachments/assets/fc9fca26-6af5-4559-8046-7b042f6e1864)

## Command Line Options

```
Usage: romoulade [OPTIONS]

Options:
  -r, --rom <ROM>     Path to the Game Boy ROM
  -d, --debug         Enable the debugger immediately
  -f, --fastboot      Start immediately and skip boot ROM
  -p, --print-serial  Print serial data to stdout
      --headless      Start the emulator in headless mode
  -h, --help          Print help
  -V, --version       Print version
```

## Contributing

This repository is open to contributions.
The code should follow the [Rust Style Guide](https://doc.rust-lang.org/stable/style-guide/) and shouldn't produce any
`clippy` warnings.

## Dependencies

* Rust >= 1.87.0

## Resources

* [The Ultimate Game Boy Talk (33C3)](https://www.youtube.com/watch?v=HyzD8pNlpwI)
* [Game Boy CPU Manual](http://marc.rawer.de/Gameboy/Docs/GBCPUman.pdf)
* [Game Boy: Complete Technical Reference](https://gekkio.fi/files/gb-docs/gbctr.pdf)
* [Pan Docs](https://gbdev.io/pandocs/)
* [Mooneye GB](https://github.com/Gekkio/mooneye-gb)
* [Educational Gameboy Emulator in Rust](https://github.com/rylev/DMG-01)
* [Opcode Table](https://izik1.github.io/gbops/)
* [Blargg's Gameboy hardware test ROMs](https://github.com/retrio/gb-test-roms)
