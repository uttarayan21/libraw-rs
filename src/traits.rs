pub trait LRString<'lrstr> {
    fn as_ascii(&self) -> &str;
}

impl<'lrstr> LRString<'lrstr> for [i8; 64] {
    fn as_ascii(&self) -> &str {
        std::str::from_utf8(
            &unsafe { std::mem::transmute::<&[i8; 64], &[u8; 64]>(self) }
                [..self.iter().position(|&i| i == 0).unwrap_or_default()],
        )
        .map(|ascii| ascii.trim_matches('\0'))
        .unwrap_or_default()
    }
}

impl<'lrstr> LRString<'lrstr> for [i8; 128] {
    fn as_ascii(&self) -> &str {
        std::str::from_utf8(
            &unsafe { std::mem::transmute::<&[i8; 128], &[u8; 128]>(self) }
                [..self.iter().position(|&i| i == 0).unwrap_or_default()],
        )
        .map(|ascii| ascii.trim_matches('\0'))
        .unwrap_or_default()
    }
}
