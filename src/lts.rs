use std::io;
use std::collections::{HashMap, HashSet, VecDeque};

use crate::ccs::{ActionLabel, CCSSystem, Process};
use crate::error::CCSResult;

type Transition = (Process, ActionLabel, Process);
type Trace = Vec<ActionLabel>;

pub struct Lts {
    system: CCSSystem,
}

pub struct LtsTransitionIterator<'a> {
    lts: &'a Lts,
    allow_duplicates: bool,
    discovered_states: HashSet<Process>,
    cached_transitions: VecDeque<Transition>,
    undiscovered_states: VecDeque<Process>,
}

pub struct LtsStateIterator<'a> {
    lts: &'a Lts,
    allow_duplicates: bool,
    discovered_states: HashSet<Process>,
    undiscovered_states: VecDeque<Process>,
}

pub struct LtsTraceIterator<'a> {
    lts: &'a Lts,
    allow_duplicates: bool,
    discovered_traces: HashSet<(Trace, Process)>,
    undiscovered_traces: VecDeque<(Trace, Process)>,
    cached_traces: VecDeque<Trace>,
}


impl Lts {
    pub fn new(system: &CCSSystem) -> Self {
        Lts { system: system.clone() }
    }

    pub fn transitions(&self, allow_duplicates: bool) -> LtsTransitionIterator {
        let destinct_process = self.system.destinct_process().clone();
        LtsTransitionIterator {
            lts: self,
            allow_duplicates,
            discovered_states: HashSet::new(),
            cached_transitions: VecDeque::new(),
            undiscovered_states: VecDeque::from([ Process::ProcessName(destinct_process) ]),
        }
    }

    pub fn states(&self, allow_duplicates: bool) -> LtsStateIterator {
        let destinct_process = self.system.destinct_process().clone();
        LtsStateIterator {
            lts: self,
            allow_duplicates,
            discovered_states: HashSet::new(),
            undiscovered_states: VecDeque::from([ Process::ProcessName(destinct_process) ]),
        }
    }

    pub fn traces(&self, allow_duplicates: bool) -> LtsTraceIterator {
        let destinct_process = self.system.destinct_process().clone();
        LtsTraceIterator {
            lts: self,
            allow_duplicates,
            cached_traces: VecDeque::new(),
            discovered_traces: HashSet::new(),
            undiscovered_traces: VecDeque::from([ (Vec::new(), Process::ProcessName(destinct_process)) ]),
        }
    }

    pub fn visualize(&self, f: &mut dyn io::Write) -> CCSResult<()> {
        Self::visualize_all(&[self], f)
    }

    pub fn visualize_all(systems: &[&Lts], f: &mut dyn io::Write) -> CCSResult<()> {
        let mut id_counter = 0;
        let nsystems = systems.len();

        let name_alloc = |process: &Process, counter: &mut usize, map: &mut HashMap<Process, usize>| {
            if let Some(id) = map.get(process) {
                *id
            } else {
                *counter += 1;
                map.insert(process.clone(), *counter);
                *counter
            }
        };

        writeln!(f, "digraph G {{")?;

        for (i, lts) in systems.iter().enumerate() {
            let mut node_ids: HashMap<Process, usize> = HashMap::new();

            if id_counter != 0 {
                writeln!(f)?;
            }

            if nsystems > 1 {
                writeln!(f, "  subgraph cluster{} {{", i)?;
                writeln!(f, "    color=lightgrey")?;
                writeln!(f, "    fontcolor=darkgrey")?;
                writeln!(f, "    margin=20")?;
                writeln!(f, "    label=\"{}\"", lts.system.name())?;
            }

            for (p, a, q) in lts.transitions(false) {
                let p_id = name_alloc(&p, &mut id_counter, &mut node_ids);
                let q_id = name_alloc(&q, &mut id_counter, &mut node_ids);

                writeln!(f, "    node_{} -> node_{} [label=\"{}\"]", p_id, q_id, a)?;
            }

            for (name, id) in node_ids.iter() {
                writeln!(f, "    node_{} [label=\"{}\"]", id, name.to_string().replace("\\", "\\\\"))?;
            }

            if nsystems > 1 {
                writeln!(f, "  }}")?;
            }
        }

        writeln!(f, "}}")?;

        Ok(())
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
            .into_iter()
            .map(|(_, succ)| succ)
            .filter(|s| !self.discovered_states.contains(s) && *s != item);
        self.undiscovered_states.extend(direct_successors);

        let transitions: HashSet<_> = item.direct_successors(&self.lts.system)
            .into_iter()
            .map(|(label, succ)| (item.clone(), label, succ))
            .collect();
        self.cached_transitions.extend(transitions);

        if !self.allow_duplicates {
            self.discovered_states.insert(item);
        }
        self.cached_transitions.pop_front()
            .or_else(|| if !self.undiscovered_states.is_empty() { self.next() } else { None })
    }
}

impl<'a> Iterator for LtsStateIterator<'a> {
    type Item = Process;

    fn next(&mut self) -> Option<Self::Item> {
        let item = match self.undiscovered_states.pop_front() {
            Some(item) => item,
            None => return None,
        };

        let mut direct_successors = item.direct_successors(&self.lts.system)
            .into_iter()
            .map(|(_, succ)| succ)
            .filter(|s| !self.discovered_states.contains(s) && *s != item)
            .collect::<Vec<_>>();
        direct_successors.dedup();
        self.undiscovered_states.extend(direct_successors);

        if !self.allow_duplicates {
            self.discovered_states.insert(item.clone());
        }

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
            .into_iter()
            .map(|(label, succ)| {
                let mut new_trace = item.0.clone();
                new_trace.push(label);
                (new_trace, succ)
            })
            .collect();
        let newly_cached: HashSet<_> = traces.iter()
            .filter(|tp| !self.discovered_traces.contains(tp) && !self.cached_traces.contains(&tp.0))
            .map(|(t, _)| t.clone())
            .collect();
        self.cached_traces.extend(newly_cached);
        self.undiscovered_traces.extend(traces);

        if !self.allow_duplicates {
            self.discovered_traces.insert(item.clone());
        }

        self.cached_traces.pop_front()
            .or_else(|| if !self.undiscovered_traces.is_empty() { self.next() } else { None })
    }
}
