use std::process;

use thiserror::Error;

use crate::parser;

pub type CCSResult<T> = Result<T, CCSError>;

#[derive(Error, Debug)]
pub enum CCSError {
    #[error("Parsing Error: unexpected rule - {0}")]
    ParsingUnexpectedRule(String),

    #[error("Parsing Error: rule not found - {0}")]
    ParsingRuleNotFound(String),

    #[error("Parsing Error: anonymous processes not allowed on rhs")]
    ParsingAnonymousProcess(),

    #[error("Syntax Error:\n{0}")]
    SyntaxError(Box<pest::error::Error<parser::Rule>>),

    #[error("File Error: {0}")]
    File(#[from] std::io::Error),

    #[error("Child Error: unable to execute '{0}'")]
    ChildCreation(String),

    #[error("Child Error: child exited with error '{0}'")]
    ChildExited(i32),
}

impl CCSError {
    pub fn parsing_unexpected_rule(rule: parser::Rule) -> Self {
        CCSError::ParsingUnexpectedRule(format!("{:?}", rule))
    }

    pub fn parsing_rule_not_found(rule: parser::Rule) -> Self {
        CCSError::ParsingRuleNotFound(format!("{:?}", rule))
    }

    pub fn parsing_anonymous_process() -> Self {
        CCSError::ParsingAnonymousProcess()
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

    pub fn syntax_error(e: pest::error::Error<parser::Rule>) -> Self {
        CCSError::SyntaxError(Box::new(e))
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
