use std::{borrow::BorrowMut, cell::RefCell, ops::Deref, rc::Rc};

#[derive(Clone)]
pub struct ListRef<T> {
    next: Option<Rc<RefCell<T>>>,
    prev: Option<Rc<RefCell<T>>>,
}

type GetRef<T> = fn(&T) -> &ListRef<T>;
type GetRefMut<T> = fn(&mut T) -> &mut ListRef<T>;

pub struct RcList<T> {
    head: Option<Rc<RefCell<T>>>,
    tail: Option<Rc<RefCell<T>>>,
    get_ref: GetRef<T>,
    get_ref_mut: GetRefMut<T>,
    size: usize,
}

pub struct RcListIterator<T> {
    current: Option<Rc<RefCell<T>>>,
    get_ref: GetRef<T>,
}

impl<T> RcList<T> {
    pub fn new(get_ref: GetRef<T>, get_ref_mut: GetRefMut<T>) -> Self {
        RcList { head: None, tail: None, size: 0, get_ref, get_ref_mut }
    }

    fn set_next(&self, e: Rc<RefCell<T>>, next: Option<Rc<RefCell<T>>>) {
        let mut mut_e = e.deref().borrow_mut();
        let e_ref = (self.get_ref_mut)(&mut mut_e);
        e_ref.next = next;
    }

    fn set_prev(&self, e: Rc<RefCell<T>>, prev: Option<Rc<RefCell<T>>>) {
        let mut mut_e = e.deref().borrow_mut();
        let e_ref = (self.get_ref_mut)(&mut mut_e);
        e_ref.prev = prev;
    }

    fn take_next(&self, e: Rc<RefCell<T>>) -> Option<Rc<RefCell<T>>> {
        let mut mut_e = e.deref().borrow_mut();
        let e_ref = (self.get_ref_mut)(&mut mut_e);
        e_ref.next.take()
    }

    fn take_prev(&self, e: Rc<RefCell<T>>) -> Option<Rc<RefCell<T>>> {
        let mut mut_e = e.deref().borrow_mut();
        let e_ref = (self.get_ref_mut)(&mut mut_e);
        e_ref.prev.take()
    }

    fn get_next(&self, e: Rc<RefCell<T>>) -> Option<Rc<RefCell<T>>> {
        let borrow_e = e.deref().borrow();
        let e_ref = (self.get_ref)(&borrow_e);
        e_ref.next.clone()
    }

    fn get_prev(&self, e: Rc<RefCell<T>>) -> Option<Rc<RefCell<T>>> {
        let borrow_e = e.deref().borrow();
        let e_ref = (self.get_ref)(&borrow_e);
        e_ref.prev.clone()
    }

    pub fn append(&mut self, e: Rc<RefCell<T>>) {
        let prev_tail = self.tail.take();
        self.tail = Some(e.clone());
        if let Some(prev_tail) = prev_tail.clone() {
            self.set_next(prev_tail, Some(e.clone()))
        } else {
            self.head = Some(e.clone());
        }
        self.set_next(e.clone(), None);
        self.set_prev(e, prev_tail);
        self.size += 1;
    }

    pub fn append_new(&mut self, e: T) {
        self.append(Rc::new(RefCell::new(e)))
    }

    pub fn remove(&mut self, e: Rc<RefCell<T>>) -> Rc<RefCell<T>> {
        let next_opt = self.take_next(e.clone());
        let prev_opt = self.take_prev(e.clone());
        assert!(next_opt.is_some() || prev_opt.is_some() || self.len() == 1);

        match next_opt {
            Some(next) => match prev_opt {
                Some(prev) => {
                    self.set_next(prev.clone(), Some(next.clone()));
                    self.set_prev(next, Some(prev));
                },
                None => {
                    self.head = Some(next.clone());
                    self.set_prev(next, None);
                },
            },
            None => match prev_opt {
                Some(prev) => {
                    self.tail = Some(prev.clone());
                    self.set_next(prev, None);
                },
                None => {
                    self.tail = None;
                    self.head = None;
                },
            }
        }

        self.size -= 1;
        e
    }

    pub fn get(&mut self, i: usize) -> Option<Rc<RefCell<T>>> {
        let mut elem = self.head.clone();
        for j in 0..i {
            elem = elem.map(|e| self.get_next(e)).flatten();
        }
        elem
    }

    pub fn pop_front(&mut self) -> Option<Rc<RefCell<T>>> {
        let front = self.peek_front();
        front.map(|f| { self.remove(f.clone()); f })
    }

    pub fn peek_front(&mut self) -> Option<Rc<RefCell<T>>> {
        self.head.clone()
    }

    pub fn iter(&self) -> RcListIterator<T> {
        RcListIterator { current: self.head.clone(), get_ref: self.get_ref }
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn empty(&self) -> bool {
        self.size == 0
    }
}

impl<T> ListRef<T> {
    pub fn new() -> Self {
        ListRef {
            next: None,
            prev: None,
        }
    }
}

impl<T> Iterator for RcListIterator<T> {
    type Item = Rc<RefCell<T>>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.current.take() {
            Some(current) => {
                let mut cur_borrow = current.deref().borrow_mut();
                let cur_ref = (self.get_ref)(&mut cur_borrow);
                self.current = cur_ref.next.clone();
                drop(cur_borrow);
                Some(current)
            },
            None => None,
        }
    }
}
