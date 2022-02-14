# CHIP-8

A [CHIP-8](https://www.rust-lang.org/) emulator written in Rust.

## Building & Usage

To compile the project [Rust](https://www.rust-lang.org/) is required.
Run the following command in the project directory to generate the executable:
```
$ cargo build --release
```

The resulting exectuable can be found under `target/release/chip8` in the project
directory.

To run a game, invoke the executable with a path to a ROM file like so:
```
$ chip8 br8kout.ch8
```

## Features

- [x] Crossplatform (Windows, Linux, MacOS)
- [x] All canonical CHIP-8 instructions (apart from calls to native code)
- [x] Graphics
- [x] Keyboard input (see [below](##Keybindings))
- [ ] Sound

## Keybindings

The following is the original CHIP-8:

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
| Z | X | C | Z |

## License

This project is licensed under the [MIT License](./LICENSE).
