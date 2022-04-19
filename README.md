# free-list

An indexed free list with constant-time removals from anywhere
in the list without invalidating indices. The underlying implementation
is similar to that of [slotmap](https://github.com/orlp/slotmap) but
doesn't provide generational indexing.

This implementation is meant to be used solely in tightly controlled
environments since it sacrifices indexing safety for performance.

Retrieval of elements is an unsafe operation and the user needs to
ensure that the slot was not previously erased. Accessing an erased
slot results in undefined behavior.

```rust
use free_list::FreeList;

fn example() {
    let mut list = FreeList::<&str, u8>::default();
    assert_eq!(list.push("first"), 0);
    assert_eq!(list.push("second"), 1);

    let element = unsafe { list.at(0) };
    assert_eq!(*element, "first");
}
```

After removal of an item, the list is not compacted and the previously
allocated slot can be immediately reused:

```rust
use free_list::FreeList;

fn example() {
    let mut list = FreeList::<&str, u8>::default();
    list.push("first");

    // After erasing the just-inserted element, the capacity stays at
    // 1 because the list is not compacted.
    list.erase(0);
    assert_eq!(list.capacity(), 1);

    // After inserting again, the capacity is still 1 because
    // the slot was reused.
    list.push("second");
    assert_eq!(list.capacity(), 1);

    // When the list is cleared completely, all slots are freed.
    list.clear();
    assert_eq!(list.capacity(), 0);
}
```
