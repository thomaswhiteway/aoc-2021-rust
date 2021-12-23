use aoc2021::a_star;
use derivative::*;
use std::collections::HashMap;
use std::fs::File;
use std::hash::Hash;
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

#[derive(Derivative)]
#[derivative(Debug)]
#[derive(Clone)]
struct State<'a> {
    #[derivative(Debug = "ignore")]
    risks: &'a RiskMap,
    position: Position,
    target: Position,
}

impl<'a> Hash for State<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.position.hash(state);
        self.target.hash(state);
    }
}

impl<'a> PartialEq for State<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.position == other.position && self.target == other.target
    }
}

impl<'a> Eq for State<'a> {}

impl<'a> State<'a> {
    fn new(risks: &'a RiskMap) -> Self {
        State {
            risks,
            position: risks.top_left(),
            target: risks.bottom_right(),
        }
    }

    fn successor(&self, position: Position) -> Self {
        State {
            risks: self.risks,
            position,
            target: self.target,
        }
    }
}

impl<'a> a_star::State for State<'a> {
    fn min_remaining_cost(&self) -> usize {
        self.position.distance_to(&self.target) as usize
    }

    fn is_complete(&self) -> bool {
        self.position == self.target
    }

    fn successors(&self) -> Box<dyn Iterator<Item = (Self, usize)> + '_> {
        Box::new(
            self.position
                .adjacent()
                .filter_map(|pos| self.risks.get(&pos).map(|risk| (self.successor(pos), risk))),
        )
    }
}

fn main() {
    let opt = Opt::from_args();

    let risks = parse_risk_map(opt.input);

    let (_, total_risk) = a_star::solve(State::new(&risks)).unwrap();
    println!("{}", total_risk);

    let risks = risks.with_mult(5);

    let (_, total_risk) = a_star::solve(State::new(&risks)).unwrap();
    println!("{}", total_risk);
}
