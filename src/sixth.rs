use std::marker::PhantomData;
use std::ptr::NonNull;

// NOTE: Lifetime subtyping and variance
// - `mut` references are invariant with T, otherwise T could be mutated into a different type
//
// NonNull makes our pointers covariant (mut references are invariant with T)
type Link<T> = Option<NonNull<Node<T>>>;

struct Node<T> {
    prev: Link<T>,
    next: Link<T>,
    elem: T,
}

pub struct LinkedList<T> {
    head: Link<T>,
    tail: Link<T>,
    len: usize,
    // We semantically store values of T by-value (necessary when doing unsafe stuff)
    //
    // PhantomData is a way to give the compiler information about us
    // owning a type of data but, for reasons, it looks like it doesn't.
    //
    // This way we're telling the compiler that we own T, so the drop checker
    // doesn't let go of T. You should do this when using raw pointers.
    _boo: PhantomData<T>,
}

impl<T> LinkedList<T> {
    pub fn new() -> Self {
        Self {
            head: None,
            tail: None,
            len: 0,
            _boo: PhantomData,
        }
    }

    pub fn push_front(&mut self, elem: T) {
        unsafe {
            let new = NonNull::new_unchecked(Box::into_raw(Box::new(Node {
                prev: None,
                next: None,
                elem,
            })));

            if let Some(old) = self.head {
                // With NonNull, you can't directly dereference the pointer
                // You have to use .as_ptr()
                (*old.as_ptr()).prev = Some(new);
                (*new.as_ptr()).next = Some(old);
            } else {
                self.tail = Some(new);
            }

            self.head = Some(new);
            self.len += 1;
        }
    }

    pub fn pop_front(&mut self) -> Option<T> {
        unsafe {
            self.head.map(|node| {
                let node = Box::from_raw(node.as_ptr());
                let elem = node.elem;

                self.head = node.next;

                if let Some(new) = self.head {
                    (*new.as_ptr()).prev = None;
                } else {
                    // List is now empty
                    self.tail = None;
                }

                self.len -= 1;

                elem
            })
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }
}

impl<T> Drop for LinkedList<T> {
    fn drop(&mut self) {
        while let Some(_) = self.pop_front() {}
    }
}

#[cfg(test)]
mod test {
    use super::LinkedList;

    #[test]
    fn test_basic_front() {
        let mut list = LinkedList::new();

        // Try to break an empty list
        assert_eq!(list.len(), 0);
        assert_eq!(list.pop_front(), None);
        assert_eq!(list.len(), 0);

        // Try to break a one item list
        list.push_front(10);
        assert_eq!(list.len(), 1);
        assert_eq!(list.pop_front(), Some(10));
        assert_eq!(list.len(), 0);
        assert_eq!(list.pop_front(), None);
        assert_eq!(list.len(), 0);

        // Mess around
        list.push_front(10);
        assert_eq!(list.len(), 1);
        list.push_front(20);
        assert_eq!(list.len(), 2);
        list.push_front(30);
        assert_eq!(list.len(), 3);
        assert_eq!(list.pop_front(), Some(30));
        assert_eq!(list.len(), 2);
        list.push_front(40);
        assert_eq!(list.len(), 3);
        assert_eq!(list.pop_front(), Some(40));
        assert_eq!(list.len(), 2);
        assert_eq!(list.pop_front(), Some(20));
        assert_eq!(list.len(), 1);
        assert_eq!(list.pop_front(), Some(10));
        assert_eq!(list.len(), 0);
        assert_eq!(list.pop_front(), None);
        assert_eq!(list.len(), 0);
        assert_eq!(list.pop_front(), None);
        assert_eq!(list.len(), 0);
    }
}
