use free_list::FreeList;

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
    assert_eq!(list.capacity(), 0);
}

#[test]
fn after_insertion_has_no_first_free() {
    let mut list = FreeList::<Complex>::default();
    assert_eq!(list.push(Complex::default()), 0);
    assert_eq!(list.capacity(), 1);
}

#[test]
fn after_deletion_has_a_first_free() {
    let mut list = FreeList::<Complex>::default();
    list.push(Complex::default());
    list.erase(0);
    assert_eq!(list.capacity(), 1);
}

#[test]
fn insert_after_delete_has_no_free() {
    let mut list = FreeList::<Complex>::default();
    list.push(Complex::default());
    list.erase(0);
    list.push(Complex::default());
    assert_eq!(list.capacity(), 1);
}

#[test]
fn first_free_points_to_last_freed_index() {
    let mut list = FreeList::<Complex>::default();
    insert_some(&mut list, 2);
    list.erase(0);
    list.erase(1);
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
    assert_eq!(list.capacity(), 4);
}

#[test]
fn clear_works() {
    let mut list = FreeList::<Complex>::default();
    insert_some(&mut list, 4);
    list.erase(1);
    list.clear();
    list.clear();
    assert_eq!(list.capacity(), 0);
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
