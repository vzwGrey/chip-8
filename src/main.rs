use core::panic;

use clap::Parser;

const WIDTH: usize = 64;
const HEIGHT: usize = 32;
const ROM_START_ADDR: usize = 0x200;
const CHAR_FONT_ADDR: usize = 0x0;

#[cfg(debug_assertions)]
macro_rules! debug_print {
    ($($tts:tt)*) => {
        print!($($tts)*);
    };
}

#[cfg(debug_assertions)]
macro_rules! debug_println {
    ($($tts:tt)*) => {
        println!($($tts)*);
    };
}

#[cfg(not(debug_assertions))]
macro_rules! debug_print {
    ($($tts:tt)*) => {};
}

#[cfg(not(debug_assertions))]
macro_rules! debug_println {
    ($($tts:tt)*) => {};
}

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// ROM to load and play in the emulator.
    rom: std::path::PathBuf,
}

trait IOManager {
    fn read(&self, addr: u16) -> u8;
    fn write(&mut self, addr: u16, data: u8);
    fn clear_display(&mut self);
    fn draw(&mut self, x: u8, y: u8, n: u8, idx: u16) -> bool;
    fn get_framebuffer(&self) -> &[u32];
    fn get_key(&self) -> Option<u8>;
}

#[derive(Debug)]
struct Cpu {
    rng: rand::prelude::ThreadRng,
    v: [u8; 16],
    idx: u16,
    sp: u16,
    pc: u16,
    delay: u8,
    sound: u8,
    cycle: u8,
}

impl Cpu {
    fn new() -> Self {
        Self {
            rng: rand::thread_rng(),
            v: [0; 16],
            idx: 0,
            sp: 0xEFF,
            pc: ROM_START_ADDR as u16,
            delay: 0,
            sound: 0,
            cycle: 59,
        }
    }

