use std::cell::RefCell;
use std::rc::Rc;

pub struct ListItem<T> {
    previous: Option<RcItem<T>>,
    next: Option<RcItem<T>>,
    data: T,
}

pub type RcItem<T> = Rc<RefCell<ListItem<T>>>;

pub struct RcList<T> {
    first: Option<RcItem<T>>,
    last: Option<RcItem<T>>,
    size: usize
}

pub struct RcListIterator<T> {
    current: Option<RcItem<T>>,
}

impl<T> RcList<T> {
    pub fn new() -> Self {
        RcList {
            first: None,
            last: None,
            size: 0,
        }
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn put(&mut self, data: T) {
        let previous_last = self.last.take();
        let item = Rc::new(RefCell::new(ListItem {
            previous: previous_last,
            next: None,
            data
        }));
        self.size += 1;
        self.last = Some(item);
    }

    pub fn remove(&mut self, item: RcItem<T>) -> T {
        let mut item_ref = item.borrow_mut();
        let previous_opt = item_ref.previous.take();
        let next_opt = item_ref.next.take();
        drop(item_ref);
        if let Some(previous) = &previous_opt {
            previous.borrow_mut().next = next_opt.clone();
        }
        if let Some(next) = &next_opt {
            next.borrow_mut().previous = previous_opt;
        }

        self.size -= 1;

        Rc::into_inner(item).unwrap()
            .into_inner().data
    }

    pub fn pop_front(&mut self) -> Option<T> {
        todo!()
    }

    pub fn get(&mut self, i: usize) -> Option<RcItem<T>> {
        todo!()
    }

    pub fn iter(&self) -> RcListIterator<T> {
        RcListIterator { current: self.first.clone() }
    }
}

impl<T> ListItem<T> {
    pub fn data(&self) -> &T {
        &self.data
    }

    pub fn data_mut(&mut self) -> &mut T {
        &mut self.data
    }
}

impl<T> Iterator for RcListIterator<T> {
    type Item = RcItem<T>;

    fn next(&mut self) -> Option<Self::Item> {
        match self.current.take() {
            Some(current) => {
                self.current = current.borrow().next.clone();
                Some(current)
            },
            None => None,
        }
    }
}

impl<T> Clone for RcList<T> {
    fn clone(&self) -> Self {
        // TODO: deep copy
        todo!()
    }
}
