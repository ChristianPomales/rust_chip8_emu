extern crate rand;

use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

#[cfg_attr(rustfmt, rustfmt_skip)]
const CHIP8_FONTSET: [u8; 80] = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, //0
    0x20, 0x60, 0x20, 0x20, 0x70, //1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, //2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, //3
    0x90, 0x90, 0xF0, 0x10, 0x10, //4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, //5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, //6
    0xF0, 0x10, 0x20, 0x40, 0x40, //7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, //8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, //9
    0xF0, 0x90, 0xF0, 0x90, 0x90, //A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, //B
    0xF0, 0x80, 0x80, 0x80, 0xF0, //C
    0xE0, 0x90, 0x90, 0x90, 0xE0, //D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, //E
    0xF0, 0x80, 0xF0, 0x80, 0x80  //F
];

pub struct Chip8 {
    stack: [u16; 16], // Stack
    sp: u16,          // Stack pointer

    memory: [u8; 4096], // Memory 4kB
    v: [u8; 16],        // V registers (V0-VF)

    pc: u16,         // Program counter
    opcode: u16,     // Current opcode
    i: u16,          // Index register
    delay_timer: u8, // Delay Timer
    sound_timer: u8, // Sound timer

    pub gfx: [u8; 64 * 32], // Graphics buffer
    pub key: [u8; 16],      // Keypad
    pub draw_flag: bool,    // Indicates a draw has occured
}

impl Chip8 {
    // Initalize
    pub fn new() -> Self {
        let mut chip = Chip8 {
            stack: [0; 16],
            sp: 0,

            memory: [0; 4096],
            v: [0; 16],

            pc: 0x200,
            opcode: 0,
            i: 0,
            delay_timer: 0,
            sound_timer: 0,

            gfx: [0; 64 * 32],
            key: [0; 16],
            draw_flag: false,
        };

        // place fonts in memory
        for (i, element) in CHIP8_FONTSET.iter().enumerate() {
            chip.memory[i] = element.to_owned();
        }

        return chip;
    }

    pub fn load(&mut self, game_name: &str) {
        let path = Path::new(game_name);
        let display = path.display();

        let mut file = match File::open(&path) {
            Err(why) => panic!("couldn't open {}: {}", display, why.description()),
            Ok(file) => file,
        };

        let mut rom = Vec::new();
        match file.read_to_end(&mut rom) {
            Err(why) => panic!("couldn't read {}: {}", display, why.description()),
            Ok(_) => {}
        }

        if rom.len() < (4096 - 512) {
            for (i, byte) in rom.iter().enumerate() {
                self.memory[i + 512] = byte.to_owned();
            }
        } else {
            println!("Rom too big to fit in memory");
        }
    }

