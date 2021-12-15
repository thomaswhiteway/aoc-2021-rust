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

struct RiskMap {
    risks: HashMap<Position, usize>,
    mult: isize,
    width: isize,
    height: isize,
}

impl RiskMap {
    fn new(risks: HashMap<Position, usize>) -> Self {
        let width = risks.keys().map(|pos| pos.x).max().unwrap() + 1;
        let height = risks.keys().map(|pos| pos.y).max().unwrap() + 1;
        RiskMap {
            risks,
            mult: 1,
            width,
            height,
        }
    }

    fn with_mult(&self, mult: isize) -> RiskMap {
        RiskMap {
            risks: self.risks.clone(),
            mult,
            width: self.width,
            height: self.height,
        }
    }

    fn top_left(&self) -> Position {
        Position::new(0, 0)
    }

    fn bottom_right(&self) -> Position {
        Position::new(self.mult * self.width - 1, self.mult * self.height - 1)
    }

    fn get(&self, position: &Position) -> Option<usize> {
        let x = position.x % self.width;
        let x_wrap = position.x / self.width;
        let y = position.y % self.width;
        let y_wrap = position.y / self.width;

        if x_wrap >= self.mult || y_wrap >= self.mult {
            return None;
        }

        self.risks
            .get(&Position::new(x, y))
            .map(|&risk| (((risk + x_wrap as usize + y_wrap as usize) - 1) % 9) + 1)
    }
}

fn parse_risk_map<P: AsRef<Path>>(input: P) -> RiskMap {
    let risks = BufReader::new(File::open(input).unwrap())
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
        .collect();

    RiskMap::new(risks)
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
                    if let Some(risk) = risks.get(&adjacent) {
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

    let total_risk =
        find_lowest_total_risk(&risks, risks.top_left(), risks.bottom_right()).unwrap();
    println!("{}", total_risk);

    let risks = risks.with_mult(5);

    let total_risk =
        find_lowest_total_risk(&risks, risks.top_left(), risks.bottom_right()).unwrap();
    println!("{}", total_risk);
}
