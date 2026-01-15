<p align="center">
  <img src="img/logo.avif" width="200px" style="image-rendering: pixelated;"/>
</p>

# Play Kid, a Game Boy emulator

Play Kid is yet another Game Boy emulator, written in Rust. But hey, it is MY Game Boy emulator, and I'm proud of it. Here are the implemented features:

- All CPU instructions and full memory map.
- ROM, MBC1, MBC2, MBC3.
- Audio, with 4 channels, envelopes, sweep, and stereo.
- Respects 160:144 aspect ratio by letter-boxing.
- Save RAM to `.sav` files to emulate the battery-backed SRAM.
- Passes the `dmg-acid2` test.
- Plays:
  - Tetris
  - Pokémon
  - Super Mario Land
  - Super Mario Land 2: 6 Golden Coins
  - Probably many more
- Errors:
  - Dr. Mario -- for some reason hangs after the menu screen

# Useful links

Pandocs: https://gbdev.io/pandocs/
Complete tech reference: https://gekkio.fi/files/gb-docs/gbctr.pdf
Game Boy CPU manual: http://marc.rawer.de/Gameboy/Docs/GBCPUman.pdf
Game Boy instruction set: https://www.pastraiser.com/cpu/gameboy/gameboy_opcodes.html