    fn step<IO: IOManager>(&mut self, io: &mut IO) {
        self.cycle -= 1;
        if self.cycle == 0 {
            self.delay = self.delay.saturating_sub(1);
            self.sound = self.sound.saturating_sub(1);
            self.cycle = 59;
        }

        let op = self.fetch(io);
        debug_print!("${:04X}:\t{:04X}\t", self.pc - 2, op);

        let o0 = op & 0xF;
        let o1 = (op >> 4) & 0xF;
        let o2 = (op >> 8) & 0xF;
        let o3 = (op >> 12) & 0xF;

        match (o3, o2, o1, o0) {
            // Clear display
            (0, 0, 0xE, 0) => {
                debug_println!("CLEAR");
                io.clear_display();
            }
            // Return
            (0, 0, 0xE, 0xE) => {
                debug_println!("RETURN");
                self.pc = self.pop(io);
            }
            // Call machine code
            (0, _, _, _) => {
                panic!(
                    "Call to machine code routine is not implemented. (PC=${:04X})",
                    self.pc - 2
                );
            }
            // GOTO n
            (1, n2, n1, n0) => {
                let n = (n2 << 8) | (n1 << 4) | n0;
                debug_println!("GOTO {:03X}", n);
                self.pc = n;
            }
            // Call nnn
            (2, n2, n1, n0) => {
                let n = (n2 << 8) | (n1 << 4) | n0;
                debug_println!("CALL {:03X}", n);
                self.push(io, self.pc);
                self.pc = n;
            }
            // if (Vx == n)
            (3, x, n1, n0) => {
                let n = (n1 << 4) | n0;
                debug_println!("if (V{:X} == {:X})", x, n);
                if self.v[x as usize] == (n as u8) {
                    self.advance();
                }
            }
            // if (Vx != n)
            (4, x, n1, n0) => {
                let n = (n1 << 4) | n0;
                debug_println!("if (V{:X} != {:X})", x, n);
                if self.v[x as usize] != (n as u8) {
                    self.advance();
                }
            }
            // if (Vx == Vy)
            (5, x, y, 0) => {
                debug_println!("if (V{:X} == V{:X})", x, y);
                if self.v[x as usize] == self.v[y as usize] {
                    self.advance();
                }
            }
            // Vx = n
            (6, x, n1, n0) => {
                let n = (n1 << 4) | n0;
                debug_println!("V{:X} == {:02X}", x, n);
                self.v[x as usize] = n as u8;
            }
            // Vx += n
            (7, x, n1, n0) => {
                let n = (n1 << 4) | n0;
                debug_println!("V{:X} += {:X}", x, n);
                let x = x as usize;
                self.v[x] = self.v[x].wrapping_add(n as u8);
            }
            // Vx = Vy
            (8, x, y, 0) => {
                debug_println!("V{:X} = V{:X}", x, y);
                self.v[x as usize] = self.v[y as usize];
            }
            // Vx |= Vy
            (8, x, y, 1) => {
                debug_println!("V{:X} |= V{:X}", x, y);
                self.v[x as usize] |= self.v[y as usize];
            }
            // Vx &= Vy
            (8, x, y, 2) => {
                debug_println!("V{:X} &= V{:X}", x, y);
                self.v[x as usize] &= self.v[y as usize];
            }
            // Vx ^= Vy
            (8, x, y, 3) => {
                debug_println!("V{:X} ^= V{:X}", x, y);
                let x = x as usize;
                let y = y as usize;
                self.v[x] ^= self.v[y];
            }
            // Vx += Vy
            (8, x, y, 4) => {
                debug_println!("V{:X} += V{:X}", x, y);
                let (res, carry) = self.v[x as usize].overflowing_add(self.v[y as usize]);
                self.v[x as usize] = res;
                self.v[0xF] = if carry { 1 } else { 0 };
            }
            // Vx -= Vy
            (8, x, y, 5) => {
                debug_println!("V{:X} += V{:X}", x, y);
                let (res, carry) = self.v[x as usize].overflowing_sub(self.v[y as usize]);
                self.v[x as usize] = res;
                self.v[0xF] = if carry { 0 } else { 1 };
            }
            // Vx >>= 1
            (8, x, _, 6) => {
                debug_println!("V{:X} >>= 1", x);
                self.v[0xF] = self.v[x as usize] & 1;
                self.v[x as usize] >>= 1;
            }
            // Vx -= Vy
            (8, x, y, 7) => {
                debug_println!("V{:X} -= V{:X}", x, y);
                let (res, carry) = self.v[y as usize].overflowing_sub(self.v[x as usize]);
                self.v[x as usize] = res;
                self.v[0xF] = if carry { 0 } else { 1 };
            }
            // Vx <<= 1
            (8, x, _, 0xE) => {
                debug_println!("V{:X} <<= 1", x);
                self.v[0xF] = (self.v[x as usize] >> 7) & 1;
                self.v[x as usize] <<= 1;
            }
            // if (Vx != Vy)
            (9, x, y, 0) => {
                debug_println!("if (V{:X} != V{:X})", x, y);
                if self.v[x as usize] != self.v[y as usize] {
                    self.advance();
                }
            }
            // Idx = nnn
            (0xA, n2, n1, n0) => {
                let n = (n2 << 8) | (n1 << 4) | n0;
                debug_println!("Idx = {:03X}", n);
                self.idx = n;
            }
            // PC = V0 + n
            (0xB, n2, n1, n0) => {
                let n = (n2 << 8) | (n1 << 4) | n0;
                debug_println!("PC = V0 + {:03X}", n);
                self.pc = (self.v[0] as u16) + n;
            }
            // Vx = rand() & n
            (0xC, x, n1, n0) => {
                use rand::Rng;
                let n = (n1 << 4) | n0;
                debug_println!("V{:X} = rand() & {:X}", x, n);
                self.v[x as usize] = self.rng.gen::<u8>() & (n as u8);
            }
            // Draw(Vx, Vy, n)
            (0xD, x, y, n) => {
                debug_println!("DRAW(V{:X}, V{:X}, {:X})", x, y, n);
                let collision = io.draw(self.v[x as usize], self.v[y as usize], n as u8, self.idx);
                self.v[0xF] = if collision { 1 } else { 0 };
            }
            // if (Key() == Vx)
            (0xE, x, 9, 0xE) => {
                debug_println!("if (Key() == V{:X}", x);
                if io.get_key() == Some(self.v[x as usize]) {
                    self.advance();
                }
            }
            // if (Key() != Vx)
            (0xE, x, 0xA, 1) => {
                debug_println!("if (Key() != V{:X}", x);
                if io.get_key() != Some(self.v[x as usize]) {
                    self.advance();
                }
            }
            // Vx = GetDelay()
            (0xF, x, 0, 7) => {
                debug_println!("V{:X} = GetDelay()", x);
                self.v[x as usize] = self.delay;
            }
            // SetDelay(Vx)
            (0xF, x, 1, 5) => {
                debug_println!("SetDelay(V{:X})", x);
                self.delay = self.v[x as usize];
            }
            // SetSound(Vx)
            (0xF, x, 1, 8) => {
                debug_println!("SetSound(V{:X})", x);
                self.sound = self.v[x as usize];
            }
            // Idx += Vx
            (0xF, x, 1, 0xE) => {
                debug_println!("Idx += V{:X}", x);
                self.idx = self.idx.wrapping_add(self.v[x as usize] as u16);
            }
            // Idx = SpriteAddress(Vx)
            (0xF, x, 2, 9) => {
                debug_println!("Idx = SpriteAddress(V{:X})", x);
                self.idx = (CHAR_FONT_ADDR as u16) + (self.v[x as usize] * 5) as u16;
            }
            // StoreBCD(Vx)
            (0xF, x, 3, 3) => {
                debug_print!("StoreBCD(V{:X})", x);
                let mut val = self.v[x as usize];
                for i in 0..3 {
                    let digit = val % 10;
                    val /= 10;
                    io.write(self.idx + 2 - i, digit);
                }
            }
            // Register dump
            (0xF, x, 5, 5) => {
                debug_println!("RegDump(V0..V{:X})", x);
                for i in 0..=x {
                    io.write(self.idx + i, self.v[i as usize]);
                }
            }
            // Register load
            (0xF, x, 6, 5) => {
                debug_println!("RegLoad(V0..V{:X})", x);
                for i in 0..=x {
                    self.v[i as usize] = io.read(self.idx + i);
                }
            }
            _ => panic!(
                "Unsupported instruction ${:04X} (PC=${:04X})",
                op,
                self.pc - 2
            ),
        }
    }

