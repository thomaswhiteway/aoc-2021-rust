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

fn opener(close: char) -> char {
    match close {
        ')' => '(',
        ']' => '[',
        '}' => '{',
        '>' => '<',
        _ => panic!("Not a close bracket: {}", close),
    }
}

fn find_invalid_char(line: &str) -> Option<char> {
    let mut stack = vec![];

    for c in line.chars() {
        match c {
            '(' | '[' | '{' | '<' => stack.push(c),
            ')' | ']' | '}' | '>' => {
                let open = stack.pop();
                if open != Some(opener(c)) {
                    return Some(c);
                }
            }
            _ => panic!("Unexpected character {}", c),
        }
    }

    None
}

fn find_invalid_chars(program: &[String]) -> impl Iterator<Item = char> + '_ {
    program
        .iter()
        .map(String::as_str)
        .filter_map(find_invalid_char)
}

fn char_score(c: char) -> usize {
    match c {
        ')' => 3,
        ']' => 57,
        '}' => 1197,
        '>' => 25137,
        _ => panic!("Unexpected invalid char: {}", c),
    }
}

fn main() {
    let opt = Opt::from_args();

    let program = read_program(opt.input);
    let score: usize = find_invalid_chars(&program).map(char_score).sum();
    println!("{}", score);
}
