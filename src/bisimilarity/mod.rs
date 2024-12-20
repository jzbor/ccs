use std::rc::Rc;
use std::time::Duration;

use crate::ccs::*;

mod fixpoint;
mod paige_tarjan;
mod list;

pub type Relation = Vec<(Rc<Process>, Rc<Process>)>;

pub fn bisimulation(system: &CCSSystem, paige_tarjan: bool, collect: bool) -> (Option<Relation>, Duration) {
    if paige_tarjan {
        paige_tarjan::bisimulation(system, collect)
    } else {
        fixpoint::bisimulation(system, collect)
    }
}
