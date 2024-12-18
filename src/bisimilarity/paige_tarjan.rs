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

    /// Fine partition P
    /// Linked by [`Block::p_ref`]
    p_blocks: RcList<Block>,

    /// List of all states
    states: RcList<State>,

    /// List of all transitions
    transitions: RcList<Transition>,
}

/// A state in the underlying [LTS](https://en.wikipedia.org/wiki/Transition_system)
pub struct State {
    /// List of all transitions _into_ this state.
    /// Linked by [`Transition::list_ref`].
    in_transitions: RcList<Transition>,

    /// Mark used to avoid duplicates in the third step
    mark3: bool,

    /// Mark used to avoid duplicates in the fifth step
    mark5: bool,

    /// count(s, B)
    count: Rc<RefCell<usize>>,

    /// Block in P that this state is a part of
    block_in_p: Weak<RefCell<Block>>,

    /// Links for list of predecessors
    pred_ref: ListRef<Self>,

    /// Links for list of limited predecessors
    limpred_ref: ListRef<Self>,

    /// Links for [`Block::elements`]
    element_ref: ListRef<Self>,

    /// Links for [`Block::elements`]; Only used by the copied block B'.
    element_copy_ref: ListRef<Self>,

    /// Links for [`PaigeTarjan::states`]
    all_ref: ListRef<Self>,
}

/// A transition in the underlying LTS
pub struct Transition {
    /// Link to the [`State`] where the transition originates from.
    lhs: Weak<RefCell<State>>,

    /// count(s, S)
    count: Rc<RefCell<usize>>,

    /// Links for [`State::in_transitions`].
    in_ref: ListRef<Self>,

    /// Links for [`PaigeTarjan::transitions`]
    all_ref: ListRef<Self>,
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

    /// Reference to block in R that this block is a part of
    upper_in_r: Option<Weak<RefCell<Block>>>,


    /// Links for [`PaigeTarjan::c_blocks`]
    c_ref: ListRef<Block>,

    /// Links for [`PaigeTarjan::r_blocks`]
    r_ref: ListRef<Block>,

    /// Links for [`PaigeTarjan::p_blocks`]
    p_ref: ListRef<Block>,

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
        let size2 = child2.deref().borrow().elements.len();
        let smaller = if size1 < size2 { child1 } else { child2 };

        // 2. Update R
        let b = divider.deref().borrow_mut().children.remove(smaller);
        let s_prime = Rc::new(RefCell::new(Block::new_containing(b.clone())));
        b.deref().borrow_mut().upper_in_r = Some(Rc::downgrade(&s_prime));
        self.r_blocks.append(s_prime.clone());
        if (*divider).borrow().children.len() > 1 {
            self.c_blocks.append(divider.clone());
        }

        // 3. Calculate Predecessors of B
        let mut b_prime = Block::new_as_copy();
        for s in b.deref().borrow().elements.iter() {
            b_prime.elements.append(s);
        }
        let mut pred_b = RcList::new(State::pred_list_ref, State::pred_list_ref_mut);
        for s_small_prime in (*b).borrow().elements.iter() {
            for trans in s_small_prime.deref().borrow().in_transitions.iter() {
                let lhs_rc = trans.deref().borrow().lhs.clone().upgrade().unwrap();
                let mut lhs = lhs_rc.deref().borrow_mut();
                if lhs.mark3 {
                    continue;
                }
                lhs.mark3 = true;
                *lhs.count.deref().borrow_mut() += 1;
                drop(lhs);
                pred_b.append(lhs_rc);
            }
        }

        // 4. Calculate P' = split(B, P)
        self.split(divider.clone(), pred_b);

