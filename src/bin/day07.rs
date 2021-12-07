use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(parse(from_os_str))]
    input: PathBuf,
}

type Crabs = HashMap<isize, isize>;

fn read_crabs<P: AsRef<Path>>(input: P) -> Crabs {
    let mut crabs = HashMap::new();

    let data = fs::read_to_string(input).unwrap();
    let positions = data
        .trim_end()
        .split(',')
        .map(|num| num.parse::<isize>().unwrap());

    for position in positions {
        (*crabs.entry(position).or_default()) += 1;
    }

    crabs
}

fn find_min_linear_fuel_to_align(crabs: &Crabs) -> isize {
    let mut current_fuel: isize = crabs.iter().map(|(position, count)| position * count).sum();
    let mut left_crabs: isize = crabs.get(&0).cloned().unwrap_or_default();
    let mut right_crabs: isize = crabs.values().sum::<isize>() - left_crabs;
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

fn find_min_quadratic_fuel_to_align(crabs: &Crabs) -> isize {
    let min_pos = crabs.keys().min().cloned().unwrap();
    let max_pos = crabs.keys().max().cloned().unwrap();

    let fuel_to_move_all_crabs = |pos: isize| {
        crabs
            .iter()
            .map(|(crab_pos, num_crabs)| num_crabs * fuel_to_move_one_crab(pos, *crab_pos))
            .sum::<isize>()
    };

    fn fuel_to_move_one_crab(pos: isize, crab_pos: isize) -> isize {
        let distance = (crab_pos - pos).abs();
        (distance * (distance + 1)) / 2
    }

    (min_pos..=max_pos)
        .map(fuel_to_move_all_crabs)
        .min()
        .unwrap()
}

fn main() {
    let opt = Opt::from_args();

    let crabs = read_crabs(&opt.input);
    let min_fuel = find_min_linear_fuel_to_align(&crabs);
    println!("{}", min_fuel);

    let min_fuel = find_min_quadratic_fuel_to_align(&crabs);
    println!("{}", min_fuel);
}
