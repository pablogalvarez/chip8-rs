use minifb::{Key, KeyRepeat, Window};

use crate::structs::VirtualChip;

pub fn cls(chip: &mut VirtualChip) {
    chip.v_screen = [[false; 64]; 32];
}

pub fn return_from_subroutine(chip: &mut VirtualChip) {
    chip.pc = chip.stack[chip.sp as usize];
    chip.sp -= 1;
}

pub fn jump_to_nnn(chip: &mut VirtualChip, n2: u8, n3: u8, n4: u8) {
    chip.pc = (((n2 << 4) | n3) as u16) << 4 | n4 as u16;
}

pub fn call(chip: &mut VirtualChip, n2: u8, n3: u8, n4: u8) {
    chip.sp += 1;
    chip.stack[chip.sp as usize] = chip.pc;
    chip.pc = (((n2 << 4) | n3) as u16) << 4 | n4 as u16;
}

pub fn skip_instruction(chip: &mut VirtualChip, n2: u8, n3: u8, n4: u8) {
    if chip.v[n2 as usize] == (n3 << 4) | n4 {
        chip.pc += 2;
    }
}

pub fn sne_vx_byte(chip: &mut VirtualChip, n2: u8, n3: u8, n4: u8) {
    if chip.v[n2 as usize] != (n3 << 4) | n4 {
        chip.pc += 2;
    }
}

pub fn se_vx_vy(chip: &mut VirtualChip, n2: u8, n3: u8) {
    if chip.v[n2 as usize] == chip.v[n3 as usize] {
        chip.pc += 2;
    }
}

pub fn ld_vx_byte(chip: &mut VirtualChip, n2: u8, n3: u8, n4: u8) {
    chip.v[n2 as usize] = (n3 << 4) | n4;
}

pub fn add_vx_byte(chip: &mut VirtualChip, n2: u8, n3: u8, n4: u8) {
    chip.v[n2 as usize] = chip.v[n2 as usize].wrapping_add((n3 << 4) | n4);
}

pub fn ld_vx_vy(chip: &mut VirtualChip, n2: u8, n3: u8) {
    chip.v[n2 as usize] = chip.v[n3 as usize];
}

pub fn or_vx_vy(chip: &mut VirtualChip, n2: u8, n3: u8) {
    chip.v[n2 as usize] = chip.v[n2 as usize] | chip.v[n3 as usize];
}

pub fn and_vx_vy(chip: &mut VirtualChip, n2: u8, n3: u8) {
    chip.v[n2 as usize] = chip.v[n2 as usize] & chip.v[n3 as usize];
}

pub fn xor_vx_vy(chip: &mut VirtualChip, n2: u8, n3: u8) {
    chip.v[n2 as usize] = chip.v[n2 as usize] ^ chip.v[n3 as usize];
}

pub fn add_vx_vy(chip: &mut VirtualChip, n2: u8, n3: u8) {
    let sum: u16 = (chip.v[n2 as usize] as u16) + (chip.v[n3 as usize] as u16);
    if sum > 255 {
        chip.v[15] = 1;
        chip.v[n2 as usize] = sum as u8;
    } else {
        chip.v[15] = 0;
    }
}

pub fn sub_vx_vy(chip: &mut VirtualChip, n2: u8, n3: u8) {
    if chip.v[n2 as usize] > chip.v[n3 as usize] {
        chip.v[15] = 1;
        chip.v[n2 as usize] -= chip.v[n3 as usize];
    } else {
        chip.v[15] = 0;
    }
}

pub fn shr_vx_vy(chip: &mut VirtualChip, n2: u8) {
    if chip.v[n2 as usize] & 1 == 1 {
        chip.v[15] = 1;
        chip.v[n2 as usize] /= 2;
    } else {
        chip.v[15] = 0;
    }
}

pub fn subn_vx_vy(chip: &mut VirtualChip, n2: u8, n3: u8) {
    if chip.v[n3 as usize] > chip.v[n2 as usize] {
        chip.v[15] = 1;
        chip.v[n2 as usize] = chip.v[n3 as usize] - chip.v[n2 as usize];
    } else {
        chip.v[15] = 0;
    }
}

