use std::collections::HashSet;

use crate::ccs::*;

mod fixpoint;

pub type Relation = HashSet<(Process, Process)>;

pub fn bisimulation(system1: &CCSSystem, system2: &CCSSystem) -> Relation {
    fixpoint::bisimulation(system1, system2)
}
