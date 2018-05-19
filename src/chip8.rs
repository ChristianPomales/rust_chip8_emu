use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

#[allow(dead_code)]
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

#[allow(dead_code)]
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
            Ok(_) => print!("{} {} contains bytes\n", display, rom.len()),
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

        // println!("--- begin memory dump ---");

        // for byte in self.memory.iter() {
        //     print!("{:x}, ", byte);
        // }

        // println!("\n--- end memory dump ---");

        println!("\n\n opcode -> {:x} \n", self.opcode & 0xF000);

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
            0x1000 => {}
            // 2NNN - Calls subroutine at NNN
            0x2000 => {
                self.stack[self.sp as usize] = self.pc;
                self.sp += 1;
                self.pc = self.opcode & 0x0FFF;
            }
            // 3XNN - Skips the next instruction if VX = NX
            0x3000 => {}
            // 4XNN - Skips the next instruction if VX != NX
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
            // 5XY0 - Skips the next instruction if VX = VY
            0x5000 => {}
            // 6XNN - Sets VX to NN
            0x6000 => {
                let x = (self.opcode & 0x0F00) >> 8;
                self.v[x as usize] = (self.opcode & 0x00FF) as u8;
                self.pc += 2;
            }
            // 7XNN - Adds NN to VX
            0x7000 => {}
            // 8XY_
            0x8000 => {}
            // 9XY0 - Skips the next instruction if VX doesn't equal VY
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
            0xB000 => {}
            // CXNN - Sets VX to a random number, masked by NN
            0xC000 => {}
            /* 
                    DXYN: Draws a sprite at coordinate (VX, VY) that has a width of 8 pixels
                    and a height of N pixels.
                    Each row of 8 pixels is read as bit-coded starting from memory location i.
                    I value doesn't change after the execution of this instruction.
                    VF is set to 1 if any screen pixels are flipped from set to unset when
                    the sprire is drawn, and to 0 if that doesn't happen.
                */
            0xD000 => {}
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
            0xF000 => {}
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
