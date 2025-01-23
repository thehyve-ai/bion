
pub fn pad_empty_bytes(data: &[u8], target: usize) -> Result<Vec<u8>, anyhow::Error> {
    if data.len() > target {
        return Err(anyhow::anyhow!(
            "pad_empty_bytes: Byte length can not exceed target.",
        ));
    }
    let mut bytes_vec = data.to_vec();
    bytes_vec.resize(target, 0);
    Ok(bytes_vec)
}
