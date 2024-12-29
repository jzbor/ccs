use std::{collections::{HashMap, HashSet, VecDeque}, fmt::Display, fs, rc::Rc};

use crate::{error::{self, CCSError, CCSResult}, parser};

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

    pub fn from_file(path: &str) -> CCSResult<Self> {
        let contents = error::resolve(
            fs::read_to_string(path)
                .map_err(CCSError::file_error)
        );
        parser::parse(path.to_owned(), &contents)
    }

    pub fn zip(system1: Self, system2: Self) -> CCSResult<Self> {
        for proc in system1.processes.keys() {
            if system2.processes.contains_key(proc) {
                return Err(CCSError::overlapping_process_error(proc.clone()))
            }
        }

        let destinct_process = system1.destinct_process.clone();
        let name = format!("{}+{}", system1.name, system2.name);
        let mut processes = system1.processes;
        processes.extend(system2.processes);

        Ok(CCSSystem { name, processes, destinct_process })
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
        let mut set = HashSet::new();
        self.direct_successors_helper(system, &mut set);
        set
    }

    fn direct_successors_helper(&self, system: &CCSSystem, set: &mut HashSet<(ActionLabel, Process)>) {
        use Process::*;
        match self {
            Deadlock() => (),
            ProcessName(name) => if let Some(p) = system.processes().get(name) { p.direct_successors_helper(system, set) },
            Action(label, process) => { set.insert((label.clone(), *process.clone())); },
            NonDetChoice(left, right) => {
                left.direct_successors_helper(system, set);
                right.direct_successors_helper(system, set);
            },
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

                set.extend(with_left_succ);
                set.extend(with_right_succ);
                set.extend(com3_succ);
            },
            Rename(process, b, a) => set.extend(
                process.direct_successors(system)
                    .into_iter()
                    .map(|(label, succ_proc)| if label == *a {
                            (b.to_owned(), Rename(succ_proc.into(), b.clone(), a.clone()))
                        } else {
                                (label.to_owned(), Rename(succ_proc.into(), b.clone(), a.clone()))
                        }
                    )
            ),
            Restriction(process, label) => set.extend(
                process.direct_successors(system)
                    .into_iter()
                    .filter(|(l, _)| l != label && !Self::actions_complementary(l, label))
                    .map(|(l, p)| (l, Restriction(p.into(), label.clone())))
            ),
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
