use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;
use std::time::Duration;
use std::time::Instant;

use crate::bisimilarity::Relation;
use crate::ccs::*;
use crate::error::CCSError;
use crate::lts;
use crate::lts::*;

use super::list::ListRef;
use super::list::RcList;
use super::BisimulationAlgorithm;

/// Naive fixpoint implementation for solving bisimilarity
pub struct NaiveFixpoint {
    /// Indicates whether the algorithm has already been run
    done: bool,

    /// Maps all process descriptions to their states
    state_map: HashMap<Process, Rc<RefCell<State>>>,

    /// Relation that is constructed by iterative refinement
    relation: Relation,
}

/// State in the naive fixpoint algorithm
struct State {
    /// Process description that is the source of this state
    desc: Rc<Process>,

    /// Outgoing transitions
    transitions: RcList<Transition>,

    /// List of all states
    all_ref: ListRef<Self>,
}

/// Transition in the naive fixpoint algorithm
struct Transition {
    /// Process description that is the source of this transition
    desc: lts::Transition,

    /// List of all transitions
    trans_ref: ListRef<Self>,
}

impl NaiveFixpoint {
    pub fn new(lts: Lts) -> Self {
        let mut states: HashMap<_, _> = lts.states(false)
            .map(|s| (s.clone(), Rc::new(RefCell::new(State::new(s)))))
            .collect();
        let lts_transitions = lts.transitions(false);

        for (from, label, to) in lts_transitions {
            let trans = Rc::new(RefCell::new(Transition::new((from.clone(), label, to.clone()))));
            states.get_mut(&from).unwrap().deref().deref().borrow_mut().transitions.append(trans.clone());
        }

        let mut all_states = RcList::new(State::all_list_ref, State::all_list_ref_mut);
        for state in states.values() {
            all_states.append(state.clone())
        }

        let relation = Self::init_relation(&all_states);

        NaiveFixpoint {
            state_map: states,
            relation,
            done: false,
        }
    }

    fn refine(&mut self) {
        self.apply_f()
    }

    fn is_in_f(&self, s: Rc<RefCell<State>>, t: Rc<RefCell<State>>) -> bool {
        // check s -a-> s'  ==>  t -a-> t'
        for strans in s.deref().borrow().transitions.iter() {
            let mut t_next = false;
            for ttrans in t.deref().borrow().transitions.iter() {
                if ttrans.deref().borrow().desc.1 == strans.deref().borrow().desc.1
                        && self.relation.contains(&(strans.deref().borrow().desc.2.clone().into(),
                                                    ttrans.deref().borrow().desc.2.clone().into())) {
                    t_next = true
                }
            }
            if !t_next {
                return false;
            }
        }

        // check t -a-> t'  ==>  s -a-> s'
        for ttrans in t.deref().borrow().transitions.iter() {
            let mut s_next = false;
            for strans in s.deref().borrow().transitions.iter() {
                if ttrans.deref().borrow().desc.1 == strans.deref().borrow().desc.1
                        && self.relation.contains(&(ttrans.deref().borrow().desc.2.clone().into(),
                                                    strans.deref().borrow().desc.2.clone().into())){
                    s_next = true
                }
            }
            if !s_next {
                return false;
            }
        }

        true
    }

    fn apply_f(&mut self) {
        self.relation = self.relation.iter()
            .map(|(s, t)| (self.state_map.get(s).unwrap().clone(), self.state_map.get(t).unwrap().clone()))
            .filter(|(s, t)| self.is_in_f(s.clone(), t.clone()))
            .map(|(s, t)| (s.deref().borrow().desc.clone(), t.deref().borrow().desc.clone()))
            .collect()
    }

    fn init_relation(states: &RcList<State>) -> Relation {
        let mut rel = Relation::new();
        for s in states.iter() {
            for t in states.iter() {
                if s.deref().borrow().desc != t.deref().borrow().desc {
                    rel.push((s.deref().borrow().desc.clone(), t.deref().borrow().desc.clone()));
                }
            }
            rel.push((s.deref().borrow().desc.clone(), s.deref().borrow().desc.clone()));
        }

        rel
    }
}

impl BisimulationAlgorithm for NaiveFixpoint {
    fn bisimulation(&mut self, collect: bool) -> (Option<Relation>, Duration) {
        assert!(!self.done);

        let starting = Instant::now();

        let mut last_size = self.relation.len() + 1;
        while self.relation.len() < last_size {
            last_size = self.relation.len();
            self.refine();
        }

        let ending = Instant::now();
        self.done = true;

        if collect {
            (Some(self.relation.clone()), ending - starting)
        } else {
            (None, ending - starting)
        }
    }

    fn check(&mut self, procs: (Rc<Process>, Rc<Process>)) -> crate::error::CCSResult<bool> {
        if !self.done {
            return Err(CCSError::results_not_available())
        }

        Ok(self.relation.contains(&procs))
    }
}

impl State {
    fn new(desc: Process) -> Self {
        State {
            desc: Rc::new(desc),
            transitions: RcList::new(Transition::trans_list_ref, Transition::trans_list_ref_mut),
            all_ref: ListRef::new(),
        }
    }

    fn all_list_ref(&self) -> &ListRef<State> {
        &self.all_ref
    }

    fn all_list_ref_mut(&mut self) -> &mut ListRef<State> {
        &mut self.all_ref
    }
}

impl Transition {
    fn new(desc: lts::Transition) -> Self {
        Transition {
            desc,
            trans_ref: ListRef::new(),
        }
    }

    fn trans_list_ref(&self) -> &ListRef<Transition> {
        &self.trans_ref
    }

    fn trans_list_ref_mut(&mut self) -> &mut ListRef<Transition> {
        &mut self.trans_ref
    }
}
