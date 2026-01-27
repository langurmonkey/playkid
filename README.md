<p align="center">
  <img src="img/logo-3x.avif" />
</p>

**Play Kid** is yet another Game Boy emulator, written in Rust. But hey, it is MY Game Boy emulator, and I'm proud of it.


<p align="center">
  <img src="img/grid.avif" />
</p>

Here are the main features of Play Kid:

- All CPU instructions implemented.
- Full memory map implemented.
- Modes: ROM, MBC1, MBC2, MBC3.
- Audio is implemented, with 4 channels, envelopes, sweep, and stereo.
- Supports game controllers.
- Multiple color palettes.
- Save screenshot of current frame buffer.
- FPS counter.
- Respects 160:144 aspect ratio by letter-boxing.
- Debug panel:
  - Step instruction.
  - Step scanline.
  - Pause/continue current execution.
  - Displays internal state of CPU, PPU, and Joypad.
  - Full program disassembly, with breakpoints.
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
  - Probably many, many more
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

The keyboard is clumsy for playing Game Boy games, so you can use any game controller. Controllers are detected when hot-plugged. Use <kbd>LB</kbd> and <kbd>RB</kbd> to cycle through different palettes.

Additionally, there are some more actions available:

- <kbd>p</kbd> - cycle the palette colors
- <kbd>w</kbd> - trigger the SRAM save operation to `.sav` file.
- <kbd>s</kbd> - save a screenshot, with name `screenshot_[time].jpg`
- <kbd>d</kbd> - toggle debug panel
- <kbd>f</kbd> - toggle FPS monitor
- <kbd>r</kbd> - reset the CPU
- <kbd>Esc</kbd> - exit the emulator

You can also use the provided UI.

# Debug panel

You can open the debug panel any time by pressing <kbd>d</kbd>, by clikcing on `Machine` > `Debug panel...`, or activate it at launch with the `-d`/`--debug` flag. The debug panel shows up to the right. It provides a view of the internal state of the emulator, with:

- Current address, instruction, operands, and opcode, to the top.
- Internal state of CPU, PPU, and JOYP, to the left.
- Disassembly of the program, to the right.
- Breakpoints.

<p align="center">
  <img src="img/debug-mode-3.avif" />
</p>

You can use the provided UI controls to work with debug mode. You can also use the keyboard. These are the key bindings:

- <kbd>F6</kbd> - step a single instruction
- <kbd>F7</kbd> - step a scanline
- <kbd>F9</kbd> - continue execution until breakpoint (if paused), or pause execution (if running)
- <kbd>d</kbd> - exit debug mode and go back to normal full-speed emulation

You can also use breakpoints. A list with the current breakpoint addresses is provided at the bottom. To create a breakpoint, either **click on the address** in the disassembly panel, or enter it (in `$abcd` format) into the text field and click <kbd>+</kbd>. Remove a breakpoint by clicking the <kbd>×</kbd> in the breakpoints list. Clear all current breakpoints with <kbd>Clear all</kbd>.

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

# SDL2 version

Play Kid started as an SDL2 application, but it was moved to a pure Rust tech stack using `winit`, `egui`, `rodio`, and `gilrs`. This makes it much easier to build for different targets (including WASM!). Additionally, the SDL2 version contains a minimalist homegrown UI library that I'm particularly proud about, but it can't hold a candle to `egui` in terms of functionality. It looks like this:

<p align="center">
  <img src="img/debug-mode-sdl2.avif" />
</p>

The SDL2 version is tagged `playkid-sdl2` ([playkid-sdl2@codeberg](https://codeberg.org/langurmonkey/playkid/src/tag/playkid-sdl2), [playkid-sdl2@github](https://github.com/langurmonkey/playkid/tree/playkid-sdl2)).

# Useful links

- Pandocs: https://gbdev.io/pandocs/
- Complete tech reference: https://gekkio.fi/files/gb-docs/gbctr.pdf
- Game Boy CPU manual: http://marc.rawer.de/Gameboy/Docs/GBCPUman.pdf
- Game Boy CPU instructions: https://meganesu.github.io/generate-gb-opcodes/
