use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::collections::HashSet;
use std::ops::Deref;
use std::rc::Rc;
use std::time::Duration;
use std::time::Instant;

use crate::bisimilarity::Relation;
use crate::ccs::*;
use crate::lts;
use crate::lts::*;

use super::list::ListRef;
use super::list::RcList;

struct Fixpoint {
    states: RcList<State>,
    state_map: HashMap<Process, Rc<RefCell<State>>>,
    relation: Relation,

}

struct State {
    desc: Process,
    transitions: RcList<Transition>,

    all_ref: ListRef<Self>,
}

struct Transition {
    desc: lts::Transition,

    trans_ref: ListRef<Self>,
}

impl Fixpoint {
    fn new(lts: Lts) -> Self {
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

        Fixpoint {
            states: all_states,
            state_map: states,
            relation,
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
                        && self.relation.contains(&(strans.deref().borrow().desc.2.clone(), ttrans.deref().borrow().desc.2.clone())){
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
                        && self.relation.contains(&(ttrans.deref().borrow().desc.2.clone(), strans.deref().borrow().desc.2.clone())){
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
        self.relation = self.relation.clone().into_iter()
            .map(|(s, t)| (self.state_map.get(&s).unwrap().clone(), self.state_map.get(&t).unwrap().clone()))
            .filter(|(s, t)| self.is_in_f(s.clone(), t.clone()))
            .map(|(s, t)| (s.deref().borrow().desc.clone(), t.deref().borrow().desc.clone()))
            .collect()
    }

    fn init_relation(states: &RcList<State>) -> Relation{
        let mut rel = Relation::new();
        for s in states.iter() {
            for t in states.iter() {
                rel.insert((s.deref().borrow().desc.clone(), t.deref().borrow().desc.clone()));
            }
        }

        rel
    }

}

fn cross_states(system1: &CCSSystem, system2: &CCSSystem) -> Relation {
    let mut set = HashSet::new();

    for x in Lts::new(system1).states(false) {
        for y in Lts::new(system2).states(false) {
            set.insert((x.clone(), y.clone()));
        }
    }

    set
}

impl State {
    fn new(desc: Process) -> Self {
        State {
            desc,
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

pub fn bisimulation(system: &CCSSystem) -> (Relation, Duration) {
    let lts = Lts::new(system);
    let mut fix = Fixpoint::new(lts);

    let starting = Instant::now();

    let mut last_size = fix.relation.len() + 1;
    while fix.relation.len() < last_size {
        last_size = fix.relation.len();
        fix.refine();
    }

    let ending = Instant::now();

    (fix.relation, ending - starting)
}
