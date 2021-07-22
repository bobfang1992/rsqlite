pub fn as_u16_be(array: &[u8; 2]) -> u16 {
    ((array[0] as u16) << 8) + (array[1] as u16)
}
pub fn as_u32_be(array: &[u8; 4]) -> u32 {
    ((array[0] as u32) << 24)
        + ((array[1] as u32) << 16)
        + ((array[2] as u32) << 8)
        + (array[3] as u32)
}
