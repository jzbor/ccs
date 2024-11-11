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
    },

    /// Print out all states of the LTS for the given CCS
    States {
        #[command(flatten)]
        common: CommonArgs,
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
            Trace { common } => common,
            States { common } => common,
            Lts { common, .. } => common,
            SyntaxTree { common } => common,
        }
    }
}


fn parse(system: CCSSystem) -> CCSResult<()> {
    println!("{}", system);
    Ok(())
}

fn lts(system: CCSSystem, graph: bool, x11: bool) -> CCSResult<()> {
    let lts = Lts::new(&system);

    if graph {
        lts.visualize(&mut io::stdout())?
    } else if x11 {
        let mut cmd = process::Command::new("dot")
            .arg("-Tx11")
            .stdin(process::Stdio::piped())
            .stderr(process::Stdio::inherit())
            .stdout(process::Stdio::inherit())
            .spawn()
            .map_err(|_| CCSError::child_creation("dot".to_string()))?;
        lts.visualize(&mut cmd.stdin.take().unwrap())?;

        let return_code = cmd.wait()
            .map_err(CCSError::file_error)?
            .code();
        if let Some(x) = return_code {
            if x != 0 {
                return Err(CCSError::child_exited(x));
            }
        }
    } else {
        for (p, a, q) in lts.transitions() {
            println!("{} --{}--> {}", p, a, q);
        }
    }

    Ok(())
}

fn trace(system: CCSSystem) -> CCSResult<()> {
    let lts = Lts::new(&system);

    for trace in lts.traces() {
        let words: Vec<String> = trace.into_iter().map(|s| (*s).clone()).collect();
        println!("{}", words.join(","));
    }

    Ok(())
}

fn states(system: CCSSystem) -> CCSResult<()> {
    let lts = Lts::new(&system);

    for state in lts.states() {
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

    let contents = error::resolve(
        fs::read_to_string(&args.subcommand.common().file)
            .map_err(CCSError::file_error)
    );

    if let Subcommand::SyntaxTree {..} = args.subcommand {
        error::resolve(syntax_tree(&contents));
    }

    let system = match parser::parse(&contents) {
        Ok(system) => system,
        Err(e) => {eprintln!("{}", e); process::exit(1) },
    };

    let result = match args.subcommand {
        Subcommand::Lts { graph, x11, .. } => lts(system, graph, x11),
        Subcommand::Parse {..} => parse(system),
        Subcommand::States {..} => states(system),
        Subcommand::SyntaxTree {..} => Ok(()),
        Subcommand::Trace {..} => trace(system),
    };

    error::resolve(result);
}
