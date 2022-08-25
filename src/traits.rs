pub trait LRString<'lrstr> {
    fn as_ascii(&self) -> &str;
}

macro_rules! lrsrting {
    ($size:expr) => {
        impl<'lrstr> LRString<'lrstr> for [i8; $size] {
            fn as_ascii(&self) -> &str {
                std::str::from_utf8(
                    &unsafe { std::mem::transmute::<&[i8; $size], &[u8; $size]>(self) }
                        [..self.iter().position(|&i| i == 0).unwrap_or_default()],
                )
                .map(|ascii| ascii.trim_matches('\0'))
                .unwrap_or_default()
            }
        }
    };
}

lrsrting!(20);
lrsrting!(64);
lrsrting!(128);

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
