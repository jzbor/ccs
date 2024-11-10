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

    #[error("Syntax Error:\n{0}")]
    SyntaxError(#[from] pest::error::Error<parser::Rule>),

    #[error("File Error: {0}")]
    FileError(#[from] std::io::Error),
}

impl CCSError {
    pub fn parsing_unexpected_rule(rule: parser::Rule) -> Self {
        CCSError::ParsingUnexpectedRule(format!("{:?}", rule))
    }

    pub fn parsing_rule_not_found(rule: parser::Rule) -> Self {
        CCSError::ParsingRuleNotFound(format!("{:?}", rule))
    }

    pub fn file_error(e: std::io::Error) -> Self {
        CCSError::FileError(e)
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
