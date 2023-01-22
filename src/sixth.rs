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

    pub fn push_back(&mut self, elem: T) {
        unsafe {
            let new = NonNull::new_unchecked(Box::into_raw(Box::new(Node {
                prev: None,
                next: None,
                elem,
            })));

            if let Some(old) = self.tail {
                (*old.as_ptr()).next = Some(new);
                (*new.as_ptr()).prev = Some(old);
            } else {
                self.head = Some(new);
            }

            self.tail = Some(new);
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

    pub fn pop_back(&mut self) -> Option<T> {
        unsafe {
            self.tail.map(|node| {
                let node = Box::from_raw(node.as_ptr());
                let elem = node.elem;

                self.tail = node.prev;

                if let Some(new) = self.tail {
                    (*new.as_ptr()).next = None;
                } else {
                    // List is now empty
                    self.head = None;
                }

                self.len -= 1;

                elem
            })
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn front(&self) -> Option<&T> {
        unsafe { self.head.map(|node| &(*node.as_ptr()).elem) }
    }

    pub fn back(&self) -> Option<&T> {
        unsafe { self.tail.map(|node| &(*node.as_ptr()).elem) }
    }

    pub fn front_mut(&mut self) -> Option<&mut T> {
        unsafe { self.head.map(|node| &mut (*node.as_ptr()).elem) }
    }

    pub fn back_mut(&self) -> Option<&mut T> {
        unsafe { self.tail.map(|node| &mut (*node.as_ptr()).elem) }
    }
}

impl<T> Drop for LinkedList<T> {
    fn drop(&mut self) {
        while let Some(_) = self.pop_front() {}
    }
}

pub struct Iter<'a, T> {
    head: Link<T>,
    tail: Link<T>,
    len: usize,
    _boo: PhantomData<&'a T>,
}

impl<T> LinkedList<T> {
    pub fn iter(&self) -> Iter<T> {
        Iter {
            head: self.head,
            tail: self.tail,
            len: self.len,
            _boo: PhantomData,
        }
    }
}

impl<'a, T> IntoIterator for &'a LinkedList<T> {
    type IntoIter = Iter<'a, T>;
    type Item = &'a T;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len > 0 {
            self.head.map(|node| unsafe {
                self.len -= 1;
                self.head = (*node.as_ptr()).next;
                &(*node.as_ptr()).elem
            })
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a, T> DoubleEndedIterator for Iter<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.len > 0 {
            self.tail.map(|node| unsafe {
                self.len -= 1;
                self.tail = (*node.as_ptr()).prev;
                &(*node.as_ptr()).elem
            })
        } else {
            None
        }
    }
}

impl<'a, T> ExactSizeIterator for Iter<'a, T> {
    fn len(&self) -> usize {
        self.len
    }
}

pub struct IterMut<'a, T> {
    head: Link<T>,
    tail: Link<T>,
    len: usize,
    _boo: PhantomData<&'a mut T>,
}

impl<T> LinkedList<T> {
    pub fn iter_mut(&self) -> IterMut<T> {
        IterMut {
            head: self.head,
            tail: self.tail,
            len: self.len,
            _boo: PhantomData,
        }
    }
}

impl<'a, T> IntoIterator for &'a mut LinkedList<T> {
    type IntoIter = IterMut<'a, T>;
    type Item = &'a mut T;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len > 0 {
            self.head.map(|node| unsafe {
                self.len -= 1;
                self.head = (*node.as_ptr()).next;
                &mut (*node.as_ptr()).elem
            })
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a, T> DoubleEndedIterator for IterMut<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.len > 0 {
            self.tail.map(|node| unsafe {
                self.len -= 1;
                self.tail = (*node.as_ptr()).prev;
                &mut (*node.as_ptr()).elem
            })
        } else {
            None
        }
    }
}

impl<'a, T> ExactSizeIterator for IterMut<'a, T> {
    fn len(&self) -> usize {
        self.len
    }
}

pub struct IntoIter<T> {
    list: LinkedList<T>,
}

impl<T> LinkedList<T> {
    pub fn into_iter(self) -> IntoIter<T> {
        IntoIter { list: self }
    }
}

impl<T> IntoIterator for LinkedList<T> {
    type Item = T;

    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.into_iter()
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.list.pop_front()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.list.len, Some(self.list.len))
    }
}

impl<T> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.list.pop_back()
    }
}

impl<T> ExactSizeIterator for IntoIter<T> {
    fn len(&self) -> usize {
        self.list.len
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
