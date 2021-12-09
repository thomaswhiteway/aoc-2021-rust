use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(parse(from_os_str))]
    input: PathBuf,
}

#[derive(Clone, Copy, Hash, Eq, PartialEq)]
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
        [(0, 1), (1, 0), (-1, 0), (0, -1)]
            .into_iter()
            .map(move |(dx, dy)| me.offset(dx, dy))
    }
}

type HeightMap = HashMap<Position, usize>;

fn read_map<P: AsRef<Path>>(input: P) -> HeightMap {
    BufReader::new(File::open(input).unwrap())
        .lines()
        .map(Result::unwrap)
        .enumerate()
        .flat_map(|(y, line)| {
            line.chars()
                .enumerate()
                .map(|(x, height)| {
                    (
                        Position::new(x as isize, y as isize),
                        height.to_digit(10).unwrap() as usize,
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

fn is_low_point(map: &HeightMap, position: &Position) -> bool {
    let this_height = *map.get(position).unwrap();
    position
        .adjacent()
        .filter_map(|adjacent| map.get(&adjacent).cloned())
        .all(|height| height > this_height)
}

fn find_low_points(map: &HeightMap) -> Box<[Position]> {
    map.keys()
        .filter(|position| is_low_point(map, position))
        .cloned()
        .collect::<Vec<_>>()
        .into_boxed_slice()
}

fn get_risk_level(map: &HeightMap, position: &Position) -> usize {
    map.get(position).unwrap() + 1
}

fn main() {
    let opt = Opt::from_args();

    let map = read_map(opt.input);
    let low_points = find_low_points(&map);
    let total_risk: usize = low_points
        .iter()
        .map(|position| get_risk_level(&map, position))
        .sum();
    println!("Total Risk: {}", total_risk)
}