    pub fn emulate_cycle(&mut self) {
        let first_byte = (self.memory[self.pc as usize] as u16) << 8;
        let second_byte = self.memory[self.pc as usize + 1] as u16;
        self.opcode = first_byte | second_byte;

        // print opcodes
        // println!("\n\n opcode -> {:x} \n", self.opcode);

        // // print registers
        // println!("---registers---");
        // for reg in self.v.iter() {
        //     print!("{:x}, ", reg);
        // }
        // print!("\n");
        // println!("---end registers---");

        // println!("i is {}", self.i);

        // println!("--- begin memory dump ---");
        // for byte in self.memory.iter() {
        //     print!("{:x}, ", byte);
        // }
        // println!("\n--- end memory dump ---");

        match self.opcode & 0xF000 {
            // 00E_
            0x0000 => match self.opcode & 0x000F {
                // 00E0 - Clear screen
                0x0000 => {
                    self.gfx = [0; 64 * 32];
                    self.draw_flag = true;
                    self.pc += 2;
                }
                //00EE - Return from subroutine
                0x000E => {
                    self.sp -= 1;
                    self.pc = self.stack[self.sp as usize];
                    self.pc += 2;
                }
                _ => println!("Unknown opcode"),
            },
            // 1NNN - Jumps to address NNN
            0x1000 => {
                self.pc = self.opcode & 0x0FFF;
            }
            // 2NNN - Calls subroutine at NNN
            0x2000 => {
                self.stack[self.sp as usize] = self.pc;
                self.sp += 1;
                self.pc = self.opcode & 0x0FFF;
            }
            // 3XNN - Skips the next instruction if VX = NN
            0x3000 => {
                let x = (self.opcode & 0x0F00) >> 8;
                let vx = self.v[x as usize] as u16;

                if vx == (self.opcode & 0x00FF) {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            }
            // 4XNN - Skips the next instruction if VX != NN
            0x4000 => {
                let x = (self.opcode & 0x0F00) >> 8;
                let vx = self.v[x as usize] as u16; // not sure if this cast is correct
                let nn = self.opcode & 0x00FF;

                if vx != nn {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            }
            // 5XY0 - Skips the next instruction if VX == VY
            0x5000 => {
                let x = (self.opcode & 0x0F00) >> 8;
                let vx = self.v[x as usize];
                let y = (self.opcode & 0x00F0) >> 4;
                let vy = self.v[y as usize];

                if vx == vy {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            }
            // 6XNN - Sets VX to NN
            0x6000 => {
                let x = (self.opcode & 0x0F00) >> 8;
                self.v[x as usize] = (self.opcode & 0x00FF) as u8;
                self.pc += 2;
            }
            // 7XNN - Adds NN to VX
            0x7000 => {
                let x = (self.opcode & 0x0F00) >> 8;
                self.v[x as usize] = self.v[x as usize].wrapping_add((self.opcode & 0x00FF) as u8);
                //self.v[x as usize] += (self.opcode & 0x00FF) as u8;
                self.pc += 2;
            }
            // 8XY_
            0x8000 => match self.opcode & 0x000F {
                // 8XY0 - Sets VX to the value of VY
                0x0000 => {
                    let x = (self.opcode & 0x0F00) >> 8;
                    let y = (self.opcode & 0x00F0) >> 4;
                    self.v[x as usize] = self.v[y as usize];
                    self.pc += 2;
                }
                // 8XY1 - Sets VX to (VX OR VY)
                0x0001 => {
                    let x = (self.opcode & 0x0F00) >> 8;
                    let y = (self.opcode & 0x00F0) >> 4;
                    self.v[x as usize] |= self.v[y as usize];
                    self.pc += 2;
                }
                // 8XY2 - Sets VX to (VX AND VY)
                0x0002 => {
                    let x = (self.opcode & 0x0F00) >> 8;
                    let y = (self.opcode & 0x00F0) >> 4;
                    self.v[x as usize] &= self.v[y as usize];
                    self.pc += 2;
                }
                // 8XY3 = Sets VX to (VX XOR VY)
                0x0003 => {
                    let x = (self.opcode & 0x0F00) >> 8;
                    let y = (self.opcode & 0x00F0) >> 4;
                    self.v[x as usize] ^= self.v[y as usize];
                    self.pc += 2;
                }
                /*
                    8XY4 - Adds VY to VX. VF is set to 1 when there is a carry,
                    and 0 when there isn't
                */
                0x0004 => {
                    let x = (self.opcode & 0x0F00) >> 8;
                    let y = (self.opcode & 0x00F0) >> 4;

                    let n = self.v[x as usize].wrapping_add(self.v[y as usize]);
                    self.v[x as usize] = n;

                    if self.v[y as usize] > self.v[x as usize] {
                        self.v[0xF] = 1 // carry
                    } else {
                        self.v[0xF] = 0 // no carry
                    }

                    self.pc += 2;
                }
                /*
                    8XY5 - VY is subtracted from VX. VF is set to 0 when there is a borrow,
                    and 1 when there isn't
                */
                0x0005 => {
                    let x = (self.opcode & 0x0F00) >> 8;
                    let y = (self.opcode & 0x00F0) >> 4;

                    if self.v[y as usize] > self.v[x as usize] {
                        self.v[0xF] = 0 // borrow
                    } else {
                        self.v[0xF] = 1 // no borrow
                    }

                    let n = self.v[x as usize].wrapping_sub(self.v[y as usize]);
                    self.v[x as usize] = n;

                    self.pc += 2;
                }
                /*
                    8XY6 - Shifts VX right by one. VF is set to the value of
                    the least significant bit of VX before the shift
                */
                0x0006 => {
                    let x = (self.opcode & 0x0F00) >> 8;
                    self.v[0xF] = self.v[x as usize] & 0x1;
                    self.v[x as usize] >>= 1;

                    self.pc += 2;
                }
                /*
                    8XY7 - Sets VX to VY minus VX. VF is set to 0 when there's
                    a borrow, and 1 when there isn't.
                */
                0x0007 => {
                    let x = (self.opcode & 0x0F00) >> 8;
                    let y = (self.opcode & 0x00F0) >> 4;

                    if self.v[x as usize] > self.v[y as usize] {
                        self.v[0xF] = 0 // borrow
                    } else {
                        self.v[0xF] = 1 // no borrow
                    }

                    self.v[x as usize] = self.v[y as usize] - self.v[x as usize];

                    self.pc += 2;
                }
                /*
                    8XYE - Shifts VX left by one. VF is set to the value of the
                    most significant bit of VX before the shift.
                */
                0x000E => {
                    let x = (self.opcode & 0x0F00) >> 8;
                    self.v[0xF] = self.v[x as usize] >> 7;
                    self.v[x as usize] <<= 1;

                    self.pc += 2;
                }
                _ => println!("Unknown opcode"),
            },
            // 9XY0 - Skips the next instruction if VX != VY
            0x9000 => {
                let x = (self.opcode & 0x0F00) >> 8;
                let vx = self.v[x as usize];
                let y = (self.opcode & 0x00F0) >> 4;
                let vy = self.v[y as usize];

                if vx != vy {
                    self.pc += 4;
                } else {
                    self.pc += 2;
                }
            }
            // ANNN - Sets i to address NNN
            0xA000 => {
                self.i = self.opcode & 0x0FFF;
                self.pc += 2;
            }
            // BNNN - Jumps to the address NNN + V0
            0xB000 => {
                self.pc = (self.opcode & 0x0FFF) + self.v[0] as u16;
            }
            // CXNN - Sets VX to a random number, masked by NN
            0xC000 => {
                // pretty sure i can just gen a random u8 instead of doing the masking
                let rn: u8 = rand::random();
                //let mod_number = (0xFF as u16).wrapping_add(1);
                let mut masked_rn: u8 = (rn) & (self.opcode & 0x0FF) as u8;
                let vx_index = ((self.opcode & 0x0F00) >> 8) as usize;
                self.v[vx_index] = masked_rn;
                self.pc += 2;
            }
            /* 
                    DXYN: Draws a sprite at coordinate (VX, VY) that has a width of 8 pixels
                    and a height of N pixels.
                    Each row of 8 pixels is read as bit-coded starting from memory location i.
                    I value doesn't change after the execution of this instruction.
                    VF is set to 1 if any screen pixels are flipped from set to unset when
                    the sprire is drawn, and to 0 if that doesn't happen.
                */
            0xD000 => {
                let vx_index = ((self.opcode & 0x0F00) >> 8) as usize;
                let x = self.v[vx_index] as u16;
                let vy_index = ((self.opcode & 0x00F0) >> 4) as usize;
                let y = self.v[vy_index] as u16;
                let height = self.opcode & 0x000F;
                let mut pixel: u16;

                self.v[0xF] = 0;
                for yline in 0..height {
                    pixel = self.memory[(self.i + yline) as usize] as u16;
                    for xline in 0..8 {
                        if (pixel & (0x80 >> xline)) != 0 {
                            let index = (x + xline + ((y + yline) * 64)) % 2048;
                            if self.gfx[index as usize] == 1 {
                                self.v[0xF] = 1;
                            }
                            self.gfx[index as usize] ^= 1;
                        }
                    }
                }

                self.draw_flag = true;
                self.pc += 2;
            }
            // EX__
            0xE000 => match self.opcode & 0x00FF {
                // EX9E - Skips the next instruction if the key stored in VX is pressed
                0x009E => {
                    let x = (self.opcode & 0x0F00) >> 8;
                    let vx = self.v[x as usize];
                    let key = self.key[vx as usize];

                    if key != 0 {
                        self.pc += 4;
                    } else {
                        self.pc += 2;
                    }
                }
                //EXA1 - Skips the next instructions if they key stored in VS isn't pressed
                0x00A1 => {
                    let x = (self.opcode & 0x0F00) >> 8;
                    let vx = self.v[x as usize];
                    let key = self.key[vx as usize];

                    if key == 0 {
                        self.pc += 4;
                    } else {
                        self.pc += 2;
                    }
                }
                _ => println!("Unknown opcode"),
            },
            // FX__
            0xF000 => match self.opcode & 0x00FF {
                // FX0A - Sets VX to the value of the delay timer
                0x0007 => {
                    let x = (self.opcode & 0x0F00) >> 8;
                    self.v[x as usize] = self.delay_timer;
                    self.pc += 2;
                }
                // FX0A - A key press is awaited, and then stored in VX
                0x000A => {
                    let x = (self.opcode & 0x0F00) >> 8;
                    let mut key_pressed = false;

                    for i in 0..16 {
                        if self.key[i] != 0 {
                            self.v[x as usize] = i as u8;
                            key_pressed = true;
                        }
                    }

                    // only increment pc if we press a key
                    // this is different from how the C++ program does it
                    if key_pressed {
                        self.pc += 2;
                    }
                }
                // FX15 - Sets the delay timer to VX
                0x0015 => {
                    let x = (self.opcode & 0x0F00) >> 8;
                    self.delay_timer = self.v[x as usize];
                    self.pc += 2;
                }
                // FX18 - Sets the sound timer to VX
                0x0018 => {
                    let x = (self.opcode & 0x0F00) >> 8;
                    self.sound_timer = self.v[x as usize];
                    self.pc += 2;
                }
                // FX1E - Adds VX to i
                0x001E => {
                    let x = (self.opcode & 0x0F00) >> 8;
                    let n = self.i.wrapping_add(self.v[x as usize] as u16);

                    if n > 0xFFF {
                        self.v[0xF] = 1;
                    } else {
                        self.v[0xF] = 0;
                    }

                    self.i += self.v[x as usize] as u16;
                    self.pc += 2;
                }
                /* 
                    FX29 - Sets i to the location of the sprite for the character in VX.
                    Characters 0-F (in hex) are represented by a 4x5 font
                */
                0x0029 => {
                    let x = (self.opcode & 0x0F00) >> 8;
                    self.i = (self.v[x as usize] * 0x5) as u16;
                    self.pc += 2;
                }
                /* 
                    FX33 - Stores the binary-coded decimal representation of VX at the 
                    address i, i + 1, and i + 2
                */
                0x0033 => {
                    let x = (self.opcode & 0x0F00) >> 8;
                    self.memory[self.i as usize] = self.v[x as usize] / 100;
                    self.memory[self.i as usize + 1] = (self.v[x as usize] / 10) % 10;
                    self.memory[self.i as usize + 2] = (self.v[x as usize] % 100) % 10;
                    self.pc += 2;
                }
                // FX55 - Stores V0 to VX in memory starting at address i
                0x0055 => {
                    let x = (self.opcode & 0x0F00) >> 8;

                    for index in 0..x {
                        self.memory[(self.i + index) as usize] = self.v[index as usize];
                    }

                    // On original interpreter, when operation is done i = i + x + 1
                    self.i += x + 1;

                    self.pc += 2;
                }
                // does things that are important i think???
                0x0065 => {
                    let x = (self.opcode & 0x0F00) >> 8;

                    for index in 0..=x {
                        self.v[index as usize] = self.memory[(self.i + index) as usize];
                    }

                    // On original interpreter, when operation is done i = i + x + 1
                    self.i += x + 1;

                    self.pc += 2;
                }
                _ => println!("Unknown opcode"),
            },
            _ => println!("Unimplemented opcode {:X}", self.opcode & 0xF000),
        }

        // Update timers
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }

        if self.sound_timer > 0 {
            if self.sound_timer == 1 {
                // Do sound things
            }
            self.sound_timer -= 1;
        }
    }
}
