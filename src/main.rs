use std::fs;
use std::io;
use std::process;
use std::rc::Rc;

use bisimilarity::bisimulation_algorithm;
use bisimilarity::AlgorithmChoice;
use ccs::CCSSystem;
use ccs::Process;
use clap::Parser;
use clap::ValueEnum;
use error::CCSError;
use error::CCSResult;
use lts::Lts;

mod bisimilarity;
mod ccs;
mod parser;
mod random;
mod lts;
mod error;
#[cfg(test)]
mod tests;

#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    subcommand: Subcommand,
}

#[derive(Clone, Debug, PartialEq, clap::Subcommand)]
enum Subcommand {
    /// Parse and echo the CCS specification
    Parse {
        /// File with CCS specification
        #[clap()]
        file: String,
    },

    /// Print out all traces of the LTS for the given CCS
    Trace {
        /// File with CCS specification
        #[clap()]
        file: String,

        /// Allow duplicates (saves memory)
        #[clap(short, long)]
        allow_duplicates: bool,
    },

    /// Print out all states of the LTS for the given CCS
    States {
        /// File with CCS specification
        #[clap()]
        file: String,

        /// Allow duplicates (saves memory)
        #[clap(short, long)]
        allow_duplicates: bool,
    },

    /// Print or visualize the Labeled Transition System for the given CCS
    Lts {
        /// File with CCS specification
        #[clap()]
        file: String,

        /// Print in dot format for graph visualization
        #[clap(short, long)]
        graph: bool,

        /// Open graph with graphviz in x11 mode
        #[clap(short, long)]
        x11: bool,

        /// Another ccs file to compare
        #[clap(short, long)]
        compare: Option<String>,

        /// Allow duplicates (saves memory)
        #[clap(short, long)]
        allow_duplicates: bool,
    },

    /// Display the syntax tree derived by the parser
    #[clap(hide(true))]
    SyntaxTree {
        /// File with CCS specification
        #[clap()]
        file: String,
    },

    /// Calculate bisimulations and decide bisimilarity
    Bisimilarity {
        /// File with CCS specification
        #[clap()]
        file: String,

        /// Other specification to compare the first to?
        #[clap()]
        other_file: Option<String>,

        /// Bench mark algorithm
        #[clap(short, long)]
        bench: bool,

        /// Calculate and print the corresponding relation
        #[clap(short, long)]
        relation: bool,

        /// Choice of algorithm
        #[clap(short, long)]
        algorithm: ExtendedAlgorithmChoice,
    },

    /// Generate a random LTS and represent it as a parsable CCS spec
    RandomLts {
        /// Number of states in the generated system
        #[clap(short, long)]
        states: usize,

        /// Number of different action labels in the generated system
        #[clap(short, long)]
        actions: usize,

        /// Number of transitions
        #[clap(short, long)]
        transitions: usize,
    }
}

#[derive(Clone, Copy, ValueEnum, PartialEq, Eq, Debug)]
pub enum ExtendedAlgorithmChoice {
    Naive,
    PaigeTarjan,
    Compare,
}


fn parse(file: String) -> CCSResult<()> {
    let contents = fs::read_to_string(&file)
            .map_err(CCSError::file_error)?;
    let system = parser::parse(file, &contents)?;
    println!("{}", system);
    Ok(())
}

fn lts(file: String, compare: Option<String>, graph: bool, x11: bool, allow_duplicates: bool) -> CCSResult<()> {
    let contents = fs::read_to_string(&file)
            .map_err(CCSError::file_error)?;
    let system = parser::parse(file, &contents)?;
    let lts = Lts::new(&system, false);

    let compare_lts_opt = match compare {
        Some(path) => {
            let contents = fs::read_to_string(&path)
                .map_err(CCSError::file_error)?;
            let compare_lts = Lts::new(&parser::parse(path, &contents)?, false);
            Some(compare_lts)
        },
        None => None,
    };

    if graph {
        if let Some(compare_lts) = &compare_lts_opt {
            Lts::visualize_all(&[&lts, compare_lts], &mut io::stdout())?;
        } else {
            lts.visualize(&mut io::stdout())?;
        }
    }

    if x11 {
        let mut cmd = process::Command::new("dot")
            .arg("-Tx11")
            .stdin(process::Stdio::piped())
            .stderr(process::Stdio::inherit())
            .stdout(process::Stdio::inherit())
            .spawn()
            .map_err(|_| CCSError::child_creation("dot".to_string()))?;

        if let Some(compare_lts) = &compare_lts_opt {
            Lts::visualize_all(&[&lts, compare_lts], &mut cmd.stdin.take().unwrap())?;
        } else {
            lts.visualize(&mut cmd.stdin.take().unwrap())?;
        }

        let return_code = cmd.wait()
            .map_err(CCSError::file_error)?
            .code();
        if let Some(x) = return_code {
            if x != 0 {
                return Err(CCSError::child_exited(x));
            }
        }
    }

    if !x11 && !graph {
        for (p, a, q) in lts.transitions(allow_duplicates) {
            println!("{} --{}--> {}", p, a, q);
        }

        if let Some(compare_lts) = compare_lts_opt {
            println!();
            for (p, a, q) in compare_lts.transitions(allow_duplicates) {
                println!("{} --{}--> {}", p, a, q);
            }
        }
    }

    Ok(())
}

