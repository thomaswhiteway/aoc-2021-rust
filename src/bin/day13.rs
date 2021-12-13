use std::collections::HashSet;
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
    fn reflect(&self, axis: Axis, line: isize) -> Position {
        match axis {
            Axis::X => Position {
                x: line - (self.x - line),
                y: self.y,
            },
            Axis::Y => Position {
                x: self.x,
                y: line - (self.y - line),
            },
        }
    }

    fn coord(&self, axis: Axis) -> isize {
        match axis {
            Axis::X => self.x,
            Axis::Y => self.y,
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq, Debug)]
enum Axis {
    X,
    Y,
}

type Paper = HashSet<Position>;

struct Fold {
    axis: Axis,
    line: isize,
}

impl Fold {
    fn apply(&self, paper: &Paper) -> Paper {
        paper
            .iter()
            .map(|position| {
                if position.coord(self.axis) > self.line {
                    position.reflect(self.axis, self.line)
                } else {
                    *position
                }
            })
            .collect()
    }
}

type Inputs = (Paper, Box<[Fold]>);

fn parse_files<P: AsRef<Path>>(input: P) -> Inputs {
    parsing::parse_input(&fs::read_to_string(input).unwrap()).unwrap()
}

fn print_paper(paper: &Paper) {
    let max_x = paper.iter().map(|pos| pos.x).max().unwrap();
    let max_y = paper.iter().map(|pos| pos.y).max().unwrap();

    for y in 0..=max_y {
        for x in 0..=max_x {
            if paper.contains(&Position { x, y }) {
                print!("#");
            } else {
                print!(".");
            }
        }
        println!();
    }
}

fn main() {
    let opt = Opt::from_args();

    let (paper, folds) = parse_files(opt.input);

    let next_paper = folds[0].apply(&paper);
    println!("{}", next_paper.len());

    let final_paper = folds.iter().fold(paper, |paper, fold| fold.apply(&paper));
    print_paper(&final_paper);
}

mod parsing {
    use crate::{Axis, Fold, Inputs, Position};

    use nom::bytes::complete::tag;
    use nom::character::complete::one_of;
    use nom::combinator::{map_res, recognize};
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
        let (input, _) = tag("\n")(input)?;
        Ok((input, Position { x, y }))
    }

    fn axis(input: &str) -> IResult<&str, Axis> {
        map_res(one_of("xy"), |axis| match axis {
            'x' => Ok(Axis::X),
            'y' => Ok(Axis::Y),
            _ => Err(format!("Unknown axis: {}", axis)),
        })(input)
    }

    fn fold(input: &str) -> IResult<&str, Fold> {
        let (input, _) = tag("fold along ")(input)?;
        let (input, axis) = axis(input)?;
        let (input, _) = tag("=")(input)?;
        let (input, line) = number(input)?;
        let (input, _) = tag("\n")(input)?;
        Ok((input, Fold { axis, line }))
    }

    pub(super) fn parse_input(
        input: &str,
    ) -> Result<Inputs, Box<dyn std::error::Error + '_>> {
        let (input, positions) = many1(position)(input).map_err(Box::new)?;
        let (input, _) = tag::<_, _, ()>("\n")(input).map_err(Box::new)?;
        let (_, folds) = many1(fold)(input).map_err(Box::new)?;
        Ok((positions.into_iter().collect(), folds.into_boxed_slice()))
    }

    #[cfg(test)]
    mod test {
        use super::*;

        #[test]
        fn test_parse_position() {
            let (rest, pos) = position("9,10\n").unwrap();
            assert_eq!(rest, "");
            assert_eq!(pos.x, 9);
            assert_eq!(pos.y, 10);
        }

        #[test]
        fn test_parse_fold() {
            let (rest, f) = fold("fold along y=7\n").unwrap();
            assert_eq!(rest, "");
            assert_eq!(f.axis, Axis::Y);
            assert_eq!(f.line, 7);
        }

        #[test]
        fn test_parse_folds() {
            let (rest, fs) = many1(fold)("fold along y=7\nfold along x=5\n").unwrap();
            assert_eq!(rest, "");
            assert_eq!(fs.len(), 2);
            assert_eq!(fs[0].axis, Axis::Y);
            assert_eq!(fs[0].line, 7);
            assert_eq!(fs[1].axis, Axis::X);
            assert_eq!(fs[1].line, 5);
        }

        #[test]
        fn test_full_parse() {
            let input = "0,14\n9,10\n0,3\n10,4\n4,11\n6,0\n6,12\n4,1\n0,13\n10,12\n3,4\n3,0\n8,4\n1,10\n2,14\n8,10\n9,0\n\nfold along y=7\nfold along x=5\n";
            parse_input(input).unwrap();
        }
    }
}
