use std::ptr;

pub struct List<T> {
    head: Link<T>,
    // If this was a Link (-> Box) we run into issues because
    // Box owns the value, so we'd have to go back to Rc<RefCell> hell
    // which makes .iter() and .iter_mut() a nightmare.
    //
    // An (unsafe) alternative would be to use a raw pointer
    tail: *mut Node<T>,
}

// Mixing smart pointers and raw pointers is a recipe for Undefined Behavior
// because the safe pointers introduce extra constraints that we're not
// obeying with the raw pointers.
//
// With raw pointers, `Option` is not as nice or useful either
type Link<T> = *mut Node<T>;

struct Node<T> {
    elem: T,
    next: Link<T>,
}

impl<T> List<T> {
    pub fn new() -> Self {
        List {
            head: ptr::null_mut(),
            tail: ptr::null_mut(),
        }
    }

    pub fn push(&mut self, elem: T) {
        unsafe {
            // We can use `Box` to allocate memory for us, because the pointer will be
            // properly aligned and non-null
            let new_tail = Box::into_raw(Box::new(Node {
                elem,
                next: ptr::null_mut(),
            }));

            if self.tail.is_null() {
                self.head = new_tail;
            } else {
                (*self.tail).next = new_tail;
            }

            self.tail = new_tail;
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        unsafe {
            if self.head.is_null() {
                return None;
            }

            // Rebuild smart pointer to drop after it goes out of scope
            let old_head = Box::from_raw(self.head);

            self.head = old_head.next;

            if self.head.is_null() {
                self.tail = ptr::null_mut();
            }

            Some(old_head.elem)
        }
    }
}

impl<T> Drop for List<T> {
    fn drop(&mut self) {
        while self.pop().is_some() {}
    }
}

#[cfg(test)]
mod test {
    use super::List;
    #[test]
    fn basics() {
        let mut list = List::new();

        // Check empty list behaves right
        assert_eq!(list.pop(), None);

        // Populate list
        list.push(1);
        list.push(2);
        list.push(3);

        // Check normal removal
        assert_eq!(list.pop(), Some(1));
        assert_eq!(list.pop(), Some(2));

        // Push some more just to make sure nothing's corrupted
        list.push(4);
        list.push(5);

        // Check normal removal
        assert_eq!(list.pop(), Some(3));
        assert_eq!(list.pop(), Some(4));

        // Check exhaustion
        assert_eq!(list.pop(), Some(5));
        assert_eq!(list.pop(), None);

        // Check the exhaustion case fixed the pointer right
        list.push(6);
        list.push(7);

        // Check normal removal
        assert_eq!(list.pop(), Some(6));
        assert_eq!(list.pop(), Some(7));
        assert_eq!(list.pop(), None);
    }
}