fn trace(file: String, allow_duplicates: bool) -> CCSResult<()> {
    let contents = fs::read_to_string(&file)
            .map_err(CCSError::file_error)?;
    let system = parser::parse(file, &contents)?;
    let lts = Lts::new(&system, false);

    for trace in lts.traces(allow_duplicates) {
        let words: Vec<String> = trace.into_iter().map(|s| (*s).clone()).collect();
        println!("{}", words.join(","));
    }

    Ok(())
}

fn states(file: String, allow_duplicates: bool) -> CCSResult<()> {
    let contents = fs::read_to_string(&file)
            .map_err(CCSError::file_error)?;
    let system = parser::parse(file, &contents)?;
    let lts = Lts::new(&system, false);

    for state in lts.states(allow_duplicates) {
        println!("{}", state);
    }

    Ok(())
}

fn syntax_tree(file: String) -> CCSResult<()> {
    let contents = fs::read_to_string(&file)
            .map_err(CCSError::file_error)?;
    println!("{:#?}", parser::first_pass(&contents));
    Ok(())
}

fn random(nstates: usize, nactions: usize, ntransitions: usize) -> CCSResult<()> {
    let lts = random::RandomLts::generate(nstates, nactions, ntransitions);
    println!("{}", lts);
    Ok(())
}

fn compare_bisimulation_algorithms(system: &CCSSystem, relation: bool) {
        let (bisimulation_pt, duration_pt) = bisimilarity::bisimulation(&system, AlgorithmChoice::PaigeTarjan, relation);
        println!("=== PAIGE-TARJAN ===");
        println!("took: {:?}\t", duration_pt);
        if let Some(bisim) = bisimulation_pt {
            println!("size of bisimulation: {:?}", bisim.len());
        }
        println!();

        let (bisimulation_nf, duration_nf) = bisimilarity::bisimulation(&system, AlgorithmChoice::Naive, relation);
        println!("=== NAIVE FIXPOINT ===");
        println!("took: {:?}\t", duration_nf);
        if let Some(bisim) = bisimulation_nf {
            println!("size of bisimulation: {:?}", bisim.len());
        }
        println!();
}

fn bisimilarity(file: String, other_file: Option<String>, algorithm_choice: ExtendedAlgorithmChoice, bench: bool, print_relation: bool) -> CCSResult<()> {
    let (roots, system) = match other_file {
        Some(other_file) => {
            let system1 = CCSSystem::from_file(&file)?;
            let system2 = CCSSystem::from_file(&other_file)?;
            (Some((system1.destinct_process().clone(), system2.destinct_process().clone())), CCSSystem::zip(system1, system2)?)
        },
        None => (None, CCSSystem::from_file(&file)?),
    };

    let collect = print_relation || roots.is_some();

    if algorithm_choice == ExtendedAlgorithmChoice::Compare {
        compare_bisimulation_algorithms(&system, collect);
        return Ok(());
    }

    let lts = Lts::new(&system, true);
    let mut algorithm = bisimulation_algorithm(lts, algorithm_choice.try_into().unwrap());
    let (relation, duration) = algorithm.bisimulation(collect);

    if print_relation {
        let relation = relation.as_ref().unwrap();

        if relation.is_empty() {
            println!("No bisimulation found");
        } else {
            println!("The bisimulation \"=BS=\":");
        }

        for (s, t) in relation {
            println!("  {} \t=BS= \t{}", s, t);
        }
        println!();
    }

    if bench {
        println!("took {:?}", duration);
    }

    if let Some((proc1, proc2)) = roots {
        let expected = (Rc::new(Process::ProcessName(proc1)), Rc::new(Process::ProcessName(proc2)));
        if relation.unwrap().contains(&expected) {
            println!("=> Systems are bisimilar");
        } else {
            println!("=> Systems are NOT bisimilar");
        }
    }

    Ok(())
}

fn main() {
    let args = Args::parse();

    use Subcommand::*;
    let result = match args.subcommand {
        Lts { file, graph, x11, compare, allow_duplicates } => lts(file, compare, graph, x11, allow_duplicates),
        Parse { file } => parse(file),
        States { file, allow_duplicates } => states(file, allow_duplicates),
        SyntaxTree { file } => syntax_tree(file),
        Trace { file, allow_duplicates } => trace(file, allow_duplicates),
        RandomLts { states, actions, transitions } => random(states, actions, transitions),
        Bisimilarity { file, bench, relation, algorithm, other_file } => bisimilarity(file, other_file, algorithm, bench, relation),
    };

    error::resolve(result);
}
