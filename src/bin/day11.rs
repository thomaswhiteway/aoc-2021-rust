use itertools::Itertools;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(parse(from_os_str))]
    input: PathBuf,
}

#[derive(Clone, Copy, Hash, Eq, PartialEq, Debug)]
struct Position {
    x: isize,
    y: isize,
}

impl Position {
    fn new(x: isize, y: isize) -> Self {
        Position { x, y }
    }

    fn offset(&self, dx: isize, dy: isize) -> Self {
        Position {
            x: self.x + dx,
            y: self.y + dy,
        }
    }

    fn adjacent(&self) -> impl Iterator<Item = Position> {
        let me = *self;
        (-1..=1)
            .cartesian_product(-1..=1)
            .filter(|&(dx, dy)| dx != 0 || dy != 0)
            .map(move |(dx, dy)| me.offset(dx, dy))
    }
}

type Octopuses = HashMap<Position, usize>;

fn read_octopuses<P: AsRef<Path>>(input: P) -> Octopuses {
    BufReader::new(File::open(input).unwrap())
        .lines()
        .map(Result::unwrap)
        .enumerate()
        .flat_map(|(y, line)| {
            line.chars()
                .enumerate()
                .map(|(x, energy)| {
                    (
                        Position::new(x as isize, y as isize),
                        energy.to_digit(10).unwrap() as usize,
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

fn all_positions() -> impl Iterator<Item = Position> {
    (0..10)
        .cartesian_product(0..10)
        .map(|(x, y)| Position::new(x, y))
}

fn step(octopuses: &mut Octopuses) -> usize {
    for energy in octopuses.values_mut() {
        *energy += 1;
    }

    let mut flashed = HashSet::new();

    loop {
        let mut have_flashed = false;

        for position in all_positions() {
            if *octopuses.get(&position).unwrap() > 9 && !flashed.contains(&position) {
                for neighbour in position.adjacent() {
                    if let Some(energy) = octopuses.get_mut(&neighbour) {
                        *energy += 1;
                    }
                }

                have_flashed = true;
                flashed.insert(position);
            }
        }

        if !have_flashed {
            break;
        }
    }

    for position in flashed.iter() {
        *octopuses.get_mut(position).unwrap() = 0;
    }

    flashed.len()
}

fn count_flashes(mut octopuses: Octopuses, steps: usize) -> usize {
    let mut total = 0;

    for _ in 0..steps {
        total += step(&mut octopuses);
    }

    total
}

fn find_when_all_flash(mut octopuses: Octopuses) -> usize {
    for index in 1.. {
        if step(&mut octopuses) == 100 {
            return index;
        }
    }
    panic!("Unreachable");
}

fn main() {
    let opt = Opt::from_args();

    let octopuses = read_octopuses(opt.input);

    let total = count_flashes(octopuses.clone(), 100);
    println!("{}", total);

    let all_flash_step = find_when_all_flash(octopuses);
    println!("{}", all_flash_step);
}
