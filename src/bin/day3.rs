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

fn get_most_common_bits(total: usize, bit_counts: &[usize]) -> String {
    bit_counts
        .iter()
        .map(|c| if c * 2 >= total { '1' } else { '0' })
        .collect()
}

fn flip_bits(input: &str) -> String {
    input
        .chars()
        .map(|c| if c == '0' { '1' } else { '0' })
        .collect()
}

fn parse_base2(input: &str) -> usize {
    usize::from_str_radix(input, 2).unwrap()
}

fn get_power_consumption(values: &[String]) -> usize {
    let bit_counts = get_bit_counts(values);

    let most_common_bits = get_most_common_bits(values.len(), &bit_counts);
    let least_common_bits = flip_bits(&most_common_bits);

    let gamma = parse_base2(&most_common_bits);
    let epsilon = parse_base2(&least_common_bits);

    gamma * epsilon
}

fn get_rating<F>(values: &[String], take_set: F) -> usize
where
    F: Fn(usize, usize) -> bool,
{
    let mut remaining: Vec<&str> = values.iter().map(String::as_str).collect();
    let mut index = 0;

    while remaining.len() > 1 {
        let (set, unset): (Vec<_>, Vec<_>) = remaining
            .iter()
            .partition(|val| val.chars().nth(index) == Some('1'));

        remaining = if take_set(set.len(), unset.len()) {
            set
        } else {
            unset
        };

        index += 1;
    }

    parse_base2(remaining[0])
}

fn get_oxygen_rating(values: &[String]) -> usize {
    get_rating(values, |set, unset| set >= unset)
}

fn get_co2_rating(values: &[String]) -> usize {
    get_rating(values, |set, unset| set < unset)
}

fn get_life_support_rating(values: &[String]) -> usize {
    let oxygen_generator_rating = get_oxygen_rating(values);
    let co2_scrubber_rating = get_co2_rating(values);

    oxygen_generator_rating * co2_scrubber_rating
}

fn main() {
    let opt = Opt::from_args();

    let values = read_values(&opt.input);

    let power_consumption = get_power_consumption(&values);
    println!("Power Consumption: {}", power_consumption);

    let life_support_rating = get_life_support_rating(&values);
    println!("Life Support Rating: {}", life_support_rating);
}
