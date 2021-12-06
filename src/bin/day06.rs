use std::fs;
use std::path::{Path, PathBuf};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(parse(from_os_str))]
    input: PathBuf,
}

type Fishes = [u128; 9];

fn read_fish<P: AsRef<Path>>(input: P) -> Fishes {
    let mut fishes = [0; 9];

    let data = fs::read_to_string(input).unwrap();
    let nums = data
        .trim_end()
        .split(',')
        .map(|num| num.parse::<usize>().unwrap());

    for num in nums {
        fishes[num] += 1;
    }

    fishes
}

fn step_day(fishes: &mut Fishes) {
    let breeding_fishes = fishes[0];
    for index in 0..8 {
        fishes[index] = fishes[index + 1];
    }

    fishes[6] += breeding_fishes;
    fishes[8] = breeding_fishes;
}

fn count_fish(fishes: &Fishes) -> u128 {
    fishes.iter().sum()
}

fn main() {
    let opt = Opt::from_args();

    let mut fishes = read_fish(&opt.input);
    println!("Day 000: {}", count_fish(&fishes));

    for day in 1..=256 {
        step_day(&mut fishes);
        println!("Day {:03}: {}", day, count_fish(&fishes));
    }
}
