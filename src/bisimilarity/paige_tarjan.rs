//! Implementation of the [Paige-Tarjan algorithm](https://doi.org/10.1137%2F0216062).
//!
//! * [Reference (German)](https://www8.cs.fau.de/ext/teaching/wise2024-25/CommPar/CommPar.pdf#subsection.2.7)

use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::ops::Deref;
use std::rc::{Rc, Weak};

use super::list::*;

/// State of the algorithm
pub struct PaigeTarjan {
    /// List of composed blocks.
    /// Linked by [`Block::c_ref`]
    c_blocks: RcList<Block>,

    /// Coarse partition R
    /// Linked by [`Block::r_ref`]
    r_blocks: RcList<Block>,
}

/// A state in the underlying [LTS](https://en.wikipedia.org/wiki/Transition_system)
pub struct State {
    /// List of all transitions _into_ this state.
    /// Linked by [`Transition::list_ref`].
    in_transitions: RcList<Transition>,

    /// Mark used to avoid duplicates in the third step
    mark: bool,

    /// Counter
    count: usize,

    /// Block in P that this state is a part of
    block_in_p: Weak<RefCell<Block>>,


    /// Links for list of predecessors
    pred_ref: ListRef<Self>,

    /// Links for [`Block::elements`]
    element_ref: ListRef<Self>,

    /// Links for [`Block::elements`]; Only used by the copied block B'.
    element_copy_ref: ListRef<Self>,
}

/// A transition in the underlying LTS
pub struct Transition {
    /// Link to the [`State`] where the transition originates from.
    lhs: Weak<RefCell<State>>,

    /// Mark used to avoid duplicates in the third step
    mark: bool,

    /// Links for [`State::in_transitions`].
    list_ref: ListRef<Self>
}

/// A block of a partition.
///
/// One [`Block`] may be part of multiple partitions (see it's parameters).
pub struct Block {
    /// List of all [`State`]s contained in the block.
    /// Linked by [`State::element_ref`] or [`State::element_copy_ref`] depending on whether this
    /// is a copy block as created by [`Block::new_as_copy`].
    elements: RcList<State>,

    /// List of all blocks contained in this one (if the block is a composed block).
    /// Linked by [`Block::child_ref`]
    children: RcList<Block>,

    /// Attached block used in the [`PaigeTarjan::split`] step.
    attached: Option<Rc<RefCell<Block>>>,


    /// Links for [`PaigeTarjan::c_blocks`]
    c_ref: ListRef<Block>,

    /// Links for [`PaigeTarjan::r_blocks`]
    r_ref: ListRef<Block>,

    /// Links for split blocks list used in [`PaigeTarjan::split`]
    split_ref: ListRef<Block>,

    /// Links for [`Block::children`]
    child_ref: ListRef<Block>,
}

impl PaigeTarjan {
    /// Refine step of the Paige-Tarjan algorithm
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
                // TODO mark transition?
            }
        }

        // 4.
    }

    /// Split blocks by `divider`.
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
    /// Create new, empty [`Block`]
    fn new() -> Self {
        Block {
            elements: RcList::new(State::element_list_ref, State::element_list_ref_mut),
            children: RcList::new(Block::child_list_ref, Block::child_list_ref_mut),
            attached: None,

            c_ref: ListRef::new(),
            r_ref: ListRef::new(),
            split_ref: ListRef::new(),
            child_ref: ListRef::new(),
        }
    }

    /// Create new [`Block`], which is set up to serve as a copy
    fn new_as_copy() -> Self {
        Block {
            elements: RcList::new(State::element_copy_list_ref, State::element_copy_list_ref_mut),
            children: RcList::new(Block::child_list_ref, Block::child_list_ref_mut),
            attached: None,

            c_ref: ListRef::new(),
            r_ref: ListRef::new(),
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
        self.mark
    }
}
