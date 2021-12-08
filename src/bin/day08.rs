use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(parse(from_os_str))]
    input: PathBuf,
}

type Signals = HashSet<char>;

struct Problem {
    distinct_digits: [Signals; 10],
    output_digits: [Signals; 4],
}

fn parse_signals(sequence: &str) -> Vec<Signals> {
    sequence
        .split(' ')
        .map(|digits| digits.chars().collect())
        .collect()
}

impl FromStr for Problem {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let parts: Vec<_> = value.trim_end().split(" | ").collect();
        if parts.len() != 2 {
            return Err(format!("Invalid problem {}", value));
        }

        let distinct_digits = parse_signals(parts[0])
            .try_into()
            .map_err(|ds: Vec<Signals>| {
                format!("Incorrect number of distinct digits: {} != 10", ds.len())
            })?;
        let output_digits = parse_signals(parts[1])
            .try_into()
            .map_err(|ds: Vec<Signals>| {
                format!("Incorrect number of output digits: {} != 4", ds.len())
            })?;

        Ok(Problem {
            distinct_digits,
            output_digits,
        })
    }
}

fn read_problems<P: AsRef<Path>>(input: P) -> impl Iterator<Item = Problem> {
    BufReader::new(File::open(input).unwrap())
        .lines()
        .map(Result::unwrap)
        .map(|line| line.parse().unwrap())
}

fn find_digit<F>(digits: &mut Vec<Signals>, pred: F) -> Option<Signals>
    where F: Fn(&Signals) -> bool
{
    digits.iter().position(pred).map(|index| digits.remove(index))
}

fn find_digits(distinct_digits: &[Signals; 10]) -> [Signals; 10] {
    let mut output: [Signals; 10] = Default::default();
    let mut remaining_digits = distinct_digits.to_vec();

    output[1] = find_digit(&mut remaining_digits, |sigs| sigs.len() == 2).unwrap();
    output[4] = find_digit(&mut remaining_digits, |sigs| sigs.len() == 4).unwrap();
    output[7] = find_digit(&mut remaining_digits, |sigs| sigs.len() == 3).unwrap();
    output[8] = find_digit(&mut remaining_digits, |sigs| sigs.len() == 7).unwrap();

    output[6] = find_digit(&mut remaining_digits, |sigs| sigs.len() == 6 && !sigs.is_superset(&output[1])).unwrap();
    output[9] = find_digit(&mut remaining_digits, |sigs| sigs.len() == 6 && sigs.is_superset(&output[4])).unwrap();
    output[0] = find_digit(&mut remaining_digits, |sigs| sigs.len() == 6).unwrap();

    // All remaining digits have 5 signals
    output[3] = find_digit(&mut remaining_digits, |sigs| sigs.is_superset(&output[1])).unwrap();
    output[5] = find_digit(&mut remaining_digits, |sigs| sigs.intersection(&output[6]).count() == 5).unwrap();
    output[2] = remaining_digits.pop().unwrap();

    output
}

fn decode_output(digits: &[Signals; 10], output: &[Signals; 4]) -> [usize; 4] {
    output.iter().map(|signals| digits.iter().position(|sigs| sigs == signals).unwrap()).collect::<Vec<_>>().try_into().unwrap()
}

fn main() {
    let opt = Opt::from_args();

    let problems = read_problems(opt.input);
    let solution: usize = problems
        .map(|problem| {
            let digits = find_digits(&problem.distinct_digits);
            let output = decode_output(&digits, &problem.output_digits);
            output
                .iter()
                .filter(|&&d| d == 1 || d == 4 || d == 7 || d == 8)
                .count()
        })
        .sum();
    println!("{}", solution);
}
