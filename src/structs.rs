pub struct VirtualChip {
    pub memory: [u8; 4096],
    pub v: [u8; 16], // general purpose registers
    pub i: u16,      // special register
    pub pc: u16,
    pub sp: u8,
    pub stack: [u16; 16],
    pub v_screen: [[bool; 64]; 32], // Chip-8 screen has 64x32 pixels
    pub keyboard: [bool; 16],
}
