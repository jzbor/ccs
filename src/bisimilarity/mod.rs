use std::{collections::HashSet, time::Duration};

use crate::ccs::*;

mod fixpoint;
mod paige_tarjan;
mod list;

pub type Relation = HashSet<(Process, Process)>;

pub fn bisimulation(system: &CCSSystem, paige_tarjan: bool) -> (Relation, Duration) {
    if paige_tarjan {
        paige_tarjan::bisimulation(system)
    } else {
        fixpoint::bisimulation(system)
    }
}
