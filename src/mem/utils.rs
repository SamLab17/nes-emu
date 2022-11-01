// Memory-related utility functions


pub fn make_address(lo: u8, hi: u8) -> u16 {
    ((hi as u16) << 8) & 0xFF00 | ((lo as u16) & 0xFF)
}

pub fn page_num(addr: u16) -> u16 {
    addr & 0xFF00
}