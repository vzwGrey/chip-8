# CHIP-8

A [CHIP-8](https://www.rust-lang.org/) emulator written in Rust.

![Screenshot of the emulator playing a breakout clone](./screenshot.png)

## Building & Usage

To compile the project [Rust](https://www.rust-lang.org/) is required.
Run the following command in the project directory to generate the executable:
```
$ cargo build --release
```

The resulting executable can be found under `target/release/chip8` in the project
directory.

To run a game, invoke the executable with a path to a ROM file like so:
```
$ chip8 br8kout.ch8
```

See the [CHIP-8 archive](https://johnearnest.github.io/chip8Archive/) for a 
collection of modern CHIP-8 games to play.

## Features

- [x] Crossplatform (Windows, Linux, MacOS)
- [x] All canonical CHIP-8 instructions (apart from calls to native code)
- [x] Graphics
- [x] Keyboard input (see [below](##Keybindings))
- [ ] Sound

## Keybindings

The following is the original CHIP-8 keyboard layout:

|   |   |   |   |
|:-:|:-:|:-:|:-:|
| 1 | 2 | 3 | C |
| 4 | 5 | 6 | D |
| 7 | 8 | 9 | E |
| A | 0 | B | F |

This emulator maps those keys onto the following for modern (QWERTY) keyboards:

|   |   |   |   |
|:-:|:-:|:-:|:-:|
| 1 | 2 | 3 | 4 |
| Q | W | E | R |
| A | S | D | F |
| Z | X | C | V |

## License

This project is licensed under the [MIT License](./LICENSE).
