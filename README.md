<p align="center">
  <img src="assets/img/logo-3x.avif" />
</p>

**Play Kid** is yet another Game Boy emulator, written in Rust. But hey, it is MY Game Boy emulator, and I'm proud of it.

Here are the main features of Play Kid:

- All CPU instructions implemented.
- Full memory map implemented.
- Modes: ROM, MBC1, MBC2, MBC3.
- Audio is implemented, with 4 channels, envelopes, sweep, and stereo.
- Supports game controllers via SDL.
- Respects 160:144 aspect ratio by letter-boxing.
- Debug mode:
  - Step instruction.
  - Step scanline.
  - Pause/continue current execution.
  - FPS counter.
  - Displays internal state.
  - Breakpoints.
  - Uses own minimal UI library with horizontal/vertical layouts, labels, buttons, and text fields.
- Save RAM to `.sav` files to emulate the battery-backed SRAM.
- Automatically adapts to multi-DPI setups by scaling the UI.
- Working games/roms:
  - Passes `dmg-acid2`
  - Tetris
  - Pokémon
  - Super Mario Land
  - Super Mario Land 2: 6 Golden Coins
  - Dr. Mario
  - Probably many more

# Run

The usual Rust stuff.

```
  cargo run -- [ROM_FILE]
```

Make the binary with:

```
  cargo build --release
```

# Operation

Here are the keyboard mappings:

- <kbd>enter</kbd> - Start button
- <kbd>space</kbd> - Select button
- <kbd>a</kbd> - A button
- <kbd>b</kbd> - B button

Additionally, there are some more actions available:

- <kbd>f</kbd> - toggle FPS monitor
- <kbd>d</kbd> - enter debug mode
- <kbd>Esc</kbd> - exit the emulator

# Debug mode

You can enter the debug mode any time by pressing `d`, or activate it at launch with the `-d`/`--debug` flag.

![Debug mode](assets/img/debug-mode.avif)

You can use the provided UI controls to work with debug mode. You can also use the keyboard. These are the key bindings:

- <kbd>F6</kbd> - step a single instruction
- <kbd>F7</kbd> - step a scanline
- <kbd>F9</kbd> - continue execution until breakpoint (if paused), or pause execution (if running)
- <kbd>r</kbd> - reset the CPU
- <kbd>d</kbd> - exit debug mode and go back to normal full-speed emulation
- <kbd>Esc</kbd> - exit the emulator

You can also use breakpoints. A list with the current breakpoint addresses is provided in yellow. To create a breakpoint, enter the desired address (in `$abcd` format) into the text field and click <kbd>Add BR</kbd>. Remove a breakpoint with <kbd>Remove BR</kbd>. Clear all current breakpoints with <kbd>Clear all</kbd>.

# CLI args

There are some CLI arguments that you can use:

```
Play Kid 0.1.0

Minimalist Game Boy emulator for the cool kids.

Usage: playkid [OPTIONS] <INPUT>

Arguments:
  <INPUT>  Path to the input ROM file to load

Options:
  -s, --scale <SCALE>  Initial window scale. It can also be resized manually [default: 4]
  -d, --debug          Activate debug mode. Use `d` to stop program at any point
  -f, --fps            Show FPS counter. Use `f` to toggle on and off
      --skipcheck      Skip global checksum, header checksum, and logo sequence check
  -h, --help           Print help
  -V, --version        Print version
```

# Useful links

- Pandocs: https://gbdev.io/pandocs/
- Complete tech reference: https://gekkio.fi/files/gb-docs/gbctr.pdf
- Game Boy CPU manual: http://marc.rawer.de/Gameboy/Docs/GBCPUman.pdf
- Game Boy instruction set: https://www.pastraiser.com/cpu/gameboy/gameboy_opcodes.html
