use std::cmp::{max, min};
use std::collections::HashMap;
use std::fs;
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

struct Line {
    start: Position,
    end: Position,
}

impl Line {
    fn is_horizontal(&self) -> bool {
        self.start.y == self.end.y
    }

    fn is_vertical(&self) -> bool {
        self.start.x == self.end.x
    }

    fn points(&self) -> impl Iterator<Item = Position> {
        if self.is_horizontal() {
            let min_x = min(self.start.x, self.end.x);
            let max_x = max(self.start.x, self.end.x);
            let y = self.start.y;
            Box::new((min_x..=max_x).map(move |x| Position { x, y }))
                as Box<dyn Iterator<Item = Position>>
        } else {
            assert!(self.is_vertical());
            let x = self.start.x;
            let min_y = min(self.start.y, self.end.y);
            let max_y = max(self.start.y, self.end.y);
            Box::new((min_y..=max_y).map(move |y| Position { x, y }))
                as Box<dyn Iterator<Item = Position>>
        }
    }
}

fn read_lines<P: AsRef<Path>>(path: P) -> Box<[Line]> {
    parsing::parse_lines(&fs::read_to_string(path).unwrap()).unwrap()
}

fn count_overlaps(lines: &[Line]) -> usize {
    let mut counts: HashMap<Position, usize> = HashMap::new();

    for line in lines {
        if line.is_horizontal() || line.is_vertical() {
            for point in line.points() {
                *counts.entry(point).or_default() += 1;
            }
        }
    }

    counts.values().filter(|c| **c > 1).count()
}

fn main() {
    let opt = Opt::from_args();

    let lines = read_lines(&opt.input);

    let overlaps = count_overlaps(&lines);

    println!("{}", overlaps);
}

mod parsing {
    use crate::Position;

    use super::Line;

    use nom::bytes::complete::tag;
    use nom::character::complete::one_of;
    use nom::combinator::{map, map_res, recognize};
    use nom::multi::many1;
    use nom::IResult;

    fn number(input: &str) -> IResult<&str, isize> {
        map_res(recognize(many1(one_of("0123456789"))), |val: &str| {
            val.parse()
        })(input)
    }

    fn position(input: &str) -> IResult<&str, Position> {
        let (input, x) = number(input)?;
        let (input, _) = tag(",")(input)?;
        let (input, y) = number(input)?;
        Ok((input, Position { x, y }))
    }

    fn line(input: &str) -> IResult<&str, Line> {
        let (input, start) = position(input)?;
        let (input, _) = tag(" -> ")(input)?;
        let (input, end) = position(input)?;
        let (input, _) = tag("\n")(input)?;
        Ok((input, Line { start, end }))
    }

    fn lines(input: &str) -> IResult<&str, Box<[Line]>> {
        map(many1(line), Vec::into_boxed_slice)(input)
    }

    pub(super) fn parse_lines(input: &str) -> Result<Box<[Line]>, impl std::error::Error + '_> {
        lines(input).map(|(_, lines)| lines)
    }
}
