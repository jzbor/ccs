use std::collections::{HashSet, VecDeque};

use crate::ccs::{ActionLabel, CCSSystem, Process};

type Transition = (Process, ActionLabel, Process);
type Trace = Vec<ActionLabel>;

pub struct Lts {
    system: CCSSystem,
}

pub struct LtsTransitionIterator<'a> {
    lts: &'a Lts,
    discovered_states: HashSet<Process>,
    cached_transitions: VecDeque<Transition>,
    undiscovered_states: VecDeque<Process>,
}

pub struct LtsStateIterator<'a> {
    lts: &'a Lts,
    discovered_states: HashSet<Process>,
    undiscovered_states: VecDeque<Process>,
}

pub struct LtsTraceIterator<'a> {
    lts: &'a Lts,
    undiscovered_traces: VecDeque<(Trace, Process)>,
    cached_traces: VecDeque<Trace>,
}


impl Lts {
    pub fn new(system: &CCSSystem) -> Self {
        Lts { system: system.clone() }
    }

    pub fn transitions(&self) -> LtsTransitionIterator {
        let destinct_process = self.system.destinct_process().clone();
        LtsTransitionIterator {
            lts: self,
            discovered_states: HashSet::new(),
            cached_transitions: VecDeque::new(),
            undiscovered_states: VecDeque::from([ Process::ProcessName(destinct_process) ]),
        }
    }

    pub fn states(&self) -> LtsStateIterator {
        let destinct_process = self.system.destinct_process().clone();
        LtsStateIterator {
            lts: self,
            discovered_states: HashSet::new(),
            undiscovered_states: VecDeque::from([ Process::ProcessName(destinct_process) ]),
        }
    }

    pub fn traces(&self) -> LtsTraceIterator {
        let destinct_process = self.system.destinct_process().clone();
        LtsTraceIterator {
            lts: self,
            cached_traces: VecDeque::new(),
            undiscovered_traces: VecDeque::from([ (Vec::new(), Process::ProcessName(destinct_process)) ]),
        }
    }
}

impl<'a> Iterator for LtsTransitionIterator<'a> {
    type Item = Transition;

    fn next(&mut self) -> Option<Self::Item> {
        let item = match self.cached_transitions.pop_front() {
            Some(trans) => return Some(trans),
            None => match self.undiscovered_states.pop_front() {
                Some(item) => item,
                None => return None,
            },
        };

        let direct_successors = item.direct_successors(&self.lts.system)
            .unwrap()
            .into_iter()
            .map(|(_, succ)| succ)
            .filter(|s| !self.discovered_states.contains(s) && *s != item);
        self.undiscovered_states.extend(direct_successors);

        let transitions: HashSet<_> = item.direct_successors(&self.lts.system)
            .unwrap_or(HashSet::new())  // ignores errors
            .into_iter()
            .map(|(label, succ)| (item.clone(), label, succ))
            .collect();
        self.cached_transitions.extend(transitions);

        self.discovered_states.insert(item);
        self.cached_transitions.pop_front()
    }
}

impl<'a> Iterator for LtsStateIterator<'a> {
    type Item = Process;

    fn next(&mut self) -> Option<Self::Item> {
        let item = match self.undiscovered_states.pop_front() {
            Some(item) => item,
            None => return None,
        };

        let direct_successors = item.direct_successors(&self.lts.system)
            .unwrap()
            .into_iter()
            .map(|(_, succ)| succ);
        self.undiscovered_states.extend(direct_successors);

        self.discovered_states.insert(item.clone());

        Some(item)
    }
}

impl<'a> Iterator for LtsTraceIterator<'a> {
    type Item = Trace;

    fn next(&mut self) -> Option<Self::Item> {
        let item = match self.cached_traces.pop_front() {
            Some(trace) => return Some(trace),
            None => match self.undiscovered_traces.pop_front() {
                Some(item) => item,
                None => return None,
            },
        };

        let traces: HashSet<_> = item.1.direct_successors(&self.lts.system)
            .unwrap()
            .into_iter()
            .map(|(label, succ)| {
                let mut new_trace = item.0.clone();
                new_trace.push(label);
                (new_trace, succ)
            })
            .collect();
        self.cached_traces.extend(traces.iter().map(|(t, _)| t.clone()));
        self.undiscovered_traces.extend(traces);

        self.cached_traces.pop_front()
    }
}
