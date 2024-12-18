use std::fmt::{self, Display};

use rand::thread_rng;
use rand::seq::IteratorRandom;
use rand::seq::SliceRandom;

pub struct RandomLts(Vec<(String, Vec<(String, String)>)>);

fn state_name(n: usize) -> String {
    format!("S{}", n)
}

fn action_name(n: usize) -> String {
    format!("a{}", n)
}

impl RandomLts {
    pub fn generate(nstates: usize, nactions: usize, ntransitions: usize) -> Self {
        let mut graph: Vec<_> = (0..nstates)
            .map(state_name)
            .map(|n| (n, Vec::new()))
            .collect();
        let action_labels: Vec<_> = (0..nactions)
            .map(action_name)
            .collect();
        let mut rng = thread_rng();

        for _ in 0..ntransitions {
            let to = graph.choose(&mut rng).unwrap().0.clone();
            let from_transitions = &mut graph.choose_mut(&mut rng).unwrap().1;
            let label = action_labels.iter().choose(&mut rng).unwrap().clone();
            from_transitions.push((label, to));
        }

        RandomLts(graph)
    }
}

impl Display for RandomLts {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (state, transitions) in self.0.iter() {
            write!(f, "{} = ", state)?;

            if transitions.is_empty() {
                writeln!(f, "0")?;
                continue;
            }

            for (i, (label, dest)) in transitions.into_iter().enumerate() {
                if i == 0 {
                    write!(f, "{}.{}", label, dest)?;
                } else {
                    write!(f, " + {}.{}", label, dest)?;
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

