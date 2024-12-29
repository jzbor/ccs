use std::rc::Rc;
use std::time::Duration;

use naive::NaiveFixpoint;
use paige_tarjan::PaigeTarjan;

use crate::error::CCSResult;
use crate::{ccs::*, ExtendedAlgorithmChoice};
use crate::lts::Lts;

mod naive;
mod paige_tarjan;
mod list;

pub type Relation = Vec<(Rc<Process>, Rc<Process>)>;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum AlgorithmChoice {
    Naive,
    PaigeTarjan,
}

pub trait BisimulationAlgorithm {
    fn bisimulation(&mut self, collect: bool) -> (Option<Relation>, Duration);
    fn check(&mut self, procs: (Rc<Process>, Rc<Process>)) -> CCSResult<bool>;
}


pub fn bisimulation(system: &CCSSystem, algorithm: AlgorithmChoice, collect: bool) -> (Option<Relation>, Duration) {
    let lts = Lts::new(system, true);
    let mut bsa = bisimulation_algorithm(lts, algorithm);
    bsa.bisimulation(collect)
}

pub fn bisimulation_algorithm(lts: Lts, algorithm: AlgorithmChoice) -> Box<dyn BisimulationAlgorithm> {
    match algorithm {
        AlgorithmChoice::Naive => Box::new(NaiveFixpoint::new(lts)),
        AlgorithmChoice::PaigeTarjan => Box::new(PaigeTarjan::new_with_labels(lts)),
    }
}

impl TryFrom<ExtendedAlgorithmChoice> for AlgorithmChoice {
    type Error = ();

    fn try_from(value: ExtendedAlgorithmChoice) -> Result<Self, Self::Error> {
        use ExtendedAlgorithmChoice::*;
        match value {
            Naive => Ok(Self::Naive),
            PaigeTarjan => Ok(Self::PaigeTarjan),
            Compare => Err(()),
        }
    }
}
