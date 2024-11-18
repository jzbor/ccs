use std::{collections::{HashMap, HashSet, VecDeque}, fmt::Display, rc::Rc};

const TAU: &str = "τ";

pub type ProcessName = Rc<String>;
pub type ActionLabel = Rc<String>;

#[derive(Debug, Clone, PartialEq)]
pub struct CCSSystem {
    name: String,
    processes: HashMap<ProcessName, Process>,
    destinct_process: ProcessName,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum Process {
    Deadlock(),
    #[allow(clippy::enum_variant_names)]
    ProcessName(ProcessName),
    Action(ActionLabel, Box<Self>),
    NonDetChoice(Box<Self>, Box<Self>),
    Parallel(Box<Self>, Box<Self>),
    Rename(Box<Self>, ActionLabel, ActionLabel),
    Restriction(Box<Self>, ActionLabel),
}

impl CCSSystem {
    pub fn new(name: String, processes: HashMap<ProcessName, Process>, destinct_process: ProcessName) -> Self {
        CCSSystem { name, processes, destinct_process }
    }

    pub fn processes(&self) -> &HashMap<ProcessName, Process> {
        &self.processes
    }

    pub fn destinct_process(&self) -> &ProcessName {
        &self.destinct_process
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Process {
    pub fn direct_successors(&self, system: &CCSSystem) -> HashSet<(ActionLabel, Process)> {
        use Process::*;
        match self {
            Deadlock() => HashSet::new(),
            ProcessName(name) => match system.processes().get(name) {
                Some(p) => p.direct_successors(system),
                None => HashSet::new(),
            },
            Action(label, process) => HashSet::from([ (label.clone(), *process.clone()) ]),
            NonDetChoice(left, right) => left.direct_successors(system)
                    .union(&right.direct_successors(system))
                    .cloned()
                    .collect(),
            Parallel(left, right) => {
                let with_left_succ: HashSet<_> = left.direct_successors(system)
                    .into_iter()
                    .map(|(action, process)| (action, Parallel(Box::new(process), right.clone())))
                    .collect();
                let with_right_succ: HashSet<_> = right.direct_successors(system)
                    .into_iter()
                    .map(|(action, process)| (action, Parallel(left.clone(), Box::new(process))))
                    .collect();

                let mut com3_succ = HashSet::new();

                for (a, a_succ) in left.direct_successors(system) {
                    for (b, b_succ) in right.direct_successors(system) {
                        if Self::actions_complementary(&a, &b) {
                            com3_succ.insert((TAU.to_owned().into(), Parallel(a_succ.clone().into(), b_succ.clone().into())));
                        }
                    }
                }

                with_left_succ.union(&with_right_succ)
                    .cloned().collect::<HashSet<_>>()
                    .union(&com3_succ)
                    .cloned().collect()
            },
            Rename(process, b, a) => process.direct_successors(system)
                .into_iter()
                .map(|(label, succ_proc)| if label == *a {
                    (b.to_owned(), Rename(succ_proc.into(), b.clone(), a.clone()))
                } else {
                        (label.to_owned(), Rename(succ_proc.into(), b.clone(), a.clone()))
                    })
                .collect(),
            Restriction(process, label) => process.direct_successors(system)
                .into_iter()
                .filter(|(l, _)| l != label && !Self::actions_complementary(l, label))
                .map(|(l, p)| (l, Restriction(p.into(), label.clone())))
                .collect(),
        }
    }

    pub fn actions_complementary(a: &ActionLabel, b: &ActionLabel) -> bool {
        **a == format!("{}'", b) || **b == format!("{}'", a)
    }

    fn zip_non_det_choice(&self) -> VecDeque<Self> {
        use Process::*;
        match self {
            NonDetChoice(left, right) => {
                let mut vec = right.zip_non_det_choice();
                vec.push_front(*left.clone());
                vec
            }
            _ => VecDeque::from([self.clone()]),
        }
    }

    fn zip_parallel(&self) -> VecDeque<Self> {
        use Process::*;
        match self {
            Parallel(left, right) => {
                let mut vec = right.zip_parallel();
                vec.push_front(*left.clone());
                vec
            }
            _ => VecDeque::from([self.clone()]),
        }
    }
}

impl Display for Process {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Process::*;
        match self {
            Deadlock() => write!(f, "0"),
            ProcessName(name) => write!(f, "{}", name),
            Action(action, rest) => write!(f, "{}.{}", action, rest),
            NonDetChoice(..) => write!(f, "({})", self.zip_non_det_choice().into_iter()
                .map(|p| p.to_string())
                .collect::<Vec<_>>()
                .join(" + ")),
            Parallel(..) => write!(f, "({})", self.zip_parallel().into_iter()
                .map(|p| p.to_string())
                .collect::<Vec<_>>()
                .join(" | ")),
            Rename(process, b, a) => write!(f, "{}[{}/{}]", process, b, a),
            Restriction(process, a) => write!(f, "{}\\{}", process, a),
        }
    }
}

impl Display for CCSSystem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} = {}", self.destinct_process, self.processes.get(&self.destinct_process).unwrap())?;

        for (name, specification) in self.processes.iter().filter(|(n, _)| **n != self.destinct_process) {
            write!(f, "\n{} = {}", name, specification)?;
        }

        Ok(())
    }
}
