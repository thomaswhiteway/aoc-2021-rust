use std::fs;
use std::num::ParseIntError;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use itertools::Itertools;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(parse(from_os_str))]
    input: PathBuf,
}

#[derive(Debug, Clone, Copy)]
struct Range {
    min: i64,
    max: i64,
}

impl FromStr for Range {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s[2..].split("..").map(i64::from_str).collect::<Result<Vec<_>, _>>()?;
        Ok(Range { min: parts[0], max: parts[1]})
    }
}

fn parse_ranges<P: AsRef<Path>>(input: P) -> (Range, Range) {
    let text = fs::read_to_string(input).unwrap();
    text[13..].trim_end().split(", ").map(Range::from_str).map(Result::unwrap).collect_tuple().unwrap()
}

fn find_max_velocity(y_range: Range) -> i64 {
    assert!(y_range.min < 0);
    -y_range.min - 1
}

fn max_height_for_velocity(velocity: i64) -> i64 {
    velocity * (velocity + 1) / 2
}

fn find_max_height(y_range: Range) -> i64 {
    let max_velocity = find_max_velocity(y_range);
    max_height_for_velocity(max_velocity)
}

fn main() {
    let opt = Opt::from_args();
    let (_, y_range) = parse_ranges(opt.input);
    let max_height = find_max_height(y_range);
    println!("{}", max_height);
}