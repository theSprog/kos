use alloc::string::String;

pub fn bytes_to_str(bytes: &[u8]) -> &str {
    let str_slice = core::str::from_utf8(bytes).unwrap();
    str_slice.trim_end_matches(char::from(0))
}

pub fn uuid_str(uuid_slice: &[u8]) -> String {
    assert_eq!(uuid_slice.len(), 16, "Input slice must have length 16");

    let mut uuid_str = String::with_capacity(36);

    for (i, byte) in uuid_slice.iter().enumerate() {
        if i == 4 || i == 6 || i == 8 || i == 10 {
            uuid_str.push('-');
        }

        let hex = alloc::format!("{:02x}", byte);
        uuid_str.push_str(&hex);
    }

    uuid_str
}
