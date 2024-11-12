use std::fs;
use std::io;
use std::process;

use ccs::CCSSystem;
use clap::Parser;
use error::CCSError;
use error::CCSResult;
use lts::Lts;

mod ccs;
mod parser;
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
        #[command(flatten)]
        common: CommonArgs,
    },

    /// Print out all traces of the LTS for the given CCS
    Trace {
        #[command(flatten)]
        common: CommonArgs,

        /// Allow duplicates (saves memory)
        #[clap(short, long)]
        allow_duplicates: bool,
    },

    /// Print out all states of the LTS for the given CCS
    States {
        #[command(flatten)]
        common: CommonArgs,

        /// Allow duplicates (saves memory)
        #[clap(short, long)]
        allow_duplicates: bool,
    },

    /// Print or visualize the Labeled Transition System for the given CCS
    Lts {
        #[command(flatten)]
        common: CommonArgs,

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
        #[command(flatten)]
        common: CommonArgs,
    },
}

#[derive(clap::Args, Debug, PartialEq, Clone)]
struct CommonArgs {
    /// File with CCS specification
    // #[clap(global=true, default_value_t = String::from("default.ccs"))]
    #[clap()]
    file: String,
}


impl Subcommand {
    fn common(&self) -> &CommonArgs {
        use Subcommand::*;
        match self {
            Parse { common } => common,
            Trace { common, .. } => common,
            States { common, .. } => common,
            Lts { common, .. } => common,
            SyntaxTree { common } => common,
        }
    }
}


fn parse(system: CCSSystem) -> CCSResult<()> {
    println!("{}", system);
    Ok(())
}

fn lts(system: CCSSystem, compare: Option<String>, graph: bool, x11: bool, allow_duplicates: bool) -> CCSResult<()> {
    let lts = Lts::new(&system);

    let compare_lts_opt = match compare {
        Some(path) => {
            let contents = fs::read_to_string(&path)
                .map_err(CCSError::file_error)?;
            let compare_lts = Lts::new(&parser::parse(path, &contents)?);
            Some(compare_lts)
        },
        None => None,
    };

    if graph {
        if let Some(compare_lts) = &compare_lts_opt {
            Lts::visualize_all(&[&lts, &compare_lts], &mut io::stdout())?;
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
            Lts::visualize_all(&[&lts, &compare_lts], &mut cmd.stdin.take().unwrap())?;
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

fn trace(system: CCSSystem, allow_duplicates: bool) -> CCSResult<()> {
    let lts = Lts::new(&system);

    for trace in lts.traces(allow_duplicates) {
        let words: Vec<String> = trace.into_iter().map(|s| (*s).clone()).collect();
        println!("{}", words.join(","));
    }

    Ok(())
}

fn states(system: CCSSystem, allow_duplicates: bool) -> CCSResult<()> {
    let lts = Lts::new(&system);

    for state in lts.states(allow_duplicates) {
        println!("{}", state);
    }

    Ok(())
}

fn syntax_tree(contents: &str) -> CCSResult<()> {
    println!("{:#?}", parser::first_pass(contents));
    Ok(())
}

fn main() {
    let args = Args::parse();
    let path = args.subcommand.common().file.clone();

    let contents = error::resolve(
        fs::read_to_string(&path)
            .map_err(CCSError::file_error)
    );

    if let Subcommand::SyntaxTree {..} = args.subcommand {
        error::resolve(syntax_tree(&contents));
    }

    let system = match parser::parse(path, &contents) {
        Ok(system) => system,
        Err(e) => {eprintln!("{}", e); process::exit(1) },
    };

    let result = match args.subcommand {
        Subcommand::Lts { graph, x11, compare, allow_duplicates, .. } => lts(system, compare, graph, x11, allow_duplicates),
        Subcommand::Parse {..} => parse(system),
        Subcommand::States { allow_duplicates, .. } => states(system, allow_duplicates),
        Subcommand::SyntaxTree {..} => Ok(()),
        Subcommand::Trace { allow_duplicates, .. } => trace(system, allow_duplicates),
    };

    error::resolve(result);
}
