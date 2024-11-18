use std::collections::HashMap;
use std::rc::Rc;

use pest::{iterators::Pair, Parser};
use pest_derive::Parser;

use crate::ccs::*;
use crate::ccs::Process;
use crate::error::{CCSError, CCSResult};

#[derive(Parser)]
#[grammar = "grammar.pest"]
struct CCSParser;

fn parse_process(pair: Pair<Rule>) -> CCSResult<Process> {
    match pair.as_rule() {
        Rule::deadlock => Ok(Process::Deadlock()),
        Rule::process => parse_process(pair.into_inner().next().unwrap()),
        Rule::action => {
            let mut inner = pair.into_inner();
            let action = inner.next().unwrap().as_span().as_str().to_owned();
            let process = Box::new(parse_process(inner.next().unwrap())?);

            if action == "tau" {
                Ok(Process::Action(String::from("τ").into(), process))
            } else {
                Ok(Process::Action(action.into(), process))
            }
        },
        Rule::summation => {
            let mut inner = pair.into_inner();
            let left = Box::new(parse_process(inner.next().unwrap())?);
            let right = Box::new(parse_process(inner.next().unwrap())?);
            Ok(Process::NonDetChoice(left, right))
        },
        Rule::parallel => {
            let mut inner = pair.into_inner();
            let left = Box::new(parse_process(inner.next().unwrap())?);
            let right = Box::new(parse_process(inner.next().unwrap())?);
            Ok(Process::Parallel(left, right))
        },
        Rule::rename => {
            let mut inner = pair.into_inner();
            let process = Box::new(parse_process(inner.next().unwrap())?);
            let b = inner.next().unwrap().as_span().as_str().to_owned();
            let a = inner.next().unwrap().as_span().as_str().to_owned();
            Ok(Process::Rename(process, b.into(), a.into()))
        },
        Rule::restriction => {
            let mut inner = pair.into_inner();
            let process = Box::new(parse_process(inner.next().unwrap())?);
            let first_label = inner.next().unwrap().as_span().as_str().to_owned();
            let mut restriction = Process::Restriction(process, first_label.into());
            for label in inner.map(|p| p.as_span().as_str().to_owned()) {
                restriction = Process::Restriction(Box::new(restriction), label.into())
            }
            Ok(restriction)
        },
        Rule::process_name => {
            let name: Rc<_> = pair.as_span().as_str().to_owned().into();
            if *name == "_" {
                Err(CCSError::parsing_anonymous_process())
            } else {
                Ok(Process::ProcessName(name))
            }
        },
        _ => Err(CCSError::parsing_unexpected_rule(pair.as_rule())),
    }
}

fn parse_specification(pair: Pair<Rule>) -> CCSResult<(ProcessName, Process)> {
    if pair.as_rule() != Rule::specification {
        return Err(CCSError::parsing_unexpected_rule(pair.as_rule()));
    }

    let mut inner = pair.into_inner();

    let name_pair = inner.next().ok_or(CCSError::parsing_rule_not_found(Rule::process_name))?;
    if name_pair.as_rule() != Rule::process_name {
        return Err(CCSError::parsing_unexpected_rule(name_pair.as_rule()));
    }
    let name = name_pair.as_span().as_str().to_owned();

    let process_pair = inner.next().ok_or(CCSError::parsing_rule_not_found(Rule::process))?;
    if process_pair.as_rule() != Rule::process {
        return Err(CCSError::parsing_unexpected_rule(process_pair.as_rule()));
    }
    let process = parse_process(process_pair)?;

    Ok((name.into(), process))
}

fn parse_system(pair: Pair<Rule>, name: String) -> CCSResult<CCSSystem> {
    if pair.as_rule() != Rule::system {
        return Err(CCSError::parsing_unexpected_rule(pair.as_rule()));
    }

    let mut processes = HashMap::new();
    let mut destinct_process = None;

    for spec_pair in pair.into_inner().filter(|p| p.as_rule() == Rule::specification) {
        let (name, process) = parse_specification(spec_pair)?;

        if destinct_process.is_none() {
            destinct_process = Some(name.clone());
        }

        processes.insert(name, process);
    }

    let destinct_process = destinct_process
        .ok_or(CCSError::parsing_rule_not_found(Rule::specification))?;

    Ok(CCSSystem::new(name, processes, destinct_process))
}

pub fn first_pass(input: &str) -> CCSResult<Pair<'_, Rule>> {
    Ok(CCSParser::parse(Rule::system, input)
        .map_err(CCSError::syntax_error)?
        .next().unwrap())
}

pub fn parse(name: String, input: &str) -> CCSResult<CCSSystem> {
    let first_pass = first_pass(input)?;
    let second_pass = parse_system(first_pass, name)?;
    Ok(second_pass)
}
