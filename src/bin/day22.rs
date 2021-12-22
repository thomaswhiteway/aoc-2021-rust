use itertools::Itertools;
use structopt::StructOpt;
use nalgebra::{Vector3, vector};
use std::path::{Path, PathBuf};
use std::collections::HashSet;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(parse(from_os_str))]
    input: PathBuf,
}

#[derive(Debug)]
struct Region {
    min: Vector3<i64>,
    max: Vector3<i64>,
}

impl Region {
    fn intersect(&self, other: &Self) -> Self {
        let min = vector![
            std::cmp::max(self.min[0], other.min[0]),
            std::cmp::max(self.min[1], other.min[1]),
            std::cmp::max(self.min[2], other.min[2])
        ];
        let max = vector![
            std::cmp::min(self.max[0], other.max[0]),
            std::cmp::min(self.max[1], other.max[1]),
            std::cmp::min(self.max[2], other.max[2])
        ];
        Region { min, max }
    }

    fn all_points(&self) -> impl Iterator<Item=Vector3<i64>> {
        (0..3).map(|axis| self.min[axis]..=self.max[axis])
            .multi_cartesian_product()
            .map(Vector3::from_vec)
    }
}

#[derive(Debug)]
struct Instruction {
    on: bool,
    region: Region,
}


struct SmallMap(HashSet<Vector3<i64>>);

impl SmallMap {
    fn new() -> Self {
        SmallMap(HashSet::new())
    }

    fn apply(&mut self, instruction: &Instruction) {
        let considered_region = Region {
            min: vector![-50, -50, -50],
            max: vector![50, 50, 50]
        };
        let range = instruction.region.intersect(&considered_region);
        for pos in range.all_points() {
            if instruction.on {
                self.0.insert(pos);
            } else {
                self.0.remove(&pos);
            }
        }
    }

    fn num_cubes_on(&self) -> usize {
        self.0.len()
    }
}

fn parse_instructions<P: AsRef<Path>>(input: P) -> Box<[Instruction]> {
    let data = std::fs::read_to_string(input).unwrap();
    parsing::instructions(&data).unwrap().1
}

fn main() {
    let opt = Opt::from_args();

    let instructions = parse_instructions(opt.input);

    let mut small_map = SmallMap::new();
    for instruction in instructions.iter() {
        small_map.apply(instruction)
    }

    println!("{}", small_map.num_cubes_on());
}

mod parsing {
    use super::*;

    use nalgebra::vector;
    use nom::bytes::complete::tag;
    use nom::character::complete::one_of;
    use nom::combinator::{map, map_res, recognize};
    use nom::multi::{many1, separated_list1};
    use nom::sequence::separated_pair;
    use nom::IResult;
    use std::str::FromStr;
    use nom::branch::alt;

    fn number(input: &str) -> IResult<&str, i64> {
        map_res(recognize(many1(one_of("-0123456789"))), i64::from_str)(input)
    }

    pub fn range(input: &str) -> IResult<&str, (i64, i64)> {
        separated_pair(number, tag(".."), number)(input)
    }

    fn command(input: &str) -> IResult<&str, bool> {
        alt((
            map(tag("on"), |_| true),
            map(tag("off"), |_| false),
        ))(input)
    }

    fn instruction(input: &str) -> IResult<&str, Instruction> {
        let (input, on) = command(input)?;
        let (input, _) = tag(" x=")(input)?;
        let (input, x_range) = range(input)?;
        let (input, _) = tag(",y=")(input)?;
        let (input, y_range) = range(input)?;
        let (input, _) = tag(",z=")(input)?;
        let (input, z_range) = range(input)?;
        Ok((input, Instruction {
            on,
            region: Region {
                min: vector![x_range.0, y_range.0, z_range.0],
                max: vector![x_range.1, y_range.1, z_range.1],

            }
        }))
    }

    pub(super) fn instructions(input: &str) -> IResult<&str, Box<[Instruction]>> {
        map(separated_list1(tag("\n"), instruction), Vec::into_boxed_slice)(input)
    }
}
