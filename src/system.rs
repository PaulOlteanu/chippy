use rand::Rng;

use minifb::{Key, Scale, Window, WindowOptions};

const WIDTH: usize = 64;
const HEIGHT: usize = 32;

#[rustfmt::skip]
const SPRITES: [u8; 16 * 5] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0,
    0x20, 0x60, 0x20, 0x20, 0x70,
    0xF0, 0x10, 0xF0, 0x80, 0xF0,
    0xF0, 0x10, 0xF0, 0x10, 0xF0,
    0x90, 0x90, 0xF0, 0x10, 0x10,
    0xF0, 0x80, 0xF0, 0x10, 0xF0,
    0xF0, 0x80, 0xF0, 0x90, 0xF0,
    0xF0, 0x10, 0x20, 0x40, 0x40,
    0xF0, 0x90, 0xF0, 0x90, 0xF0,
    0xF0, 0x90, 0xF0, 0x10, 0xF0,
    0xF0, 0x90, 0xF0, 0x90, 0x90,
    0xE0, 0x90, 0xE0, 0x90, 0xE0,
    0xF0, 0x80, 0x80, 0x80, 0xF0,
    0xE0, 0x90, 0x90, 0x90, 0xE0,
    0xF0, 0x80, 0xF0, 0x80, 0xF0,
    0xF0, 0x80, 0xF0, 0x80, 0x80,
];

fn display_value(v: bool) -> u32 {
    if v {
        0xFFFFFFFF
    } else {
        0
    }
}

fn key_to_index(key: minifb::Key) -> Option<usize> {
    match key {
        Key::Key1 => Some(1),
        Key::Key2 => Some(2),
        Key::Key3 => Some(3),
        Key::Key4 => Some(0xC),

        Key::Q => Some(4),
        Key::W => Some(5),
        Key::E => Some(6),
        Key::R => Some(0xD),

        Key::A => Some(7),
        Key::S => Some(8),
        Key::D => Some(9),
        Key::F => Some(0xE),

        Key::Z => Some(0xA),
        Key::X => Some(0),
        Key::C => Some(0xB),
        Key::V => Some(0xF),

        _ => None,
    }
}

#[derive(Debug)]
pub struct System {
    mem: [u8; 4096],
    reg: [u8; 16],
    i: u16,
    pc: u16,
    sp: u8,
    stack: [u16; 16],
    delay: u8,
    sound: u8,
    display: [[bool; 64]; 32],
    buttons: [bool; 16],
    waiting_for_key: bool,
    insert_key_at: u8,
}

impl System {
    pub fn new(program: &[u8; 3584]) -> Self {
        let mut mem = [0; 4096];

        mem[..16 * 5].copy_from_slice(&SPRITES);
        mem[0x200..].copy_from_slice(program);

        Self {
            mem,
            reg: [0; 16],
            i: 0,
            pc: 0x200,
            sp: 0,
            stack: [0; 16],
            delay: 0,
            sound: 0,
            display: [[false; 64]; 32],
            buttons: [false; 16],
            waiting_for_key: false,
            insert_key_at: 0,
        }
    }

    pub fn run(mut self) {
        let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];

        let mut window = Window::new(
            "Chippy",
            WIDTH,
            HEIGHT,
            WindowOptions {
                scale: Scale::X8,
                ..WindowOptions::default()
            },
        )
        .unwrap();

        self.buttons = [false; 16];

        window.limit_update_rate(Some(std::time::Duration::from_millis(1000 / 500)));

