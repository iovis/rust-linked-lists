use std::marker::PhantomData;
use std::ptr::NonNull;

pub struct LinkedList<T> {
    front: Link<T>,
    back: Link<T>,
    len: usize,
    // We semantically store values of T by-value (necessary when doing unsafe stuff)
    // PhantomData is a way to give the compiler information about us
    // owning a type of data but, for reasons, it looks like it doesn't
    _boo: PhantomData<T>,
}

// (Lifetime subtyping note)
// NonNull makes our pointers covariant (mut references are invariant with T)
type Link<T> = Option<NonNull<Node<T>>>;

struct Node<T> {
    front: Link<T>,
    back: Link<T>,
    elem: T,
}
