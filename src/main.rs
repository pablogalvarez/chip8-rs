use minifb::{Key, KeyRepeat, Window, WindowOptions};
use std::fs;

mod instructions;

mod structs;
use structs::VirtualChip;

const WINDOW_WIDTH: usize = 64;
const WINDOW_HEIGHT: usize = 32;
const DIGITS_SPRITES: [u8; 80] = [
    0xf0, 0x90, 0x90, 0x90, 0xf0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xf0, 0x10, 0xf0, 0x80, 0xf0, // 2
    0xf0, 0x10, 0xf0, 0x10, 0xf0, // 3
    0x90, 0x90, 0xf0, 0x10, 0x10, // 4
    0xf0, 0x80, 0xf0, 0x10, 0xf0, // 5
    0xf0, 0x80, 0xf0, 0x90, 0xf0, // 6
    0xf0, 0x10, 0x20, 0x40, 0x40, // 7
    0xf0, 0x90, 0xf0, 0x90, 0xf0, // 8
    0xf0, 0x90, 0xf0, 0x10, 0xf0, // 9
    0xf0, 0x90, 0xf0, 0x90, 0x90, // A
    0xe0, 0x90, 0xe0, 0x90, 0xe0, // B
    0xf0, 0x80, 0x80, 0x80, 0xf0, // C
    0xe0, 0x90, 0x90, 0x90, 0xe0, // D
    0xf0, 0x80, 0xf0, 0x80, 0xf0, // E
    0xf0, 0x80, 0xf0, 0x80, 0x80, // F
]

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

fn execute(chip: &mut VirtualChip, window: &Window, op: u8, n2: u8, n3: u8, n4: u8) {
    match op {
        0x0 => match n4 {
            0x0 => instructions::cls(chip),
            0xe => instructions::return_from_subroutine(chip),
            _ => return,
        },
        0x1 => instructions::jump_to_nnn(chip, n2, n3, n4),
        0x2 => instructions::call(chip, n2, n3, n4),
        0x3 => instructions::skip_instruction(chip, n2, n3, n4),
        0x4 => instructions::sne_vx_byte(chip, n2, n3, n4),
        0x5 => instructions::se_vx_vy(chip, n2, n3),
        0x6 => instructions::ld_vx_byte(chip, n2, n3, n4),
        0x7 => instructions::add_vx_byte(chip, n2, n3, n4),
        0x8 => match n4 {
            0x0 => instructions::ld_vx_vy(chip, n2, n3),
            0x1 => instructions::or_vx_vy(chip, n2, n3),
            0x2 => instructions::and_vx_vy(chip, n2, n3),
            0x3 => instructions::xor_vx_vy(chip, n2, n3),
            0x4 => instructions::add_vx_vy(chip, n2, n3),
            0x5 => instructions::sub_vx_vy(chip, n2, n3),
            0x6 => instructions::shr_vx_vy(chip, n2),
            0x7 => instructions::subn_vx_vy(chip, n2, n3),
            0xe => instructions::shl_vx_vy(chip, n2),
            _ => return,
        },
        0x9 => instructions::sne_vx_vy(chip, n2, n3),
        0xa => instructions::ld_i_addr(chip, n2, n3, n4),
        0xb => instructions::jump_to_nnn_v0(chip, n2, n3, n4),
        0xc => instructions::rnd_vx_byte(chip, n2, n3, n4),
        0xd => instructions::drw(chip, n2, n3, n4),
        0xe => match n3 {
            0x9 => instructions::skp_vx(chip, n2),
            0xa => instructions::sknp_vx(chip, n2),
            _ => return,
        },
        0xf => match n3 {
            0x0 => match n4 {
                0x7 => {}
                0xa => instructions::ld_vx_k(chip, window, n2),
                _ => return,
            },
            0x1 => match n4 {
                0xE => instructions::add_i_vx(chip, n2, n3, n4),
                _ => return,
            },
            0x2 =>
            0x3 => instructions::ld_b_vx(chip, n2),
            0x5 => instructions::ld_i_vx(chip, n2),
            0x6 => instructions::ld_vx_i(chip, n2),
            _ => return,
        },
        _ => return,
    }
}

fn main() {
    let mut window = Window::new(
        "Chip8-rs",
        WINDOW_WIDTH,
        WINDOW_HEIGHT,
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
        keyboard: [false; 16],
    };

    // Load digits sprites into memory
    chip.memory[0..80].copy_from_slice(&DIGITS_SPRITES);

    // TODO: rom_path must be a shell argument
    let rom_path = "ibm_logo.ch8";
    let rom_data: Vec<u8> = load_rom(rom_path);

    let rom_size = rom_data.len();
    chip.memory[512..rom_size + 512].copy_from_slice(&rom_data);

    // Main loop `fetch -> decode -> execute`
    while window.is_open() {
        window.get_keys().iter().for_each(|key| match key {
            Key::Key0 => chip.keyboard[0] = true,
            Key::Key1 => chip.keyboard[1] = true,
            Key::Key2 => chip.keyboard[2] = true,
            Key::Key3 => chip.keyboard[3] = true,
            Key::Key4 => chip.keyboard[4] = true,
            Key::Key5 => chip.keyboard[5] = true,
            Key::Key6 => chip.keyboard[6] = true,
            Key::Key7 => chip.keyboard[7] = true,
            Key::Key8 => chip.keyboard[8] = true,
            Key::Key9 => chip.keyboard[9] = true,
            Key::A => chip.keyboard[10] = true,
            Key::B => chip.keyboard[11] = true,
            Key::C => chip.keyboard[12] = true,
            Key::D => chip.keyboard[13] = true,
            Key::E => chip.keyboard[14] = true,
            Key::F => chip.keyboard[15] = true,
            _ => (),
        });

        let opcode = fetch(&chip);
        chip.pc += 2;

        let (op, n2, n3, n4) = decode(opcode);
        execute(&mut chip, &window, op, n2, n3, n4);

        let mut buffer: Vec<u32> = Vec::new();
        for column in 0..chip.v_screen.len() {
            for row in 0..chip.v_screen[0].len() {
                let mut rgb_value: u32 = 0;
                if chip.v_screen[column][row] {
                    rgb_value = 0xffffff;
                } else {
                    rgb_value = 0x000000;
                }
                buffer.push(rgb_value);
            }
        }
        window.update_with_buffer(&buffer, WINDOW_WIDTH, WINDOW_HEIGHT);
        chip.keyboard = [false; 16];
    }
}
