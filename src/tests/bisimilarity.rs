use std::fs;

use crate::{bisimilarity, parser};


fn compare_bisimulation_impls(file: &str) {
    let contents = fs::read_to_string(file).unwrap();
    let system = parser::parse(file.to_string(), &contents).unwrap();

    let mut fix: Vec<_> = bisimilarity::bisimulation(&system, false, true).0.unwrap()
        .into_iter()
        .map(|(p, q)| (p.to_string(), q.to_string()))
        .collect();
    let mut pt: Vec<_> = bisimilarity::bisimulation(&system, true, true).0.unwrap()
        .into_iter()
        .map(|(p, q)| (p.to_string(), q.to_string()))
        .collect();
    fix.sort();
    pt.sort();
    assert_eq!(fix.len(), pt.len(), "[{}]", file);
    assert_eq!(fix, pt, "[{}]", file);
}


#[test]
fn compare_bisimulation() {
    for example in super::EXAMPLES {
        compare_bisimulation_impls(example)
    }
}

#[test]
#[ntest::timeout(10000)]
fn big_bisimulation() {
    let file = "examples/bisimbench_25k_25k.ccs";
    let contents = fs::read_to_string(file).unwrap();
    let system = parser::parse(file.to_string(), &contents).unwrap();

    bisimilarity::bisimulation(&system, true, false);
}

#[test]
#[cfg(feature = "very_big_test")]
#[ntest::timeout(80000)]
fn very_big_bisimulation() {
    let file = "examples/bisimbench_1M_1M.ccs";
    let contents = fs::read_to_string(file).unwrap();
    let system = parser::parse(file.to_string(), &contents).unwrap();

    bisimilarity::bisimulation(&system, true, false);
}
