// Memory-related utility functions

pub fn hi_byte(addr: u16) -> u8 {
    ((addr >> 8) & 0xFF) as u8
}

pub fn lo_byte(addr: u16) -> u8 {
    (addr & 0xFF) as u8
}

pub fn make_address(lo: u8, hi: u8) -> u16 {
    ((hi as u16) << 8) & 0xFF00 | ((lo as u16) & 0xFF)
}

pub fn page_num(addr: u16) -> u16 {
    addr & 0xFF00
}