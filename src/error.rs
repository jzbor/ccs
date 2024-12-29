use std::process;

use thiserror::Error;

use crate::{ccs::ProcessName, parser};

pub type CCSResult<T> = Result<T, CCSError>;

#[derive(Error, Debug)]
pub enum CCSError {
    #[error("Parsing Error: unexpected rule - {0} [{1}:{2}]")]
    ParsingUnexpectedRule(String, usize, usize),

    #[error("Parsing Error: rule not found - {0} (this is probably a bug)")]
    ParsingRuleNotFound(String),

    #[error("Parsing Error: anonymous processes are not allowed on the rhs of a specification [{0}:{1}]")]
    ParsingAnonymousProcess(usize, usize),

    #[error("Syntax Error:\n{0}")]
    SyntaxError(Box<pest::error::Error<parser::Rule>>),

    #[error("File Error: {0}")]
    File(#[from] std::io::Error),

    #[error("Child Error: unable to execute '{0}'")]
    ChildCreation(String),

    #[error("Child Error: child exited with error '{0}'")]
    ChildExited(i32),

    #[error("Overlapping Process Names: process '{0}' exists in both systems")]
    OverlappingProcess(ProcessName),

    #[error("Results are not yet calculated (this is probably a bug)")]
    ResultsNotAvailable(),
}


impl CCSError {
    pub fn parsing_unexpected_rule(rule: parser::Rule, span: &pest::Span) -> Self {
        let pos = span.start_pos().line_col();
        CCSError::ParsingUnexpectedRule(format!("{:?}", rule), pos.0, pos.1)
    }

    pub fn parsing_rule_not_found(rule: parser::Rule) -> Self {
        CCSError::ParsingRuleNotFound(format!("{:?}", rule))
    }

    pub fn parsing_anonymous_process(span: &pest::Span) -> Self {
        let pos = span.start_pos().line_col();
        CCSError::ParsingAnonymousProcess(pos.0, pos.1)
    }

    pub fn child_creation(name: String) -> Self {
        CCSError::ChildCreation(name)
    }

    pub fn child_exited(code: i32) -> Self {
        CCSError::ChildExited(code)
    }

    pub fn file_error(e: std::io::Error) -> Self {
        CCSError::File(e)
    }

    pub fn overlapping_process_error(process: ProcessName) -> Self {
        CCSError::OverlappingProcess(process)
    }

    pub fn syntax_error(e: pest::error::Error<parser::Rule>) -> Self {
        CCSError::SyntaxError(Box::new(e))
    }

    pub fn results_not_available() -> Self {
        CCSError::ResultsNotAvailable()
    }
}

pub fn resolve<T>(result: CCSResult<T>) -> T {
    match result {
        Ok(t) => t,
        Err(e) => {
            eprintln!("{}", e);
            process::exit(1);
        },
    }
}
