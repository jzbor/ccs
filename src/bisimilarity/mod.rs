use std::rc::Rc;
use std::time::Duration;

use fixpoint::Fixpoint;
use paige_tarjan::PaigeTarjan;

use crate::ccs::*;
use crate::lts::Lts;

mod fixpoint;
mod paige_tarjan;
mod list;

pub type Relation = Vec<(Rc<Process>, Rc<Process>)>;

pub trait BisimulationAlgorithm {
    fn bisimulation(&mut self, collect: bool) -> (Option<Relation>, Duration);
}


pub fn bisimulation(system: &CCSSystem, paige_tarjan: bool, collect: bool) -> (Option<Relation>, Duration) {
    let lts = Lts::new(system, true);
    let mut bsa = bisimulation_algorithm(lts, paige_tarjan);
    bsa.bisimulation(collect)
}

fn bisimulation_algorithm(lts: Lts, paige_tarjan: bool) -> Box<dyn BisimulationAlgorithm> {
    if paige_tarjan {
        Box::new(PaigeTarjan::new_with_labels(lts))
    } else {
        Box::new(Fixpoint::new(lts))
    }
}

