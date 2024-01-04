pub fn get_bool_from_buf(buf: &[u8], index: usize) -> bool {
    buf[index] != 0
}

pub fn get_u8_from_buf(buf: &[u8], index: usize) -> u8 {
    buf[index]
}

pub fn get_u16_from_buf(buf: &[u8], index: usize) -> u16 {
    u16::from_be_bytes([buf[index], buf[index + 1]])
}

pub fn get_i16_from_buf(buf: &[u8], index: usize) -> i16 {
    i16::from_be_bytes([buf[index], buf[index + 1]])
}

pub fn get_u32_from_buf(buf: &[u8], index: usize) -> u32 {
    u32::from_be_bytes([
        buf[index],
        buf[index + 1],
        buf[index + 2],
        buf[index + 3],
    ])
}

pub fn get_u64_from_buf(buf: &[u8], index: usize) -> u64 {
    u64::from_be_bytes([
        buf[index],
        buf[index + 1],
        buf[index + 2],
        buf[index + 3],
        buf[index + 4],
        buf[index + 5],
        buf[index + 6],
        buf[index + 7],
    ])
}