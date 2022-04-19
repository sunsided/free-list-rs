use std::fmt::Debug;

/// A trait for the type that is used as an index into the list.
/// The type needs to be convertible to `usize` and should generally
/// be as small as possible; the list can store up to the maximum
/// value available by the type _minus one_.
///
/// ## Example
/// If the list only contains up to 254 elements, the type `u8` should be used
/// since `u8::MAX - 1 == 254`.
pub trait IndexType:
    Sized + Copy + Eq + PartialOrd + Ord + Debug + MaxValue + FromAndIntoUsize
{
}

/// Automatic implementation of the `IndexType` trait.
impl<T> IndexType for T where
    T: Sized + Copy + Eq + PartialOrd + Ord + Debug + MaxValue + FromAndIntoUsize
{
}

/// A trait providing a conversion method into and from `usize` values.
///
/// ## Safety
/// This conversion can fail if the specified `value` is higher than
/// the highest possible value of the underlying type, and vice versa.
pub trait FromAndIntoUsize {
    unsafe fn from(value: usize) -> Self;
    unsafe fn into(self) -> usize;
}

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

impl FromAndIntoUsize for u8 {
    unsafe fn from(value: usize) -> Self {
        debug_assert!(
            value <= Self::MAX as usize,
            "can address at most {} values",
            Self::MAX
        );
        value as Self
    }

    unsafe fn into(self) -> usize {
        debug_assert!(
            self <= usize::MAX as Self,
            "can address at most {} values",
            usize::MAX
        );
        self as usize
    }
}

impl FromAndIntoUsize for u16 {
    unsafe fn from(value: usize) -> Self {
        debug_assert!(
            value <= Self::MAX as usize,
            "can address at most {} values",
            Self::MAX
        );
        value as Self
    }

    unsafe fn into(self) -> usize {
        debug_assert!(
            self <= usize::MAX as Self,
            "can address at most {} values",
            usize::MAX
        );
        self as usize
    }
}

impl FromAndIntoUsize for u32 {
    unsafe fn from(value: usize) -> Self {
        debug_assert!(
            value <= Self::MAX as usize,
            "can address at most {} values",
            Self::MAX
        );
        value as Self
    }

    unsafe fn into(self) -> usize {
        debug_assert!(
            self <= usize::MAX as Self,
            "can address at most {} values",
            usize::MAX
        );
        self as usize
    }
}

impl FromAndIntoUsize for u64 {
    unsafe fn from(value: usize) -> Self {
        debug_assert!(
            value <= Self::MAX as usize,
            "can address at most {} values",
            Self::MAX
        );
        value as Self
    }

    unsafe fn into(self) -> usize {
        debug_assert!(
            self <= usize::MAX as Self,
            "can address at most {} values",
            usize::MAX
        );
        self as usize
    }
}

impl FromAndIntoUsize for u128 {
    unsafe fn from(value: usize) -> Self {
        debug_assert!(
            value <= Self::MAX as usize,
            "can address at most {} values",
            Self::MAX
        );
        value as Self
    }

    unsafe fn into(self) -> usize {
        debug_assert!(
            self <= usize::MAX as Self,
            "can address at most {} values",
            usize::MAX
        );
        self as usize
    }
}

/// The only safe implementation of `FromUnsafe`.
impl FromAndIntoUsize for usize {
    unsafe fn from(value: usize) -> Self {
        value
    }

    unsafe fn into(self) -> usize {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn u8_max() {
        assert_eq!(<u8 as MaxValue>::MAX, 255);
    }

    #[test]
    fn u16_max() {
        assert_eq!(<u16 as MaxValue>::MAX, 65535);
    }

    #[test]
    fn u32_max() {
        assert_eq!(<u32 as MaxValue>::MAX, 4294967295);
    }

    #[test]
    fn u64_max() {
        assert_eq!(<u64 as MaxValue>::MAX, 18446744073709551615);
    }

    #[test]
    fn u128_max() {
        assert_eq!(
            <u128 as MaxValue>::MAX,
            340282366920938463463374607431768211455
        );
    }

    #[test]
    fn usize_max() {
        assert_eq!(<usize as MaxValue>::MAX, usize::MAX);
    }
}
