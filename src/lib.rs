mod max_value;

use crate::max_value::MaxValue;

use std::fmt::Debug;
use std::mem::ManuallyDrop;

/// A trait for the type that is used as an index into the list.
/// The type needs to be convertible to `usize` and should generally
/// be as small as possible; the list can store up to the maximum
/// value available by the type _minus one_.
///
/// ## Example
/// If the list only contains up to 254 elements, the type `u8` should be used
/// since `u8::MAX - 1 == 254`.
pub trait IndexType:
    Sized + Copy + Eq + PartialOrd + Ord + Debug + MaxValue + Into<usize> + From<usize>
{
}

/// Automatic implementation of the `IndexType` trait.
impl<T> IndexType for T where
    T: Sized + Copy + Eq + PartialOrd + Ord + Debug + MaxValue + Into<usize> + From<usize>
{
}

/// Provides an indexed free list with constant-time removals from anywhere
/// in the list without invalidating indices. T must be trivially constructible
/// and destructible.
pub struct FreeList<T, TIndex = usize>
where
    T: Default,
    TIndex: IndexType,
{
    /// The number of live elements in the list.
    #[cfg(debug_assertions)]
    length: usize,
    /// The actual data.
    data: Vec<FreeElement<T, TIndex>>,
    /// The index of the the most recently freed element, or `SENTINEL` if no
    /// element is free.
    first_free: TIndex,
}

union FreeElement<T, TIndex>
where
    TIndex: IndexType,
{
    /// This field contains the data as long as the element was not removed.
    element: ManuallyDrop<T>,
    /// If the element was "removed", this index is pointing to the next index
    /// of an element that is also freed, or `SENTINEL` if no other element is free.
    next: TIndex,
}

impl<T, TIndex> Default for FreeList<T, TIndex>
where
    T: Default,
    TIndex: IndexType,
{
    fn default() -> Self {
        Self {
            data: Vec::default(),
            first_free: Self::SENTINEL,
            #[cfg(debug_assertions)]
            length: 0,
        }
    }
}

impl<T, TIndex> FreeList<T, TIndex>
where
    T: Default,
    TIndex: IndexType,
{
    /// The sentinel value indicates the absence of a valid value.
    pub(crate) const SENTINEL: TIndex = TIndex::MAX;

    /// Inserts an element to the free list and returns an index to it.
    pub fn push(&mut self, element: T) -> TIndex {
        #[cfg(debug_assertions)]
        {
            self.length += 1;
        }

        return if self.first_free != Self::SENTINEL {
            let index = self.first_free;
            let index_usize = index.into();

            // Set the "first free" pointer to the next free index.
            self.first_free = unsafe { self.data[index_usize].next };

            // Place the element into the previously free location.
            self.data[index_usize].element = ManuallyDrop::new(element);
            index
        } else {
            let fe = FreeElement {
                element: ManuallyDrop::new(element),
            };
            self.data.push(fe);
            <TIndex as From<usize>>::from(self.data.len() - 1)
        };
    }

    /// Removes the nth element from the free list.
    pub fn erase(&mut self, n: TIndex) {
        self.first_free = Self::SENTINEL;
        if self.data.is_empty() {
            return;
        }
        debug_assert!(!self.debug_is_in_free_list(n));

        #[cfg(debug_assertions)]
        debug_assert!(self.length > 0);

        let n_usize = n.into();
        unsafe { ManuallyDrop::drop(&mut self.data[n_usize].element) };
        self.data[n_usize].next = self.first_free;
        self.first_free = n;

        #[cfg(debug_assertions)]
        {
            self.length -= 1;
        }
    }

    /// Removes all elements from the free list.
    pub fn clear(&mut self) {
        if self.data.is_empty() {
            assert_eq!(self.first_free, Self::SENTINEL);
            return;
        }

        // Collect all free indexes and sort them such that they
        // are in ascending order.
        let mut free_indexes = Vec::new();
        let mut token = self.first_free;
        while token != Self::SENTINEL {
            free_indexes.push(token);
            token = unsafe { self.data[token.into()].next };
        }
        free_indexes.sort();

        // As long as there are free indexes, pop elements from the
        // vector and ignore them if they correspond to a free index.
        if !free_indexes.is_empty() {
            for (i, entry) in self.data.iter_mut().enumerate() {
                if free_indexes.is_empty()
                    || *free_indexes.last().unwrap() != <TIndex as From<usize>>::from(i)
                {
                    // This is not a pointer entry, drop required.
                    unsafe { ManuallyDrop::drop(&mut entry.element) };
                } else {
                    // The entry only contains a index to another free spot; nothing to drop.
                    let _ = free_indexes.pop();
                }
            }
        }

        // At this point there are no free indexes anymore, so the
        // list can be trivially cleared.
        self.data.clear();
        self.first_free = Self::SENTINEL;

        #[cfg(debug_assertions)]
        {
            self.length = 0;
        }
    }

    /// Gets a reference to the value at the specified index.
    ///
    /// # Safety
    ///
    /// If the element at the specified index was erased, the union now acts
    ///  as a pointer to the next free element. Accessing the same index again after that.
    ///  is undefined behavior.
    #[inline]
    pub unsafe fn at(&self, index: TIndex) -> &T {
        debug_assert_ne!(index, Self::SENTINEL);
        debug_assert!(!self.debug_is_in_free_list(index));
        &self.data[index.into()].element
    }

    /// Gets a mutable reference to the value at the specified index.
    ///
    /// # Safety
    ///
    /// If the element at the specified index was erased, the union now acts
    /// as a pointer to the next free element. Accessing the same index again after that.
    /// is undefined behavior.
    #[inline]
    pub unsafe fn at_mut(&mut self, index: TIndex) -> &mut T {
        debug_assert_ne!(index, Self::SENTINEL);
        debug_assert!(!self.debug_is_in_free_list(index));
        &mut self.data[index.into()].element
    }

    /// Gets the current capacity of the list.
    #[allow(dead_code)]
    pub fn capacity(&self) -> usize {
        self.data.len()
    }

    /// Gets the number of elements in the list.
    #[allow(dead_code)]
    pub fn debug_len(&self) -> usize {
        #[cfg(debug_assertions)]
        return self.length;
        #[cfg(not(debug_assertions))]
        unimplemented!()
    }

    #[allow(dead_code, unused_variables)]
    fn debug_is_in_free_list(&self, n: TIndex) -> bool {
        #[cfg(any(debug_assertions, test))]
        {
            assert_ne!(n, Self::SENTINEL);
            let mut token = self.first_free;
            while token != Self::SENTINEL {
                if n == token {
                    return true;
                }
                token = unsafe { self.data[token.into()].next };
            }
            return false;
        }
        #[cfg(not(any(debug_assertions, test)))]
        unimplemented!()
    }
}

