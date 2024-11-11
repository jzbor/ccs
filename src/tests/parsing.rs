use std::fs;

use crate::parser;

fn parse_file_twice(file: &str) {
    let contents = fs::read_to_string(file).unwrap();

    let first_parse_iteration = parser::parse(&contents).unwrap();
    let first_string = first_parse_iteration.to_string();
    let second_parse_iteration = parser::parse(&first_string).unwrap();

    for (name, spec) in first_parse_iteration.processes() {
        assert_eq!(spec.to_string(), second_parse_iteration.processes().get(name).unwrap().to_string(), "[{}]", file);
        assert_eq!(spec, second_parse_iteration.processes().get(name).unwrap(), "[{}]", file);
    }
    for (name, spec) in second_parse_iteration.processes() {
        assert_eq!(spec.to_string(), first_parse_iteration.processes().get(name).unwrap().to_string(), "[{}]", file);
        assert_eq!(spec, first_parse_iteration.processes().get(name).unwrap(), "[{}]", file);
    }

    assert_eq!(first_parse_iteration, second_parse_iteration, "[{}]", file);
}


#[test]
fn parse_twice() {
    for example in super::EXAMPLES {
        parse_file_twice(example)
    }
}
