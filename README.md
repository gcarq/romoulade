# Romoulade

Experimental Gameboy Emulator implementation in Rust.

## State

This emulator is not production ready and mainly a emulation exploration project.
At this point it is capable of processing the Original
Gameboy [Bootstrap ROM](https://gbdev.gg8.se/wiki/articles/Gameboy_Bootstrap_ROM).

### Blargg's CPU test results

| Test No.              | Result | Remark |
|-----------------------|--------|--------|
| 01-special            | ✅      | Passed |
| 02-interrupts         | ✅      | Passed |
| 03-op sp,hl           | ✅      | Passed |
| 04-op r,imm           | ✅      | Passed |
| 05-op rp              | ✅      | Passed |
| 06-ld r,r             | ✅      | Passed |
| 07-jr,jp,call,ret,rst | ✅      | Passed |
| 08-misc instrs        | ✅      | Passed |
| 09-op r,r             | ✅      | Passed |
| 10-bit ops            | ✅      | Passed |
| 11-op a,(hl)          | ✅      | Passed |

## Debugger

Passing `--debug` starts a simple debugger,
this feature is WIP and might just not work.

![Debugger](https://i.imgur.com/c6XeizK.png)

## Usage

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

## Development

```sh
$ git clone https://github.com/gcarq/romoulade
$ emerge media-libs/libsdl2 # pacman -S sdl2; apt install libsdl2-dev
$ cd romoulade
$ cargo run -- <path_to_rom>
```

This repository is open to contributions.
The code should follow the Rust style guideline.

## Dependencies

* Rust
* SDL2-2.0

## TODOs

- ROM banking: Implement MBC2+
- Increase instruction test coverage
- Finish Pixel Processing Unit
- Implement Sound Processing Unit
- Pass [Test ROMs](https://gbdev.gg8.se/files/roms/blargg-gb-tests/)
- ...

## Resources

* [The Ultimate Game Boy Talk (33c3)](https://www.youtube.com/watch?v=HyzD8pNlpwI)
* [gbdev.gg8.se](https://gbdev.gg8.se/)
* [Educational Gameboy Emulator in Rust](https://github.com/rylev/DMG-01)
* [Opcode Table](https://izik1.github.io/gbops/)
* [Game Boy CPU Manual](http://marc.rawer.de/Gameboy/Docs/GBCPUman.pdf)
* [Game Boy: Complete Technical Reference](https://gekkio.fi/files/gb-docs/gbctr.pdf)
* [Blargg's Gameboy hardware test ROMs](https://github.com/retrio/gb-test-roms)
