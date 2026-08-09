use std::fs;

use minifb::{Window, WindowOptions};

struct VirtualChip {
    memory: [u8; 4096],
    v: [u8; 16], // general purpose registers
    i: u16,      // special register
    pc: u16,
    sp: u8,
    stack: [u16; 16],
    v_screen: [[bool; 64]; 32], // Chip-8 screen has 64x32 pixels
}

fn load_rom(rom_path: &str) -> Vec<u8> {
    match fs::read(rom_path) {
        Err(_) => panic!("Error leyendo la ROM"),
        Ok(v) => v,
    }
}

fn fetch(chip: &VirtualChip) -> u16 {
    // # Memory management
    //
    // Memory is addressed byte by byte
    //   - `0x200` -> holds 1 byte
    //   - `0x207` -> holds 1 byte
    //
    // Chip8 instructions are **2 bytes long**, so we first
    // read 2 bytes from memory and then we combine them,
    // with the first byte being the most significant one

    let pc = chip.pc as usize;
    let (first_byte, second_byte) = (chip.memory[pc], chip.memory[pc + 1]);

    ((first_byte as u16) << 8u16) | second_byte as u16
}

fn decode(opcode: u16) -> (u8, u8, u8, u8) {
    // # Decoding
    //
    // Every part of an instruction carries a meaning
    // so we need to extract them one by one. For example:
    // ```text
    // ***2nnn - CALL addr***
    // Call subroutine at *nnn*
    // ```
    (
        ((opcode >> 12) & 0xF) as u8,
        ((opcode >> 8) & 0xF) as u8,
        ((opcode >> 4) & 0xF) as u8,
        (opcode & 0xF) as u8,
    )
}