    fn advance(&mut self) {
        self.pc += 2;
    }

    fn fetch<IO: IOManager>(&mut self, io: &IO) -> u16 {
        let hi = io.read(self.pc);
        let lo = io.read(self.pc + 1);
        self.advance();
        u16::from_be_bytes([hi, lo])
    }

    fn push<IO: IOManager>(&mut self, io: &mut IO, data: u16) {
        io.write(self.sp, (data & 0xFF) as u8);
        io.write(self.sp - 1, ((data >> 8) & 0xFF) as u8);
        self.sp -= 2;
    }

    fn pop<IO: IOManager>(&mut self, io: &IO) -> u16 {
        self.sp += 2;
        let lo = io.read(self.sp);
        let hi = io.read(self.sp - 1);
        u16::from_be_bytes([hi, lo])
    }
}

struct IO {
    frame_buffer: Vec<u32>,
    did_draw: bool,
    mem: Vec<u8>,
    key: Option<u8>,
}

impl IO {
    fn new(rom: &[u8]) -> Self {
        let mut mem = vec![0; 4 * 1024];

        let char_font = [
            // 0
            0b1111_0000,
            0b1001_0000,
            0b1001_0000,
            0b1001_0000,
            0b1111_0000,
            // 1
            0b0010_0000,
            0b0110_0000,
            0b0010_0000,
            0b0010_0000,
            0b0111_0000,
            // 2
            0b1111_0000,
            0b0001_0000,
            0b1111_0000,
            0b1000_0000,
            0b1111_0000,
            // 3
            0b1111_0000,
            0b0001_0000,
            0b1111_0000,
            0b0001_0000,
            0b1111_0000,
            // 4
            0b1001_0000,
            0b1001_0000,
            0b1111_0000,
            0b0001_0000,
            0b0001_0000,
            // 5
            0b1111_0000,
            0b1000_0000,
            0b1111_0000,
            0b0001_0000,
            0b1111_0000,
            // 6
            0b1111_0000,
            0b1000_0000,
            0b1111_0000,
            0b1001_0000,
            0b1111_0000,
            // 7
            0b1111_0000,
            0b0001_0000,
            0b0010_0000,
            0b0100_0000,
            0b0100_0000,
            // 8
            0b1111_0000,
            0b1001_0000,
            0b1111_0000,
            0b1001_0000,
            0b1111_0000,
            // 9
            0b1111_0000,
            0b1001_0000,
            0b1111_0000,
            0b0001_0000,
            0b1111_0000,
            // A
            0b1111_0000,
            0b1001_0000,
            0b1111_0000,
            0b1001_0000,
            0b1001_0000,
            // B
            0b1111_0000,
            0b1001_0000,
            0b1110_0000,
            0b1001_0000,
            0b1111_0000,
            // C
            0b1111_0000,
            0b1000_0000,
            0b1000_0000,
            0b1000_0000,
            0b1111_0000,
            // D
            0b1110_0000,
            0b1001_0000,
            0b1001_0000,
            0b1001_0000,
            0b1110_0000,
            // E
            0b1111_0000,
            0b1000_0000,
            0b1111_0000,
            0b1000_0000,
            0b1111_0000,
            // F
            0b1111_0000,
            0b1000_0000,
            0b1111_0000,
            0b1000_0000,
            0b1000_0000,
        ];

        mem[CHAR_FONT_ADDR..][..char_font.len()].copy_from_slice(&char_font);
        mem[ROM_START_ADDR..][..rom.len()].copy_from_slice(rom);

        Self {
            frame_buffer: vec![0; WIDTH * HEIGHT],
            did_draw: false,
            mem,
            key: None,
        }
    }
}

