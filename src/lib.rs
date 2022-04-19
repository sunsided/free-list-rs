mod index_type;

use crate::index_type::*;

use std::mem::ManuallyDrop;

/// Provides an indexed free list with constant-time removals from anywhere
/// in the list without invalidating indices.
///
/// ## Safety
/// - The maximum number of elements that can be added to this list is `TIndex::MAX - 1`,
///   e.g. `254` when `TIndex` is substituted with a `u8`.
/// - While the implementation of this type makes heavy use of debug-time assertions, the
///   user must make sure to never add more items to the list than the index type
///   can maintain.
/// - At most `usize::MAX` elements can be stored in this vector.
///
/// ## Type parameters
/// * `T` - The type of the element. Must be trivially constructible and destructible.
/// * `TIndex` - The type of the index; see safety considerations above. "Smaller" types (e.g. `u8`)
///   result in a more memory-efficient representation, while "larger" types (e.g. `usize`) allow
///   for more data to be stored.
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
    /// Creates an empty list.
    ///
    /// ## Example
    /// ```rust
    /// use free_list::FreeList;
    ///
    /// let list = FreeList::<&str, u8>::default();
    /// assert_eq!(list.capacity(), 0);
    /// ```
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
    ///
    /// ## Example
    /// ```rust
    /// use free_list::FreeList;
    ///
    /// let mut list = FreeList::<&str, u8>::default();
    /// assert_eq!(list.push("test"), 0);
    /// assert_eq!(list.capacity(), 1);
    /// ```
    pub fn push(&mut self, element: T) -> TIndex {
        #[cfg(debug_assertions)]
        {
            if self.length >= usize::MAX {
                panic!(
                    "Attempted to insert more elements than can be addressed by the underlying index type ({:?} allowed)",
                    usize::MAX
                );
            }

            if self.length >= unsafe { Self::SENTINEL.into() } - 1 {
                panic!(
                    "Attempted to insert more elements than can be addressed by the provided index type ({:?} allowed)",
                    TIndex::MAX
                );
            }

            self.length += 1;
        }

        return if self.first_free != Self::SENTINEL {
            let index = self.first_free;
            let index_usize = unsafe { index.into() };

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
            unsafe { <TIndex as FromAndIntoUsize>::from(self.data.len() - 1) }
        };
    }

    /// Removes the nth element from the free list.
    ///
    /// ## Example
    /// ```rust
    /// use free_list::FreeList;
    ///
    /// let mut list = FreeList::<&str, u8>::default();
    /// list.push("uses one slot");
    ///
    /// // After erasing the just-inserted element, the capacity stays at
    /// // 1 because the list is not compacted.
    /// list.erase(0);
    /// assert_eq!(list.capacity(), 1);
    ///
    /// // After inserting again, the capacity is still 1 because
    /// // the slot was reused.
    /// list.push("uses the same slot");
    /// assert_eq!(list.capacity(), 1);
    /// ```
    pub fn erase(&mut self, n: TIndex) {
        if self.data.is_empty() {
            return;
        }
        debug_assert!(!self.debug_is_in_free_list(n));

        #[cfg(debug_assertions)]
        debug_assert!(self.length > 0);

        let n_usize = unsafe { n.into() };
        unsafe { ManuallyDrop::drop(&mut self.data[n_usize].element) };
        self.data[n_usize].next = self.first_free;
        self.first_free = n;

        #[cfg(debug_assertions)]
        {
            self.length -= 1;
        }
    }

    /// Removes all elements from the free list.
    /// ## Example
    /// ```rust
    /// use free_list::FreeList;
    ///
    /// let mut list = FreeList::<&str, u8>::default();
    /// list.push("one");
    /// list.push("two");
    ///
    /// list.clear();
    /// assert_eq!(list.capacity(), 0);
    /// ```
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
                    || *free_indexes.last().unwrap()
                        != unsafe { <TIndex as FromAndIntoUsize>::from(i) }
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
    ///
    /// ## Example
    /// ```rust
    /// use free_list::FreeList;
    ///
    /// let mut list = FreeList::<&str, u8>::default();
    /// assert_eq!(list.push("first"), 0);
    /// assert_eq!(list.push("second"), 1);
    ///
    /// let element = unsafe { list.at(0) };
    /// assert_eq!(*element, "first");
    /// ```
    ///
    /// Note that accessing a previously erased item is undefined behavior:
    ///
    /// ```rust
    /// use free_list::FreeList;
    ///
    /// let mut list = FreeList::<&str, u8>::default();
    /// list.push("first");
    ///
    /// // SAFETY: The code below is undefined behavior:
    /// list.erase(0);
    /// // let element = unsafe { list.at(0) };
    /// // assert_eq!(*element, "first");
    /// ```
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
    ///
    /// ## Example
    /// ```rust
    /// use free_list::FreeList;
    ///
    /// let mut list = FreeList::<&str, u8>::default();
    /// assert_eq!(list.push("first"), 0);
    /// assert_eq!(list.push("second"), 1);
    ///
    /// let element = unsafe { list.at_mut(1) };
    /// *element = "danger";
    ///
    /// assert_eq!(unsafe { list.at(1) }, &"danger");
    /// ```
    ///
    /// As with `at()`, accessing a previously erased element is
    /// undefined behavior.
    #[inline]
    pub unsafe fn at_mut(&mut self, index: TIndex) -> &mut T {
        debug_assert_ne!(index, Self::SENTINEL);
        debug_assert!(!self.debug_is_in_free_list(index));
        &mut self.data[index.into()].element
    }

    /// Gets the current capacity of the list.
    ///
    /// ```rust
    /// use free_list::FreeList;
    ///
    /// let mut list = FreeList::<&str, u8>::default();
    ///
    /// // The first elements increase the capacity.
    /// list.push("first");
    /// list.push("second");
    /// assert_eq!(list.capacity(), 2);
    ///
    /// // Erasing elements does not decrease the capacity.
    /// list.erase(0);
    /// list.erase(1);
    /// assert_eq!(list.capacity(), 2);
    ///
    /// // Adding elements after an erase does not increase capacity.
    /// list.push("fourth");
    /// list.push("fifth");
    /// assert_eq!(list.capacity(), 2);
    ///
    /// // Adding more elements increases capacity.
    /// list.push("sixth");
    /// list.push("seventh");
    /// assert_eq!(list.capacity(), 4);
    ///
    /// // Clearing the list frees all resources.
    /// list.clear();
    /// assert_eq!(list.capacity(), 0);
    /// ```
    #[allow(dead_code)]
    pub fn capacity(&self) -> usize {
        self.data.len()
    }

    /// Gets the number of elements in the list.
    #[allow(dead_code)]
    #[cfg(debug_assertions)]
    fn debug_len(&self) -> usize {
        #[cfg(debug_assertions)]
        return self.length;
        #[cfg(not(debug_assertions))]
        unimplemented!()
    }

    #[allow(dead_code, unused_variables)]
    #[cfg(debug_assertions)]
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

    #[test]
    fn size_of_static() {
        // Complex as payload type.
        assert_eq!(std::mem::size_of::<Complex>(), 16);
        assert_eq!(
            std::mem::size_of::<FreeElement<Complex, u8>>(),
            std::mem::size_of::<Complex>()
        );
        assert_eq!(
            std::mem::size_of::<FreeElement<Complex, u16>>(),
            std::mem::size_of::<Complex>()
        );
        assert_eq!(
            std::mem::size_of::<FreeElement<Complex, u32>>(),
            std::mem::size_of::<Complex>()
        );
        assert_eq!(
            std::mem::size_of::<FreeElement<Complex, u64>>(),
            std::mem::size_of::<Complex>()
        );
        assert_eq!(
            std::mem::size_of::<FreeElement<Complex, u128>>(),
            std::mem::size_of::<Complex>()
        );
        assert_eq!(
            std::mem::size_of::<FreeElement<Complex, usize>>(),
            std::mem::size_of::<Complex>()
        );

        // u8 as payload type.
        assert_eq!(
            std::mem::size_of::<FreeElement<u8, u8>>(),
            std::mem::size_of::<u8>()
        );
        assert_eq!(
            std::mem::size_of::<FreeElement<u8, u16>>(),
            std::mem::size_of::<u16>()
        );
        assert_eq!(
            std::mem::size_of::<FreeElement<u8, u32>>(),
            std::mem::size_of::<u32>()
        );
        assert_eq!(
            std::mem::size_of::<FreeElement<u8, u64>>(),
            std::mem::size_of::<u64>()
        );
        assert_eq!(
            std::mem::size_of::<FreeElement<u8, u128>>(),
            std::mem::size_of::<u128>()
        );
        assert_eq!(
            std::mem::size_of::<FreeElement<u8, usize>>(),
            std::mem::size_of::<usize>()
        );
    }

    fn insert_some(list: &mut FreeList<Complex>, n: usize) {
        for _ in 0..n {
            list.push(Complex::default());
        }
    }
}
