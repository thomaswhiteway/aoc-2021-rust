use aoc2021::position::{Direction, Position, TorusMap};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(parse(from_os_str))]
    input: PathBuf,
}

type CucumberMap = TorusMap<Direction>;

fn read_map<P: AsRef<Path>>(input: P) -> CucumberMap {
    let grid = BufReader::new(File::open(input).unwrap())
        .lines()
        .map(Result::unwrap)
        .map(|line| line.chars().collect::<Vec<_>>())
        .collect::<Vec<_>>();

    let map = grid
        .iter()
        .enumerate()
        .flat_map(|(y, chars)| {
            chars
                .iter()
                .enumerate()
                .filter_map(|(x, &c)| {
                    Direction::try_from(c)
                        .ok()
                        .map(|d| (Position::new(x as i64, y as i64), d))
                })
                .collect::<Vec<_>>()
        })
        .collect();

    CucumberMap::new(map, grid[0].len() as i64, grid.len() as i64)
}

fn move_cucumbers(map: &CucumberMap, move_in: Direction) -> CucumberMap {
    map.map(|(position, direction)| {
        if *direction == move_in && !map.contains_key(&position.step(*direction)) {
            (position.step(*direction), *direction)
        } else {
            (*position, *direction)
        }
    })
}

#[allow(dead_code)]
fn print_map(map: &CucumberMap) {
    for y in 0..map.height() {
        for x in 0..map.width() {
            print!(
                "{}",
                map.get(&Position::new(x, y))
                    .cloned()
                    .map(char::from)
                    .unwrap_or('.')
            )
        }
        println!()
    }
    println!()
}

fn move_until_gridlock(map: &CucumberMap) -> usize {
    let mut map = map.clone();

    for step in 1.. {
        let mut new_map = move_cucumbers(&map, Direction::East);
        new_map = move_cucumbers(&new_map, Direction::South);

        if new_map == map {
            return step;
        }

        map = new_map;
    }

    unreachable!()
}

fn main() {
    let opt = Opt::from_args();
    let map = read_map(opt.input);

    let step = move_until_gridlock(&map);
    println!("{}", step);
}
