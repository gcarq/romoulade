# Romoulade

Experimental Gameboy ([DMG-01](https://en.wikipedia.org/wiki/Game_Boy)) implementation in Rust.

### State

This emulator is not production ready and mainly a emulation exploration project.
At this point it is capable of processing the Original Gameboy [Bootstrap ROM](https://gbdev.gg8.se/wiki/articles/Gameboy_Bootstrap_ROM).

### Development

```sh
$ git clone https://github.com/gcarq/romoulade
$ emerge media-libs/libsdl2 # pacman -S sdl2; apt install libsdl2-dev
$ cd romouldate
$ cargo run -- <path_to_rom>
```

This repository is open to contributions.
The code should follow the Rust style guideline.

### Dependencies

* Rust
* SDL2-2.0

### Todos

 - Finish ROM banking
 - Implement Timer
 - Implement missing instructions
 - Implement missing instruction tests
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