pub fn shl_vx_vy(chip: &mut VirtualChip, n2: u8) {
    if (chip.v[n2 as usize] >> 7) & 1 == 1 {
        chip.v[15] = 1;
        chip.v[n2 as usize] *= 2;
    } else {
        chip.v[15] = 0;
    }
}

pub fn sne_vx_vy(chip: &mut VirtualChip, n2: u8, n3: u8) {
    if chip.v[n2 as usize] != chip.v[n3 as usize] {
        chip.pc += 2;
    }
}

pub fn ld_i_addr(chip: &mut VirtualChip, n2: u8, n3: u8, n4: u8) {
    chip.i = ((((n2 << 4) | n3) as u16) << 4) | n4 as u16;
}

pub fn jump_to_nnn_v0(chip: &mut VirtualChip, n2: u8, n3: u8, n4: u8) {
    chip.pc = (((((n2 << 4) | n3) as u16) << 4) | n4 as u16) + chip.v[0] as u16;
}

pub fn rnd_vx_byte(chip: &mut VirtualChip, n2: u8, n3: u8, n4: u8) {
    chip.v[n2 as usize] = rand::random::<u8>() & (n3 << 4 | n4);
}

pub fn drw(chip: &mut VirtualChip, n2: u8, n3: u8, n4: u8) {
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

pub fn skp_vx(chip: &mut VirtualChip, n2: u8) {
    let idx = chip.v[n2 as usize];
    if chip.keyboard[idx as usize] {
        chip.pc += 2;
    }
}

pub fn sknp_vx(chip: &mut VirtualChip, n2: u8) {
    let idx = chip.v[n2 as usize];
    if !chip.keyboard[idx as usize] {
        chip.pc += 2;
    }
}

pub fn ld_vx_k(chip: &mut VirtualChip, window: &Window, n2: u8) {
    if window.get_keys().len() == 0 {
        chip.pc -= 2;
    }
    if let Some(k) = window.get_keys().first() {
        match k {
            Key::Key0 => chip.v[n2 as usize] = 0,
            Key::Key1 => chip.v[n2 as usize] = 1,
            Key::Key2 => chip.v[n2 as usize] = 2,
            Key::Key3 => chip.v[n2 as usize] = 3,
            Key::Key4 => chip.v[n2 as usize] = 4,
            Key::Key5 => chip.v[n2 as usize] = 5,
            Key::Key6 => chip.v[n2 as usize] = 6,
            Key::Key7 => chip.v[n2 as usize] = 7,
            Key::Key8 => chip.v[n2 as usize] = 8,
            Key::Key9 => chip.v[n2 as usize] = 9,
            Key::A => chip.v[n2 as usize] = 10,
            Key::B => chip.v[n2 as usize] = 11,
            Key::C => chip.v[n2 as usize] = 12,
            Key::D => chip.v[n2 as usize] = 13,
            Key::E => chip.v[n2 as usize] = 14,
            Key::F => chip.v[n2 as usize] = 15,
            _ => (),
        }
    };
}

pub fn ld_vx_dt(chip: &mut VirtualChip, n2: u8, n3: u8, n4: u8) {}

pub fn add_i_vx(chip: &mut VirtualChip, n2: u8, n3: u8, n4: u8) {
    chip.i += chip.v[n2 as usize] as u16;
}

pub fn ld_f_vx(chip: &mut VirtualChip, n2: u8) {
    chip.i = chip.v[n2 as usize] as u16 * 5;
}

pub fn ld_b_vx(chip: &mut VirtualChip, n2: u8) {
    let vx = chip.v[n2 as usize];
    let mem_addr = chip.i as usize;

    chip.memory[mem_addr] = vx / 100;
    chip.memory[mem_addr + 1] = (vx % 100) / 10;
    chip.memory[mem_addr + 2] = vx % 10;
}

pub fn ld_i_vx(chip: &mut VirtualChip, n2: u8) {
    let mut mem_addr = chip.i as usize;
    for i in 0..=n2 {
        chip.memory[mem_addr] = chip.v[i as usize];
        mem_addr += 1;
    }
}

pub fn ld_vx_i(chip: &mut VirtualChip, n2: u8) {
    let mut mem_addr = chip.i as usize;
    for i in 0..=n2 {
        chip.v[i as usize] = chip.memory[mem_addr];
        mem_addr += 1;
    }
}
