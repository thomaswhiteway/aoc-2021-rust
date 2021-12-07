use std::fs;
use std::path::{Path, PathBuf};
use structopt::StructOpt;
use std::collections::HashMap;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(parse(from_os_str))]
    input: PathBuf,
}

type Crabs = HashMap<usize, usize>;

fn read_crabs<P: AsRef<Path>>(input: P) -> Crabs {
    let mut crabs = HashMap::new();

    let data = fs::read_to_string(input).unwrap();
    let positions = data
        .trim_end()
        .split(',')
        .map(|num| num.parse::<usize>().unwrap());

    for position in positions {
        (*crabs.entry(position).or_default()) += 1;
    }

    crabs
}

fn find_min_fuel_to_align(crabs: &Crabs) -> usize {
    let mut current_fuel: usize = crabs.iter().map(|(position, count)| position * count).sum();
    let mut left_crabs: usize = crabs.get(&0).cloned().unwrap_or_default();
    let mut right_crabs: usize = crabs.values().sum::<usize>() - left_crabs;
    let mut position = 0;
    
    while right_crabs > left_crabs {
        current_fuel -= right_crabs - left_crabs;

        position += 1;
        let new_crabs = crabs.get(&position).cloned().unwrap_or_default();
        left_crabs += new_crabs;
        right_crabs -= new_crabs;

    }   
    
    current_fuel
}

fn main() {
    let opt = Opt::from_args();

    let crabs = read_crabs(&opt.input);
    let min_fuel = find_min_fuel_to_align(&crabs);
    println!("{}", min_fuel);
}
