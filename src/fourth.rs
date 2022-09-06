use std::rc::Rc;

// RefCell will enforce the borrowing rules at runtime instead of statically.
// If you break the rules, the program will panic
use std::cell::RefCell;

pub struct List<T> {
    head: Link<T>,
    tail: Link<T>,
}

type Link<T> = Option<Rc<RefCell<Node<T>>>>;

pub struct Node<T> {
    elem: T,
    next: Link<T>,
    prev: Link<T>,
}

impl<T> Node<T> {
    pub fn new(elem: T) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            elem,
            next: None,
            prev: None,
        }))
    }
}

impl<T> List<T> {
    pub fn new() -> Self {
        Self {
            head: None,
            tail: None,
        }
    }

    pub fn push_front(&mut self, elem: T) {
        let new_head = Node::new(elem);

        match self.head.take() {
            Some(old_head) => {
                new_head.borrow_mut().next = Some(old_head.clone());
                old_head.borrow_mut().prev = Some(new_head.clone());
                self.head = Some(new_head);
            },
            None => {
                self.head = Some(new_head.clone());
                self.tail = Some(new_head);
            }
        }
    }
}
