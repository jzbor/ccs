use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::{Rc, Weak};

use super::list::{ListItem, RcItem, RcList};

type BlockList = RcList<Rc<RefCell<Block>>>;
type StateList = RcList<Rc<RefCell<State>>>;
type TransitionList = RcList<Rc<RefCell<Transition>>>;

pub struct PaigeTarjan {
    c_blocks: BlockList,
    r_blocks: BlockList,
}

pub struct State {
    in_transitions: RcList<Transition>,
    mark: bool,
    count: usize,
    block_in_p: Weak<RefCell<ListItem<Rc<RefCell<Block>>>>>,
}

pub struct Transition {
    lhs: Weak<RefCell<State>>,
}

#[derive(Clone)]
pub struct Block {
    elements: StateList,
    children: BlockList,
    attached: Option<Rc<RefCell<Block>>>
}

impl PaigeTarjan {
    fn refine(&mut self) {
        // 1.
        let divider = self.c_blocks.pop_front().unwrap();
        let child1 = divider.deref().borrow_mut().children.get(0).unwrap();
        let child2 = divider.deref().borrow_mut().children.get(1).unwrap();
        let data1: Rc<RefCell<Block>> = (*child1).borrow().data().clone();
        let size1 = (*data1).borrow().elements.len();
        let data2: Rc<RefCell<Block>> = (*child2).borrow().data().clone();
        let size2 = (*data2).borrow().elements.len();
        let smaller = if size1 < size2 { child1 } else { child2 };

        // 2.
        let b = divider.deref().borrow_mut().children.remove(smaller);
        let s_prime = Rc::new(RefCell::new(Block::new_containing(&[b.clone()])));
        if (*divider).borrow().children.len() > 1 {
            self.c_blocks.put(divider);
        }

        // 3.
        let b_prime = Rc::new(RefCell::new(Block::new_copy_states(&*((*b).borrow()))));
        let mut pred_b = RcList::new();
        for s_small_prime in (*b).borrow().elements.iter() {
            for trans in s_small_prime.deref().borrow().data().deref().borrow().in_transitions.iter()
                    .filter(|trans| !trans.deref().borrow().data().is_marked()){
                let lhs = trans.deref().borrow().data().borrow().lhs.clone();
                lhs.upgrade().unwrap().deref().borrow_mut().mark = true;
                lhs.upgrade().unwrap().deref().borrow_mut().count += 1;
                pred_b.put(lhs);
            }
        }

        // 4.
    }

    fn split(&mut self, divider: Rc<RefCell<Block>>, pred_b: RcList<Rc<RefCell<State>>>) {
        let mut splitblocks = RcList::new();
        for s in pred_b.iter() {
            let d = s.deref().borrow().data().clone()
                .deref().borrow()
                .deref().borrow().block_in_p
                .upgrade().unwrap().clone();

            if d.deref().borrow().data().deref().borrow().attached.is_none() {
                let d_prime = Rc::new(RefCell::new(Block::new()));
                d.deref().borrow_mut().deref().borrow().data().deref().borrow_mut().attached = Some(d_prime);
            }

            let d_prime = d.deref().borrow().data().deref().borrow().attached.clone().unwrap();
            let s_small = d.deref().borrow_mut().data().deref().borrow_mut().elements.remove(s);
            d_prime.deref().borrow_mut().elements.put(s_small);

            divider.deref().borrow_mut().children.put(d_prime);
            splitblocks.put(d);
        }
        for d in splitblocks.iter() {
            let d_prime = d.deref().borrow().data().deref().borrow().data().deref().borrow_mut().attached.take();
            if d.deref().borrow().data().deref().borrow().data().deref().borrow().elements.len() == 0 {
                //   v TODO: r or p?
                self.r_blocks.remove(d.deref().borrow().data().clone());
                // TODO: delete from upper block?
            } else {
                // TODO take s' in R s.t. D sub S'
            }
        }
    }
}

impl Block {
    fn new() -> Self {
        Block {
            elements: RcList::new(),
            children: RcList::new(),
            attached: None,
        }
    }
    fn new_containing(blocks: &[Rc<RefCell<Block>>]) -> Self {
        let mut children = RcList::new();
        for block in blocks {
            children.put(block.clone())
        }
        // TODO do we need elements here?
        Block { elements: RcList::new(), children }
    }

    fn new_copy_states(other: &Self) -> Self {
        Block { elements: other.elements.clone(), children: RcList::new() }
    }
}

impl Transition {
    fn is_marked(&self) -> bool {
        todo!()
    }
}
