/// Obtains the highest possible value of the implementing type.
pub trait MaxValue {
    /// The highest possible value of the type.
    const MAX: Self;
}

impl MaxValue for u8 {
    const MAX: u8 = u8::MAX;
}

impl MaxValue for u16 {
    const MAX: u16 = u16::MAX;
}

impl MaxValue for u32 {
    const MAX: u32 = u32::MAX;
}

impl MaxValue for u64 {
    const MAX: u64 = u64::MAX;
}

impl MaxValue for u128 {
    const MAX: u128 = u128::MAX;
}

impl MaxValue for usize {
    const MAX: usize = usize::MAX;
}