impl<T, TIndex> Drop for FreeList<T, TIndex>
where
    T: Default,
    TIndex: IndexType,
{
    fn drop(&mut self) {
        self.clear();
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Default, Debug, PartialEq, PartialOrd)]
    struct Complex(f64, f64);

    impl Drop for Complex {
        fn drop(&mut self) {
            self.0 = 42.;
            self.1 = 1337.0;
        }
    }

    #[test]
    fn after_construction_has_no_first_free() {
        let list = FreeList::<Complex>::default();
        assert_eq!(list.first_free, FreeList::<Complex>::SENTINEL);
        assert_eq!(list.capacity(), 0);
    }

    #[test]
    fn after_insertion_has_no_first_free() {
        let mut list = FreeList::<Complex>::default();
        assert_eq!(list.push(Complex::default()), 0);
        assert_eq!(list.first_free, FreeList::<Complex>::SENTINEL);
        assert_eq!(list.capacity(), 1);
    }

    #[test]
    fn after_deletion_has_a_first_free() {
        let mut list = FreeList::<Complex>::default();
        list.push(Complex::default());
        list.erase(0);
        assert_eq!(list.first_free, 0);
        assert_eq!(list.capacity(), 1);
    }

    #[test]
    fn insert_after_delete_has_no_free() {
        let mut list = FreeList::<Complex>::default();
        list.push(Complex::default());
        list.erase(0);
        list.push(Complex::default());
        assert_eq!(list.first_free, FreeList::<Complex>::SENTINEL);
        assert_eq!(list.capacity(), 1);
    }

    #[test]
    fn first_free_points_to_last_freed_index() {
        let mut list = FreeList::<Complex>::default();
        insert_some(&mut list, 2);
        list.erase(0);
        list.erase(1);
        assert_eq!(list.first_free, 1);
        assert_eq!(list.capacity(), 2);
    }

    #[test]
    fn erase_in_ascending_order() {
        let mut list = FreeList::<Complex>::default();
        insert_some(&mut list, 4);
        list.erase(0);
        list.erase(1);
        list.erase(2);
        list.erase(3);
        assert_eq!(list.first_free, 3);
        assert_eq!(list.capacity(), 4);
    }

    #[test]
    fn erase_in_descending_order() {
        let mut list = FreeList::<Complex>::default();
        insert_some(&mut list, 4);
        list.erase(3);
        list.erase(2);
        list.erase(1);
        list.erase(0);
        assert_eq!(list.first_free, 0);
        assert_eq!(list.capacity(), 4);
    }

    #[test]
    fn erase_in_mixed_order() {
        let mut list = FreeList::<Complex>::default();
        insert_some(&mut list, 4);
        list.erase(0);
        list.erase(3);
        list.erase(1);
        list.erase(2);
        assert_eq!(list.first_free, 2);
        assert_eq!(list.capacity(), 4);
    }

    #[test]
    fn clear_works() {
        let mut list = FreeList::<Complex>::default();
        insert_some(&mut list, 4);
        list.erase(1);
        list.clear();
        list.clear();
        assert_eq!(list.first_free, FreeList::<Complex>::SENTINEL);
        assert_eq!(list.capacity(), 0);
    }

    #[test]
    fn is_in_free_list_works() {
        let mut list = FreeList::<Complex>::default();
        insert_some(&mut list, 2);
        list.erase(0);
        assert!(list.debug_is_in_free_list(0));
        assert!(!list.debug_is_in_free_list(1));
    }

    #[test]
    fn at_works() {
        let mut list = FreeList::<Complex>::default();
        list.push(Complex(1., 2.));
        list.push(Complex::default());
        let element = unsafe { list.at(0) };
        assert_eq!(*element, Complex(1., 2.));
    }

    #[test]
    fn at_mut_works() {
        let mut list = FreeList::<Complex>::default();
        list.push(Complex(1., 2.));
        list.push(Complex::default());

        // Mutably access the element and exchange it.
        let element = unsafe { list.at_mut(0) };
        *element = Complex::default();

        // Get a new reference and verify.
        let element = unsafe { list.at(0) };
        assert_eq!(*element, Complex(0., 0.));
    }

    fn insert_some(list: &mut FreeList<Complex>, n: usize) {
        for _ in 0..n {
            list.push(Complex::default());
        }
    }
}