        // 5. Calculate <-[B]\<-[S\B]
        let mut limited_pred_b = RcList::new(State::limpred_list_ref, State::limpred_list_ref_mut);
        for s_small_prime in b_prime.elements.iter() {
            for trans in s_small_prime.deref().borrow().in_transitions.iter() {
                let lhs_rc = trans.deref().borrow().lhs.clone().upgrade().unwrap();
                let mut lhs = lhs_rc.deref().borrow_mut();
                let trans_count = *trans.deref().borrow().count.deref().borrow();
                let lhs_count = *lhs.count.deref().borrow();

                if lhs_count == trans_count && !lhs.mark5 {
                    lhs.mark5 = true;
                    drop(lhs);
                    limited_pred_b.append(lhs_rc)
                }
            }
        }


        // 6. Calculate split(S\B, P')
        self.split(divider, limited_pred_b);

        // 7. Update counter and cleanup markers
        for s_small_prime in b_prime.elements.iter() {
            for trans_rc in s_small_prime.deref().borrow().in_transitions.iter() {
                let mut trans = trans_rc.deref().borrow_mut();
                if *trans.count.deref().borrow() > 0 {
                    *trans.count.deref().borrow_mut() -= 1;
                }
                trans.count = trans.lhs.upgrade().unwrap().deref().borrow().count.clone();
            }
        }
        for state in self.states.iter() {
            let mut state = state.deref().borrow_mut();
            state.mark3 = false;
            state.mark5 = false;
        }
    }

    /// Split blocks by `divider`.
    fn split(&mut self, divider: Rc<RefCell<Block>>, pred_b: RcList<State>) {
        // TODO update block_in_r?
        let mut splitblocks = RcList::new(Block::split_list_ref, Block::split_list_ref_mut);
        for s_small in pred_b.iter() {
            let d = s_small.deref().borrow().block_in_p
                .upgrade().unwrap().clone();

            if d.deref().borrow().attached.is_none() {
                let d_prime = Rc::new(RefCell::new(Block::new()));
                d.deref().borrow_mut().attached = Some(d_prime.clone());

                // only append d and d' once
                divider.deref().borrow_mut().children.append(d_prime);
                splitblocks.append(d.clone());
            }

            let d_prime = d.deref().borrow().attached.clone().unwrap();
            let s_small = d.deref().borrow_mut().elements.remove(s_small);
            d_prime.deref().borrow_mut().elements.append(s_small);
        }
        for d in splitblocks.iter() {
            d.deref().borrow_mut().attached = None;
            if d.deref().borrow().elements.empty() {
                self.p_blocks.remove(d.clone());
            } else {
                let s_prime = d.deref().borrow().upper_in_r.as_ref()
                    .unwrap().upgrade().unwrap();
                if s_prime.deref().borrow().children.len() == 2 {
                    self.c_blocks.append(s_prime.clone())
                }
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
            upper_in_r: None,

            c_ref: ListRef::new(),
            r_ref: ListRef::new(),
            p_ref: ListRef::new(),
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
            upper_in_r: None,

            c_ref: ListRef::new(),
            r_ref: ListRef::new(),
            p_ref: ListRef::new(),
            split_ref: ListRef::new(),
            child_ref: ListRef::new(),
        }
    }

    fn new_containing(block: Rc<RefCell<Block>>) -> Self {
        let mut new = Self::new();
        new.children.append(block.clone());

        for state in block.deref().borrow().elements.iter() {
            new.elements.append(state);
        }
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

    fn limpred_list_ref(&self) -> &ListRef<State> {
        &self.borrow().limpred_ref
    }

    fn limpred_list_ref_mut(&mut self) -> &mut ListRef<State> {
        &mut self.borrow_mut().limpred_ref
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

    fn all_list_ref(&self) -> &ListRef<State> {
        &self.borrow().all_ref
    }

    fn all_list_ref_mut(&mut self) -> &mut ListRef<State> {
        &mut self.borrow_mut().all_ref
    }
}

impl Transition {
    fn all_list_ref(&self) -> &ListRef<Transition> {
        &self.borrow().all_ref
    }

    fn all_list_ref_mut(&mut self) -> &mut ListRef<Transition> {
        &mut self.borrow_mut().all_ref
    }
}
