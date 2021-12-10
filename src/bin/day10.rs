use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(parse(from_os_str))]
    input: PathBuf,
}

fn read_program<P: AsRef<Path>>(input: P) -> Box<[String]> {
    BufReader::new(File::open(input).unwrap())
        .lines()
        .map(Result::unwrap)
        .collect::<Vec<_>>()
        .into_boxed_slice()
}

enum ValidateResult {
    Invalid(char),
    Incomplete(String)
}

impl ValidateResult {
    fn invalid_char(&self) -> Option<char> {
        match *self {
            ValidateResult::Invalid(c) => Some(c),
            _ => None
        }
    }

    fn remaining_string(&self) -> Option<&str> {
        match self {
            ValidateResult::Incomplete(remaining) => Some(&remaining),
            _ => None
        }
    }    
}

fn closer(open: char) -> char {
    match open {
        '(' => ')',
        '[' => ']',
        '{' => '}',
        '<' => '>',
        _ => panic!("Not a open bracket: {}", open),
    }
}

fn validate_line(line: &str) -> ValidateResult {
    let mut stack = vec![];

    for c in line.chars() {
        match c {
            '(' | '[' | '{' | '<' => stack.push(closer(c)),
            ')' | ']' | '}' | '>' => {
                let expected = stack.pop();
                if expected != Some(c) {
                    return ValidateResult::Invalid(c);
                }
            }
            _ => panic!("Unexpected character {}", c),
        }
    }

    let remaining = stack.into_iter().rev().collect();
    ValidateResult::Incomplete(remaining)
}

fn validate_program(program: &[String]) -> Box<[ValidateResult]> {
    program
        .iter()
        .map(String::as_str)
        .map(validate_line)
        .collect::<Vec<_>>()
        .into_boxed_slice()
}

fn invalid_char_score(c: char) -> usize {
    match c {
        ')' => 3,
        ']' => 57,
        '}' => 1197,
        '>' => 25137,
        _ => panic!("Unexpected invalid char: {}", c),
    }
}

fn remaining_char_score(c: char) -> usize {
    match c {
        ')' => 1,
        ']' => 2,
        '}' => 3,
        '>' => 4,
        _ => panic!("Unexpected remaining char: {}", c),
    }
}

fn remaining_score(remaining: &str) -> usize {
    remaining.chars().rev().enumerate().map(|(index, c)| 5_usize.pow(index as u32) * remaining_char_score(c)).sum()
}

fn main() {
    let opt = Opt::from_args();

    let program = read_program(opt.input);
    let validate_results = validate_program(&program);
    let invalid_score: usize = validate_results.iter().filter_map(ValidateResult::invalid_char).map(invalid_char_score).sum();
    println!("{}", invalid_score);

    let mut remaining_scores: Vec<usize> = validate_results.iter().filter_map(ValidateResult::remaining_string).map(remaining_score).collect();
    remaining_scores.sort();
    let middle_score = remaining_scores[remaining_scores.len()/2];
    println!("{}", middle_score);
}