fn execute(chip: &mut VirtualChip, op: u8, n2: u8, n3: u8, n4: u8) {
    match op {
        0x0 => match n4 {
            0x0 => chip.v_screen = [[false; 64]; 32],
            0xe => {
                chip.pc = chip.stack[chip.sp as usize];
                chip.sp -= 1;
            }
            _ => return,
        },
        0x1 => chip.pc = (((n2 << 4) | n3) as u16) << 4 | n4 as u16,
        0x2 => {
            chip.sp += 1;
            chip.stack[chip.sp as usize] = chip.pc;
            chip.pc = (((n2 << 4) | n3) as u16) << 4 | n4 as u16;
        }
        0x3 => {
            if chip.v[n2 as usize] == (n3 << 4) | n4 {
                chip.pc += 2
            } else {
            }
        }
        0x4 => {
            if chip.v[n2 as usize] != (n3 << 4) | n4 {
                chip.pc += 2
            } else {
            }
        }
        0x5 => {
            if chip.v[n2 as usize] == chip.v[n3 as usize] {
                chip.pc += 2
            } else {
            }
        }
        0x6 => chip.v[n2 as usize] = (n3 << 4) | n4,
        0x7 => chip.v[n2 as usize] += (n3 << 4) | n4,
        0x8 => match n4 {
            0x0 => chip.v[n2 as usize] = chip.v[n3 as usize],
            0x1 => chip.v[n2 as usize] = chip.v[n2 as usize] | chip.v[n3 as usize],
            0x2 => chip.v[n2 as usize] = chip.v[n2 as usize] & chip.v[n3 as usize],
            0x3 => chip.v[n2 as usize] = chip.v[n2 as usize] ^ chip.v[n3 as usize],
            0x4 => {
                let sum: u16 = (chip.v[n2 as usize] as u16) + (chip.v[n3 as usize] as u16);
                if sum > 255 {
                    chip.v[15] = 1;
                    chip.v[n2 as usize] = sum as u8;
                } else {
                    chip.v[15] = 0;
                }
            }
            0x5 => {
                if chip.v[n2 as usize] > chip.v[n3 as usize] {
                    chip.v[15] = 1;
                    chip.v[n2 as usize] -= chip.v[n3 as usize]
                } else {
                    chip.v[15] = 0;
                }
            }
            0x6 => {
                if chip.v[n2 as usize] & 1 == 1 {
                    chip.v[15] = 1;
                    chip.v[n2 as usize] /= 2
                } else {
                    chip.v[15] = 0;
                }
            }
            0x7 => {
                if chip.v[n3 as usize] > chip.v[n2 as usize] {
                    chip.v[15] = 1;
                    chip.v[n2 as usize] = chip.v[n3 as usize] - chip.v[n2 as usize]
                } else {
                    chip.v[15] = 0;
                }
            }
            0xe => {
                if (chip.v[n2 as usize] >> 7) & 1 == 1 {
                    chip.v[15] = 1;
                    chip.v[n2 as usize] *= 2
                } else {
                    chip.v[15] = 0;
                }
            }
            _ => return,
        },
        0x9 => {
            if chip.v[n2 as usize] != chip.v[n3 as usize] {
                chip.pc += 2
            } else {
            }
        }
        0xa => chip.i = ((((n2 << 4) | n3) as u16) << 4) | n4 as u16,
        0xb => chip.pc = (((((n2 << 4) | n3) as u16) << 4) | n4 as u16) + chip.v[0] as u16,
        0xc => chip.v[n2 as usize] = rand::random::<u8>() & (n3 << 4 | n4),
        0xd => {
            chip.v[15] = 0;
            let mut mem_addr = chip.i as usize;
            for i in 0..n4 {
                let sprite_byte = chip.memory[mem_addr]; // 1 byte from memory
                for bit_idx in 0..8 {
                    let (screen_x, screen_y) = (
                        (chip.v[n2 as usize] as u16 + bit_idx) % 64,
                        (chip.v[n3 as usize] as u16 + i as u16) % 32,
                    );
                    let current_screen_value = chip.v_screen[screen_y as usize][screen_x as usize];
                    let bit_value = (sprite_byte >> (7 - bit_idx)) & 1;
                    let new_value = current_screen_value as u8 ^ bit_value;

                    if current_screen_value && bit_value == 1 {
                        chip.v[15] |= 1;
                    }
                    chip.v_screen[screen_y as usize][screen_x as usize] = new_value != 0;
                }
                mem_addr += 1;
            }
        }
        0xe => println!("SKP Vx"),
        0xf => match n3 {
            0x0 => {}
            0x1 => match n4 {
                0xE => chip.i += chip.v[n2 as usize] as u16,
                _ => return,
            },
            0x3 => {
                let vx = chip.v[n2 as usize];
                let mem_addr = chip.i as usize;

                chip.memory[mem_addr] = vx / 100;
                chip.memory[mem_addr + 1] = (vx % 100) / 10;
                chip.memory[mem_addr + 2] = vx % 10;
            }
            0x5 => {
                let mut mem_addr = chip.i as usize;
                for i in 0..=n2 {
                    chip.memory[mem_addr] = chip.v[i as usize];
                    mem_addr += 1;
                }
            }
            0x6 => {
                let mut mem_addr = chip.i as usize;
                for i in 0..=n2 {
                    chip.v[i as usize] = chip.memory[mem_addr];
                    mem_addr += 1;
                }
            }
            _ => return,
        },
        _ => return,
    }
}

fn main() {
    let mut window = Window::new(
        "Chip8-rs",
        64,
        32,
        WindowOptions {
            resize: true,
            ..WindowOptions::default()
        },
    )
    .expect("Unable to open Window");

    let mut chip = VirtualChip {
        memory: [0; 4096],
        v: [0; 16],
        i: 0,
        pc: 512,
        sp: 0,
        stack: [0; 16],
        v_screen: [[false; 64]; 32],
    };

    // TODO: rom_path must be a shell argument
    let rom_path = "ibm_logo.ch8";
    let rom_data: Vec<u8> = load_rom(rom_path);

    let rom_size = rom_data.len();
    chip.memory[512..rom_size + 512].copy_from_slice(&rom_data);

    // Main loop *fetch -> decode -> execute*
    loop {
        let opcode = fetch(&chip);
        chip.pc += 2;

        let (op, n2, n3, n4) = decode(opcode);
        execute(&mut chip, op, n2, n3, n4);
    }
}
