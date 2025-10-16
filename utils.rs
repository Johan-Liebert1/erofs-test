use anyhow::Result;

#[fn_error_context::context("For u64 {msg}")]
pub fn u64_le(buf: &[u8], msg: &str) -> Result<u64> {
    let thing: &[u8; 8] = buf[..8].try_into().unwrap();
    Ok(u64::from_le_bytes(*thing))
}

#[fn_error_context::context("For u32 {msg}")]
pub fn u32_le(buf: &[u8], msg: &str) -> Result<u32> {
    let thing: &[u8; 4] = buf[..4].try_into().unwrap();
    Ok(u32::from_le_bytes(*thing))
}

#[fn_error_context::context("For u16 {msg}")]
pub fn u16_le(buf: &[u8], msg: &str) -> Result<u16> {
    let thing: &[u8; 2] = buf[..2].try_into().unwrap();
    Ok(u16::from_le_bytes(*thing))
}
