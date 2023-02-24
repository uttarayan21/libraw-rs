pub trait LRString<'lrstr> {
    fn as_ascii(&self) -> &str;
}

impl<'lrstr> LRString<'lrstr> for &[i8] {
    fn as_ascii(&self) -> &str {
        std::str::from_utf8(
            &unsafe { std::mem::transmute::<&[i8], &[u8]>(self) }
                [..self.iter().position(|&i| i == 0).unwrap_or_default()],
        )
        .map(|ascii| ascii.trim_matches('\0'))
        .unwrap_or_default()
    }
}

impl<'lrstr, const LEN: usize> LRString<'lrstr> for [i8; LEN] {
    fn as_ascii(&self) -> &str {
        std::str::from_utf8(
            &unsafe { std::mem::transmute::<&[i8; LEN], &[u8; LEN]>(self) }
                [..self.iter().position(|&i| i == 0).unwrap_or_default()],
        )
        .map(|ascii| ascii.trim_matches('\0'))
        .unwrap_or_default()
    }
}