        while window.is_open() && !window.is_key_down(Key::Escape) {
            if self.waiting_for_key {
                let keys: Vec<usize> = window
                    .get_keys_pressed(minifb::KeyRepeat::Yes)
                    .iter()
                    .filter_map(|k| key_to_index(*k))
                    .collect();

                if !keys.is_empty() {
                    self.reg[self.insert_key_at as usize] = keys[0] as u8;
                    self.waiting_for_key = false;
                }

                break;
            }

            if self.delay != 0 {
                self.delay -= 1;
            }

            if self.sound != 0 {
                self.sound -= 1;
            }

            window
                .get_keys_pressed(minifb::KeyRepeat::Yes)
                .iter()
                .filter_map(|k| key_to_index(*k))
                .for_each(|k| self.buttons[k] = true);

            window
                .get_keys_released()
                .iter()
                .filter_map(|k| key_to_index(*k))
                .for_each(|k| self.buttons[k] = false);

            self.tick();

            for row in 0..32 {
                for col in 0..64 {
                    buffer[row * 64 + col] = display_value(self.display[row][col]);
                }
            }

            window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
        }
    }

    fn tick(&mut self) {
        let op: u16 =
            ((self.mem[self.pc as usize] as u16) << 8) + (self.mem[self.pc as usize + 1] as u16);

        let x = ((op & 0x0F00) >> 8) as u8;
        let y = ((op & 0x00F0) >> 4) as u8;

        let vx = self.reg[x as usize];
        let vy = self.reg[y as usize];

        let nnn = op & 0x0FFF;
        let kk = (op & 0x00FF) as u8;
        let n = (op & 0x000F) as u8;

        match op >> 12 {
            0x0 => match op & 0x00FF {
                0xE0 => {
                    for row in self.display.iter_mut() {
                        for col in row.iter_mut() {
                            *col = false;
                        }
                    }
                }

                0xEE => {
                    self.sp -= 1;
                    self.pc = self.stack[self.sp as usize] + 2;
                    return;
                }

                _ => println!("Unknown 0x0___ op: {:#06X}", op),
            },

            0x1 => {
                self.pc = nnn;
                return;
            }

            0x2 => {
                self.stack[self.sp as usize] = self.pc;
                self.sp += 1;
                self.pc = nnn;
                return;
            }

            0x3 => {
                if vx == kk {
                    self.pc += 2;
                }
            }

            0x4 => {
                if vx != kk {
                    self.pc += 2;
                }
            }

            0x5 => {
                if vx == vy {
                    self.pc += 2;
                }
            }

            0x6 => {
                self.reg[x as usize] = kk;
            }

            0x7 => {
                self.reg[x as usize] = vx.wrapping_add(kk);
            }

            0x8 => match op & 0xF {
                0x0 => self.reg[x as usize] = vy,

                0x1 => self.reg[x as usize] |= vy,

                0x2 => self.reg[x as usize] &= vy,

                0x3 => self.reg[x as usize] ^= vy,

                0x4 => {
                    let (result, overflow) = vx.overflowing_add(vy);

                    self.reg[x as usize] = result;
                    self.reg[0xF] = overflow as u8;
                }

                0x5 => {
                    let (result, overflow) = vx.overflowing_sub(vy);

                    self.reg[x as usize] = result;
                    self.reg[0xF] = !overflow as u8;
                }

                0x6 => {
                    self.reg[0xF] = vx & 1;
                    self.reg[x as usize] = vx >> 1;
                }

                0x7 => {
                    let (result, overflow) = vy.overflowing_sub(vx);

                    self.reg[x as usize] = result;
                    self.reg[0xF] = !overflow as u8;
                }

                0xE => {
                    self.reg[0xF] = (vx >> 7) & 1;
                    self.reg[x as usize] = vx << 1;
                }

                _ => println!("Unknown 0x8___ op: {:#06X}", op),
            },

            0x9 => {
                if vx != vy {
                    self.pc += 2;
                }
            }

            0xA => {
                self.i = nnn;
            }

            0xB => {
                self.pc = nnn + self.reg[0] as u16;
                return;
            }

            0xC => {
                let mut rng = rand::thread_rng();
                let num = rng.gen_range(0..=255);

                self.reg[x as usize] = num & kk;
            }

            0xD => {
                self.reg[0xF] = 0;

                // Y iterates over the byte
                for y in 0..n {
                    // X iterates over the bit of the byte
                    for x in 0..8 {
                        let display_y = vy.wrapping_add(y) as usize;
                        let display_x = vx.wrapping_add(x) as usize;

                        let write = (self.mem[self.i as usize + y as usize] >> (7 - x)) & 1;

                        if self.display[display_y % 32][display_x % 64] && write == 1 {
                            self.reg[0xF] = 1;
                        }

                        self.display[display_y % 32][display_x % 64] ^= write != 0;
                    }
                }
            }

            0xE => match op & 0x00FF {
                0x9E => {
                    if self.buttons[vx as usize] {
                        self.pc += 2;
                    }
                }

                0xA1 => {
                    if !self.buttons[vx as usize] {
                        self.pc += 2;
                    }
                }

                _ => println!("Unknown 0xE___ op: {:#06X}", op),
            },

            0xF => match op & 0x00FF {
                0x07 => self.reg[x as usize] = self.delay,

                0x0A => {
                    self.waiting_for_key = true;
                    self.insert_key_at = x;
                }

                0x15 => self.delay = vx,

                0x18 => self.sound = vx,

                0x1E => {
                    let (result, overflow) = self.i.overflowing_add(vx as u16);
                    self.i = result;
                    self.reg[0xF] = overflow as u8;
                }

                0x29 => {
                    self.i = vx as u16 * 5;
                }

                0x33 => {
                    self.mem[self.i as usize] = vx / 100;
                    self.mem[self.i as usize + 1] = (vx % 100) / 10;
                    self.mem[self.i as usize + 2] = vx % 10;
                }

                0x55 => {
                    self.mem[(self.i as usize)..=(self.i as usize + x as usize)]
                        .copy_from_slice(&self.reg[..=(x as usize)]);
                }

                0x65 => {
                    self.reg[..=(x as usize)].copy_from_slice(
                        &self.mem[(self.i as usize)..=(self.i as usize + x as usize)],
                    );
                }

                _ => println!("Unknown 0xF___ op: {:#06X}", op),
            },

            _ => println!("Unknown op: {:#06X}", op),
        }

        self.pc += 2;
    }
}
