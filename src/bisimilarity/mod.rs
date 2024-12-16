use std::collections::HashSet;

use crate::ccs::*;

mod fixpoint;
mod paige_tarjan;
mod list;

pub type Relation = HashSet<(Process, Process)>;

pub fn bisimulation(system1: &CCSSystem, system2: &CCSSystem) -> Relation {
    fixpoint::bisimulation(system1, system2)
}
