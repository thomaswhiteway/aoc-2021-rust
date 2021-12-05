use std::cmp::{max, Ordering};
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

impl Position {
    fn offset(self, dx: isize, dy: isize) -> Position {
        Position {
            x: self.x + dx,
            y: self.y + dy,
        }
    }
}

#[derive(Clone)]
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
        fn delta(start: isize, end: isize) -> isize {
            use Ordering::*;
            match start.cmp(&end) {
                Less => 1,
                Equal => 0,
                Greater => -1,
            }
        }
        let dx = delta(self.start.x, self.end.x);
        let dy = delta(self.start.y, self.end.y);

        let length_x = (self.end.x - self.start.x).abs() + 1;
        let length_y = (self.end.y - self.start.y).abs() + 1;
        assert!(length_x == length_y || length_x == 1 || length_y == 1);
        let length = max(length_x, length_y);

        let start = self.start;
        (0..length).map(move |offset| start.offset(offset * dx, offset * dy))
    }
}

fn read_lines<P: AsRef<Path>>(path: P) -> Box<[Line]> {
    parsing::parse_lines(&fs::read_to_string(path).unwrap()).unwrap()
}

fn count_overlaps(lines: &[Line]) -> usize {
    let mut counts: HashMap<Position, usize> = HashMap::new();

    for line in lines {
        for point in line.points() {
            *counts.entry(point).or_default() += 1;
        }
    }

    counts.values().filter(|c| **c > 1).count()
}

fn main() {
    let opt = Opt::from_args();

    let all_lines = read_lines(&opt.input);

    let flat_lines = all_lines
        .iter()
        .filter(|line| line.is_horizontal() || line.is_vertical())
        .cloned()
        .collect::<Vec<_>>();
    let flat_overlaps = count_overlaps(&flat_lines);
    println!("Flat Overlaps: {}", flat_overlaps);

    let all_overlaps = count_overlaps(&all_lines);
    println!("All Overlaps: {}", all_overlaps);
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
