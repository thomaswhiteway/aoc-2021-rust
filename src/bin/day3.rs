use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(parse(from_os_str))]
    input: PathBuf,
}

fn read_values<P: AsRef<Path>>(input: P) -> Box<[String]> {
    BufReader::new(File::open(input).unwrap())
        .lines()
        .map(Result::unwrap)
        .collect::<Vec<_>>()
        .into_boxed_slice()
}

fn get_bit_counts(values: &[String]) -> Box<[usize]> {
    let mut counts = vec![0_usize; values[0].len()];
    for value in values {
        for (index, c) in value.chars().enumerate() {
            if c == '1' {
                counts[index] += 1
            }
        }
    }
    counts.into_boxed_slice()
}

fn get_bit_value<F>(bit_counts: &[usize], bit_set: F) -> usize
where
    F: Fn(usize) -> bool,
{
    let value_str: String = bit_counts
        .iter()
        .map(|c| if bit_set(*c) { "1" } else { "0" })
        .collect();
    usize::from_str_radix(&value_str, 2).unwrap()
}

fn get_gamma(total: usize, bit_counts: &[usize]) -> usize {
    get_bit_value(bit_counts, |c| c * 2 > total)
}

fn get_epsilon(total: usize, bit_counts: &[usize]) -> usize {
    get_bit_value(bit_counts, |c| c * 2 < total)
}

fn get_power_consumption(values: &[String]) -> usize {
    let bit_counts = get_bit_counts(values);

    let gamma = get_gamma(values.len(), &bit_counts);
    let epsilon = get_epsilon(values.len(), &bit_counts);

    gamma * epsilon
}

fn main() {
    let opt = Opt::from_args();

    let values = read_values(&opt.input);
    let power_consumption = get_power_consumption(&values);
    println!("{}", power_consumption);
}
