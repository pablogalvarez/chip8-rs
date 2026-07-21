use std::fs;

struct VirtualChip {
    memory: [u8; 4096],
    v: [u8; 16], // general purpose registers
    i: u16,      // special register
    pc: u16,
    sp: u8,
    stack: [u16; 16],
}

fn load_rom(rom_path: &str) -> Vec<u8> {
    match fs::read(rom_path) {
        Err(_) => panic!("Error leyendo la ROM"),
        Ok(v) => v,
    }
}

fn fetch(chip: &VirtualChip) -> u16 {
    let pc = chip.pc as usize;
    let (first_byte, second_byte) = (chip.memory[pc], chip.memory[pc + 1]);

    ((first_byte as u16) << 8u16) | second_byte as u16
}

fn decode(opcode: u16) -> (u8, u8, u8, u8) {
    (
        ((opcode >> 12) & 0xF) as u8,
        ((opcode >> 8) & 0xF) as u8,
        ((opcode >> 4) & 0xF) as u8,
        (opcode & 0xF) as u8,
    )
}

fn execute() {}

fn main() {
    let mut chip = VirtualChip {
        memory: [0; 4096],
        v: [0; 16],
        i: 0,
        pc: 512,
        sp: 0,
        stack: [0; 16],
    };

    // TODO: rom_path must be a shell argument
    let rom_path = "ibm_logo.ch8";
    let rom_data: Vec<u8> = load_rom(rom_path);

    let rom_size = rom_data.len();
    chip.memory[512..rom_size + 512].copy_from_slice(&rom_data);
}
