<p align="center">
  <img src="assets/img/logo-3x.avif" />
</p>

**Play Kid** is yet another Game Boy emulator, written in Rust. But hey, it is MY Game Boy emulator, and I'm proud of it. Here are the implemented features:

- All CPU instructions and full memory map.
- ROM, MBC1, MBC2, MBC3.
- Audio, with 4 channels, envelopes, sweep, and stereo.
- Respects 160:144 aspect ratio by letter-boxing.
- Save RAM to `.sav` files to emulate the battery-backed SRAM.
- Passes the `dmg-acid2` test.
- Tested games:
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

# CLI args

There are some CLI arguments that you can use:

```
Play Kid 1.0

Minimalist Game Boy emulator for the cool kids.

Usage: playkid [OPTIONS] <INPUT>

Arguments:
  <INPUT>  Path to the input ROM file to load

Options:
  -s, --scale <SCALE>  Initial window scale. It can also be resized manually [default: 3]
  -d, --debug          Activate debug mode
  -f, --fps            Show FPS counter
      --skipcheck      Skip global checksum, header checksum, and logo sequence check
  -h, --help           Print help
  -V, --version        Print version
```

# FPS

You can print the current FPS at any time by pressing <kbd>f</kbd>.

# Debug mode

Running with `-d` enables debug mode. In this mode, the game steps through the instructions one-by-one, unless continue (<kbd>c</kbd>) is hit in the terminal. The terminal outputs the state of the machine after each instruction:

```
$006f:     JR NZ, s8   0x006b
T-cycles:  2480
Reg:       AF: 05 50
           BC: 00 13
           DE: 00 d8
           HL: 01 4d
Flags:     _ N _ C
SP:        0xfffc
(i)DIV:    0xb36c/0xb3
Next b/w:  0xfa / 0xf0fa
LCDC:      0x80
STAT:      0x00
LYC:       0x00
LY:        0x05
LX:        0x00
Opcode:    0x20
Joypad:    _ _ _ _ _ _ _ _


===========
(enter)         step
(c)             continue
(b $ADDR)       add breakpoint to $ADDR
(b | b list)    list breakpoints
(b del)         delete all breakpoints
(b del $ADDR)   delete breakpoint
(r)             reset emulator
(q)             quit
> 
```

Here are the key bindings and commands:

- <kbd>Enter</kbd> -- step to next instruction
- `c` -- continue (until breakpoint)
- `b $ADDR` -- set a breakpoint at the given address. Example: `b $006a`
- `b list` -- list current breakpoints
- `b del $ADDR` -- delete breakpoint from given address `$ADDR`, if it exists
- `r` -- reset emulator
- `q` -- quit

Even if you did not start the emulator with the debug flag `-d`, you can always press <kbd>d</kbd> in the emulator window to stop it at the current instruction and enter debug mode. Operation carries on in the terminal window in the usual way.

# Useful links

- Pandocs: https://gbdev.io/pandocs/
- Complete tech reference: https://gekkio.fi/files/gb-docs/gbctr.pdf
- Game Boy CPU manual: http://marc.rawer.de/Gameboy/Docs/GBCPUman.pdf
- Game Boy instruction set: https://www.pastraiser.com/cpu/gameboy/gameboy_opcodes.html
