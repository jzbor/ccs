use std::collections::HashSet;

use crate::ccs::*;
use crate::lts::*;

type Relation = HashSet<(Process, Process)>;

fn cross_states(system1: &CCSSystem, system2: &CCSSystem) -> Relation {
    let mut set = HashSet::new();

    for x in Lts::new(system1).states(false) {
        for y in Lts::new(system2).states(false) {
            set.insert((x.clone(), y.clone()));
        }
    }

    set
}

fn is_in_F(system1: &CCSSystem, system2: &CCSSystem, s: &Process, t: &Process, r: &Relation) -> bool {
    // check s -a-> s'  ==>  t -a-> t'
    for (a, s_next) in s.direct_successors(system1) {
        if let Some((_, t_next)) = t.direct_successors(system2).into_iter().filter(|(l, _)| *l == a).next() {
            if !r.contains(&(s_next, t_next)) {
                return false;
            }
        } else {
            return false;
        }
    }

    // check t -a-> t'  ==>  s -a-> s'
    for (a, t_next) in t.direct_successors(system2) {
        if let Some((_, s_next)) = s.direct_successors(system1).into_iter().filter(|(l, _)| *l == a).next() {
            if !r.contains(&(s_next, t_next)) {
                return false;
            }
        } else {
            return false;
        }
    }

    true
}

fn apply_F(system1: &CCSSystem, system2: &CCSSystem, r: Relation) -> Relation {
    r.clone().into_iter()
        .filter(|(s, t)| is_in_F(system1, system2, s, t, &r))
        .collect()
}

pub fn bisimulation(system1: &CCSSystem, system2: &CCSSystem) -> Relation {
    let mut r = cross_states(system1, system2);
    while r != apply_F(system1, system2, r.clone()) {
        r = apply_F(system1, system2, r.clone());
    }

    r
}

