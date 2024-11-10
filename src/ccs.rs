use std::{collections::{HashMap, HashSet}, fmt::Display, rc::Rc};

const TAU: &str = "τ";

pub type ProcessName = Rc<String>;
pub type ActionLabel = Rc<String>;

#[derive(Debug, Clone)]
pub struct CCSSystem {
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
    pub fn new(processes: HashMap<ProcessName, Process>, destinct_process: ProcessName) -> Self {
        CCSSystem { processes, destinct_process }
    }

    pub fn processes(&self) -> &HashMap<ProcessName, Process> {
        &self.processes
    }

    pub fn destinct_process(&self) -> &ProcessName {
        &self.destinct_process
    }
}

impl Process {
    pub fn direct_successors(&self, system: &CCSSystem) -> Result<HashSet<(ActionLabel, Process)>, String> {
        use Process::*;
        match self {
            Deadlock() => Ok(HashSet::new()),
            ProcessName(name) => system.processes().get(name)
                .ok_or(format!("Unable to find process specification for {}", name))?
                .clone()
                .direct_successors(system),
            Action(label, process) => Ok(HashSet::from([ (label.clone(), *process.clone()) ])),
            NonDetChoice(left, right) => Ok(
                left.direct_successors(system)?
                    .union(&right.direct_successors(system)?)
                    .cloned()
                    .collect()
            ),
            Parallel(left, right) => {
                let with_left_succ: HashSet<_> = left.direct_successors(system)?
                    .into_iter()
                    .map(|(action, process)| (action, Parallel(Box::new(process), right.clone())))
                    .collect();
                let with_right_succ: HashSet<_> = right.direct_successors(system)?
                    .into_iter()
                    .map(|(action, process)| (action, Parallel(left.clone(), Box::new(process))))
                    .collect();

                let mut com3_succ = HashSet::new();

                for (a, a_succ) in left.direct_successors(system)? {
                    for (b, b_succ) in right.direct_successors(system)? {
                        if Self::actions_complementary(&a, &b) {
                            com3_succ.insert((TAU.to_owned().into(), Parallel(a_succ.clone().into(), b_succ.clone().into())));
                        }
                    }
                }

                Ok(
                    with_left_succ.union(&with_right_succ)
                        .cloned().collect::<HashSet<_>>()
                        .union(&com3_succ)
                        .cloned().collect()
                )
            },
            Rename(process, b, a) => Ok(
                process.direct_successors(system)?
                    .into_iter()
                    .map(|(label, succ_proc)| if label == *a {
                        (b.to_owned(), Rename(succ_proc.into(), b.clone(), a.clone()))
                    } else {
                            (label.to_owned(), Rename(succ_proc.into(), b.clone(), a.clone()))
                        })
                    .collect()
            ),
            Restriction(process, label) => Ok(
                process.direct_successors(system)?
                    .into_iter()
                    .filter(|(l, _)| l != label && !Self::actions_complementary(l, label))
                    .map(|(l, p)| (l, Restriction(p.into(), label.clone())))
                    .collect()
            ),
        }
    }

    pub fn actions_complementary(a: &ActionLabel, b: &ActionLabel) -> bool {
        **a == format!("{}'", b) || **b == format!("{}'", a)
    }
}

impl Display for Process {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Process::*;
        match self {
            Deadlock() => write!(f, "0"),
            ProcessName(name) => write!(f, "{}", name),
            Action(action, rest) => write!(f, "{}.{}", action, rest),
            NonDetChoice(left, right) => write!(f, "({} + {})", left, right),
            Parallel(left, right) => write!(f, "({} | {})", left, right),
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
