use std::collections::BinaryHeap;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(parse(from_os_str))]
    input: PathBuf,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
struct Position {
    x: isize,
    y: isize,
}

impl Position {
    fn new(x: isize, y: isize) -> Self {
        Position { x, y }
    }
    fn distance_to(&self, other: &Position) -> isize {
        (self.x - other.x).abs() + (self.y - other.y).abs()
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

type RiskMap = HashMap<Position, usize>;

fn parse_risk_map<P: AsRef<Path>>(input: P) -> RiskMap {
    BufReader::new(File::open(input).unwrap())
        .lines()
        .map(Result::unwrap)
        .enumerate()
        .flat_map(|(y, row)| {
            row.chars()
                .enumerate()
                .map(move |(x, c)| {
                    (
                        Position {
                            x: x as isize,
                            y: y as isize,
                        },
                        c.to_digit(10).unwrap() as usize,
                    )
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

#[derive(PartialEq, Eq)]
struct Candidate {
    position: Position,
    total_risk: usize,
    min_remaining: usize,
}

impl Candidate {
    fn new(position: Position, target: Position, total_risk: usize) -> Self {
        Candidate {
            position,
            total_risk,
            min_remaining: position.distance_to(&target) as usize, // Each step involves at least one risk
        }
    }
}

impl PartialOrd for Candidate {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(
            (self.total_risk + self.min_remaining)
                .cmp(&(other.total_risk + other.min_remaining))
                .reverse(),
        )
    }
}

impl Ord for Candidate {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

fn find_lowest_total_risk(risks: &RiskMap, start: Position, end: Position) -> Option<usize> {
    let mut candidates = BinaryHeap::new();
    let mut visited = HashSet::new();

    candidates.push(Candidate::new(start, end, 0));

    while let Some(candidate) = candidates.pop() {
        if candidate.position == end {
            return Some(candidate.total_risk);
        }

        if !visited.contains(&candidate.position) {
            visited.insert(candidate.position);

            for adjacent in candidate.position.adjacent() {
                if !visited.contains(&adjacent) {
                    if let Some(&risk) = risks.get(&adjacent) {
                        candidates.push(Candidate::new(adjacent, end, candidate.total_risk + risk))
                    }
                }
            }
        }
    }

    None
}

fn main() {
    let opt = Opt::from_args();

    let risks = parse_risk_map(opt.input);

    let max_x = risks.keys().map(|pos| pos.x).max().unwrap();
    let max_y = risks.keys().map(|pos| pos.y).max().unwrap();

    let total_risk =
        find_lowest_total_risk(&risks, Position::new(0, 0), Position::new(max_x, max_y)).unwrap();
    println!("{}", total_risk);
}
