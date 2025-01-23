/// Encode `data` as a 0x-prefixed hex string.
pub fn hex_encode<T: AsRef<[u8]>>(data: T) -> String {
    let hex = hex::encode(data);

    let mut s = "0x".to_string();
    s.push_str(hex.as_str());
    s
}
