use std::collections::HashMap;
use std::fs;
use std::process;

use ccs::CCSSystem;
use ccs::Process;
use clap::Parser;
use error::CCSError;
use error::CCSResult;
use lts::Lts;

mod ccs;
mod parser;
mod lts;
mod error;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[clap(subcommand)]
    subcommand: Subcommand,

    /// File with CCS specification
    #[clap(global=true, default_value_t = String::from("default.ccs"))]
    file: String,
}

#[derive(Clone, Debug, PartialEq, clap::Subcommand)]
enum Subcommand {
    /// Parse and echo the CCS specification
    Parse {},

    /// Print out all traces of the LTS for the given CCS
    Trace {},

    /// Print or visualize the LTS for the given CCS
    Lts {
        #[clap(short, long)]
        dot: bool,
    },

    /// Display the syntax tree derived by the parser
    #[clap(hide(true))]
    SyntaxTree {},
}

fn parse(system: CCSSystem) -> CCSResult<()> {
    println!("{}", system);
    Ok(())
}

fn lts(system: CCSSystem, dot: bool) -> CCSResult<()> {
    let lts = Lts::new(&system);
    if !dot {
        for (p, a, q) in lts.transitions() {
            println!("{} --{}--> {}", p, a, q);
        }
    } else {
        let mut node_ids: HashMap<Process, usize> = HashMap::new();
        let mut id_counter = 0;

        let name_alloc = |process: &Process, counter: &mut usize, map: &mut HashMap<Process, usize>| {
            if let Some(id) = map.get(&process) {
                id.clone()
            } else {
                *counter += 1;
                map.insert(process.clone(), *counter);
                *counter
            }
        };

        println!("digraph G {{");
        for (p, a, q) in lts.transitions() {
            let p_id = name_alloc(&p, &mut id_counter, &mut node_ids);
            let q_id = name_alloc(&q, &mut id_counter, &mut node_ids);

            println!("  node_{} -> node_{} [label=\"{}\"]", p_id, q_id, a);
        }
        for (name, id) in node_ids.iter() {
            println!("  node_{} [label=\"{}\"]", id, name.to_string().replace("\\", "\\\\"));
        }
        println!("}}");
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

fn syntax_tree(contents: &str) -> CCSResult<()> {
    println!("{:#?}", parser::first_pass(contents));
    Ok(())
}


fn main() {
    let args = Args::parse();

    let contents = error::resolve(
        fs::read_to_string(args.file)
            .map_err(CCSError::file_error)
    );

    if let Subcommand::SyntaxTree {} = args.subcommand {
        error::resolve(syntax_tree(&contents));
    }

    let system = match parser::parse(&contents) {
        Ok(system) => system,
        Err(e) => {eprintln!("{}", e); process::exit(1) },
    };

    let result = match args.subcommand {
        Subcommand::Parse {} => parse(system),
        Subcommand::Lts { dot } => lts(system, dot),
        Subcommand::Trace {} => trace(system),
        Subcommand::SyntaxTree {} => Ok(()),
    };

    error::resolve(result);
}