impl IOManager for IO {
    fn read(&self, addr: u16) -> u8 {
        self.mem[addr as usize]
    }

    fn write(&mut self, addr: u16, data: u8) {
        self.mem[addr as usize] = data;
    }

    fn clear_display(&mut self) {
        for p in &mut self.frame_buffer {
            *p = 0;
        }
    }

    fn draw(&mut self, x: u8, y: u8, n: u8, idx: u16) -> bool {
        self.did_draw = true;

        let x = x as usize;
        let y = y as usize;
        let n = n as usize;
        let idx = idx as usize;

        let mut collision = false;
        for (dy, row) in self.mem[idx..][..n].iter().enumerate() {
            for dx in 0..8 {
                let bit = (row >> (7 - dx)) & 1;
                let pixel = (bit as u32) * 0x00FF_FFFF;

                let pi = (x + dx) + (y + dy) * WIDTH;
                if pi >= self.frame_buffer.len() {
                    continue;
                }

                let old_pixel = self.frame_buffer[pi];
                let new_pixel = (self.frame_buffer[pi] ^ pixel) & 0x00FF_FFFF;
                self.frame_buffer[pi] = new_pixel;

                if old_pixel != 0 && new_pixel == 0 {
                    collision = true;
                }
            }
        }
        collision
    }

    fn get_framebuffer(&self) -> &[u32] {
        &self.frame_buffer
    }

    fn get_key(&self) -> Option<u8> {
        self.key
    }
}

impl IO {
    fn update_with_window(&mut self, win: &mut minifb::Window) -> eyre::Result<()> {
        use minifb::Key;
        let keys = [
            Key::X,    // #0
            Key::Key1, // #1
            Key::Key2, // #2
            Key::Key3, // #3
            Key::Q,    // #4
            Key::W,    // #5
            Key::E,    // #6
            Key::A,    // #7
            Key::S,    // #8
            Key::D,    // #9
            Key::Z,    // #A
            Key::X,    // #B
            Key::Key4, // #C
            Key::R,    // #D
            Key::F,    // #E
            Key::V,    // #F
        ];
        self.key = None;
        for (i, key) in keys.iter().enumerate() {
            if win.is_key_down(*key) {
                self.key = Some(i as u8);
            }
        }

        if self.did_draw {
            win.update_with_buffer(&self.frame_buffer, WIDTH, HEIGHT)?;
        }

        Ok(())
    }
}

fn main() -> eyre::Result<()> {
    let args = Args::parse();
    let rom = std::fs::read(args.rom)?;

    let mut io = IO::new(&rom);
    let mut cpu = Cpu::new();

    let win_options = minifb::WindowOptions {
        scale: minifb::Scale::X16,
        ..minifb::WindowOptions::default()
    };
    let mut win = minifb::Window::new("CHIP-8", WIDTH, HEIGHT, win_options)?;
    win.limit_update_rate(None);

    #[cfg(debug_assertions)]
    let mut i = 0;

    while win.is_open() && !win.is_key_down(minifb::Key::Escape) {
        debug_print!("{}\t", i);
        io.update_with_window(&mut win)?;
        cpu.step(&mut io);

        #[cfg(debug_assertions)]
        {
            i += 1
        }
    }

    Ok(())
}
