<p align="center">
  <img src="assets/img/logo-3x.avif" />
</p>

**Play Kid** is yet another Game Boy emulator, written in Rust. But hey, it is MY Game Boy emulator, and I'm proud of it.

<p align="center">
  <img src="assets/img/grid.avif" />
</p>

Here are the main features of Play Kid:

- All CPU instructions implemented.
- Full memory map implemented.
- Modes: ROM, MBC1, MBC2, MBC3.
- Audio is implemented, with 4 channels, envelopes, sweep, and stereo.
- Supports game controllers.
- Multiple color palettes.
- Save screenshot of current frame buffer.
- Respects 160:144 aspect ratio by letter-boxing.
- Debug mode:
  - Step instruction.
  - Step scanline.
  - Pause/continue current execution.
  - FPS counter.
  - Displays internal state.
  - Breakpoints.
- Save RAM to `.sav` files to emulate the battery-backed SRAM. Those are saved every minute.
- Automatically adapts to multi-DPI setups by scaling the UI.
- Working games/roms:
  - Passes `dmg-acid2`
  - Tetris
  - Pokémon
  - Super Mario Land
  - Super Mario Land 2: 6 Golden Coins
  - Wario Land (Super Mario Land 3)
  - Wario Land II
  - Bugs Bunny Crazy Castle
  - The Amazing Spider-Man
  - Dr. Mario
  - Probably many more
- Works on Linux, macOS, and Windows.

# Download

The easiest way to run Play Kid is getting the package for your operating system:

- [Codeberg releases](https://codeberg.org/langurmonkey/playkid/releases)
- [GitHub releases](https://github.com/langurmonkey/playkid/releases)

# Build

Build the project with `cargo build`.

# Run

The usual Rust stuff.

```bash
  cargo run -- [ROM_FILE]
```

Make the binary with:

```bash
  cargo build --release
```

# Operation

Here are the Joypad keyboard mappings:

- <kbd>enter</kbd> - Start button
- <kbd>space</kbd> - Select button
- <kbd>a</kbd> - A button
- <kbd>b</kbd> - B button

You can also use any game controller. The SDL2 usual mappings apply.

Additionally, there are some more actions available:

- <kbd>p</kbd> - change the palette colors
- <kbd>w</kbd> - trigger the SRAM save operation to `.sav` file.
- <kbd>f</kbd> - toggle FPS monitor
- <kbd>s</kbd> - save a screenshot, with name `screenshot_[time].jpg`
- <kbd>d</kbd> - enter debug mode
- <kbd>Esc</kbd> - exit the emulator

You can also use the provided UI.

# Debug mode

You can enter the debug mode any time by pressing `d`, by clikcing on `Debug` > `Debug panel...`, or activate it at launch with the `-d`/`--debug` flag.

<p align="center">
  <img src="assets/img/debug-mode.avif" />
</p>

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
- Game Boy CPU instructions: https://meganesu.github.io/generate-gb-opcodes/
