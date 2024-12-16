use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::{Rc, Weak};

use super::list::*;

pub struct PaigeTarjan {
    c_blocks: RcList<Block>,
    r_blocks: RcList<Block>,
}

pub struct State {
    in_transitions: RcList<Transition>,
    mark: bool,
    count: usize,
    block_in_p: Weak<RefCell<Block>>,

    pred_ref: ListRef<Self>,
    element_ref: ListRef<Self>,
    element_copy_ref: ListRef<Self>,
}

pub struct Transition {
    lhs: Weak<RefCell<State>>,
}

type RcBlock = Rc<RefCell<Block>>;

pub struct Block {
    elements: RcList<State>,
    children: RcList<Block>,
    attached: Option<Rc<RefCell<Block>>>,

    c_ref: ListRef<Block>,
    split_ref: ListRef<Block>,
    child_ref: ListRef<Block>,
}

impl PaigeTarjan {
    fn refine(&mut self) {
        // 1. Select Divider
        let divider = self.c_blocks.pop_front().unwrap();
        let child1 = divider.deref().borrow_mut().children.get(0).unwrap();
        let child2 = divider.deref().borrow_mut().children.get(1).unwrap();
        let size1 = child1.deref().borrow().elements.len();
        let size2 = child1.deref().borrow().elements.len();
        let smaller = if size1 < size2 { child1 } else { child2 };

        // 2. Update R
        let b = divider.deref().borrow_mut().children.remove(smaller);
        let s_prime = Block::new_containing(&[b.clone()]);
        self.r_blocks.append_new(s_prime);
        if (*divider).borrow().children.len() > 1 {
            self.c_blocks.append(divider);
        }

        // 3. Calculate Predecessors of B
        let mut b_prime = Block::new_as_copy();
        for s in b.deref().borrow().elements.iter() {
            b_prime.elements.append(s);
        }
        let mut pred_b = RcList::new(State::pred_list_ref, State::pred_list_ref_mut);
        for s_small_prime in (*b).borrow().elements.iter() {
            for trans in s_small_prime.deref().borrow().in_transitions.iter()
                    .filter(|trans| !trans.deref().borrow().is_marked()){
                let lhs = trans.deref().borrow().lhs.clone().upgrade().unwrap();
                lhs.deref().borrow_mut().mark = true;
                lhs.deref().borrow_mut().count += 1;
                pred_b.append(lhs);
            }
        }

        // 4.
    }

    fn split(&mut self, divider: Rc<RefCell<Block>>, pred_b: RcList<State>) {
        let mut splitblocks = RcList::new(Block::split_list_ref, Block::split_list_ref_mut);
        for s in pred_b.iter() {
            let d = s.deref().borrow().block_in_p
                .upgrade().unwrap().clone();

            if d.deref().borrow().attached.is_none() {
                let d_prime = Rc::new(RefCell::new(Block::new()));
                d.deref().borrow_mut().attached = Some(d_prime);
            }

            let d_prime = d.deref().borrow().attached.clone().unwrap();
            let s_small = d.deref().borrow_mut().elements.remove(s);
            d_prime.deref().borrow_mut().elements.append(s_small);

            divider.deref().borrow_mut().children.append(d_prime);
            splitblocks.append(d);
        }
        for d in splitblocks.iter() {
            let d_prime = d.deref().borrow_mut().attached.take();
            if d.deref().borrow().elements.len() == 0 {
                //   v TODO: r or p?
                self.r_blocks.remove(d.clone());
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
            elements: RcList::new(State::element_list_ref, State::element_list_ref_mut),
            children: RcList::new(Block::child_list_ref, Block::child_list_ref_mut),
            attached: None,

            c_ref: ListRef::new(),
            split_ref: ListRef::new(),
            child_ref: ListRef::new(),
        }
    }

    fn new_as_copy() -> Self {
        Block {
            elements: RcList::new(State::element_copy_list_ref, State::element_copy_list_ref_mut),
            children: RcList::new(Block::child_list_ref, Block::child_list_ref_mut),
            attached: None,

            c_ref: ListRef::new(),
            split_ref: ListRef::new(),
            child_ref: ListRef::new(),
        }
    }

    fn new_containing(blocks: &[Rc<RefCell<Block>>]) -> Self {
        let mut new = Self::new();
        for block in blocks {
            new.children.append(block.clone())
        }
        // TODO do we need elements here?
        new
    }

    fn new_copy_states(other: &Self) -> Self {
        let mut new = Self::new();
        new.elements = other.elements.clone(); // TODO: This is a problem!
        new
    }

    fn split_list_ref(&self) -> &ListRef<Block> {
        &self.borrow().split_ref
    }

    fn split_list_ref_mut(&mut self) -> &mut ListRef<Block> {
        &mut self.borrow_mut().split_ref
    }

    fn child_list_ref(&self) -> &ListRef<Block> {
        &self.borrow().child_ref
    }

    fn child_list_ref_mut(&mut self) -> &mut ListRef<Block> {
        &mut self.borrow_mut().child_ref
    }
}

impl State {
    fn pred_list_ref(&self) -> &ListRef<State> {
        &self.borrow().pred_ref
    }

    fn pred_list_ref_mut(&mut self) -> &mut ListRef<State> {
        &mut self.borrow_mut().pred_ref
    }

    fn element_list_ref(&self) -> &ListRef<State> {
        &self.borrow().element_ref
    }

    fn element_list_ref_mut(&mut self) -> &mut ListRef<State> {
        &mut self.borrow_mut().element_ref
    }

    fn element_copy_list_ref(&self) -> &ListRef<State> {
        &self.borrow().element_copy_ref
    }

    fn element_copy_list_ref_mut(&mut self) -> &mut ListRef<State> {
        &mut self.borrow_mut().element_copy_ref
    }
}

impl Transition {
    fn is_marked(&self) -> bool {
        todo!()
    }
}
