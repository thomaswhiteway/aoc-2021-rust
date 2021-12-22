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

trait CubeMap {
    fn new() -> Self;
    fn apply(&mut self, instruction: &Instruction);
    fn num_cubes_on(&self) -> usize;
}

#[derive(Debug)]
struct Instruction {
    on: bool,
    region: Region,
}


struct SmallMap(HashSet<Vector3<i64>>);

impl CubeMap for SmallMap {
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

#[derive(Clone)]
struct Range<T> {
    start: i64,
    contents: T
}

struct LargeMap(Vec<Range<Vec<Range<Vec<Range<bool>>>>>>);

impl LargeMap {
    fn split_at<T: Default + Clone>(val: i64, ranges: &mut Vec<Range<T>>) -> usize {
        if let Some(index) = Self::find_range_index(val, ranges) {
            if ranges[index as usize].start != val {
                ranges.insert(index+1, Range { start: val, contents: ranges[index].contents.clone() });
                index + 1
            } else {
                index
            }
        } else  {
            ranges.insert(0, Range{ start: val, contents: Default::default() });
            0
        }
    }

    fn find_range_index<T>(val: i64, ranges: &[Range<T>]) -> Option<usize> {
        if ranges.is_empty() || val < ranges[0].start {
            None
        } else {
            Some(ranges.iter().enumerate().find_map(|(index, range)| if range.start > val { Some(index - 1) } else { None }).unwrap_or_else(|| ranges.len() - 1))
        }
    }

    fn get_sections<T>(ranges: &[Range<T>]) -> impl Iterator<Item=(&T, i64)> {
        ranges.iter().tuple_windows().map(|(range, next_range)| (&range.contents, next_range.start - range.start))
    }
}

impl CubeMap for LargeMap {
    fn new() -> Self {
        LargeMap(vec![])
    }

    fn apply(&mut self, instruction: &Instruction) {

        let x_ranges = &mut self.0;

        let x_start_index = Self::split_at(instruction.region.min[0], x_ranges);
        let x_end_index = Self::split_at(instruction.region.max[0] + 1, x_ranges);

        for x_index in x_start_index..x_end_index {
            let y_ranges = &mut x_ranges[x_index].contents;

            let y_start_index = Self::split_at(instruction.region.min[1], y_ranges);
            let y_end_index = Self::split_at(instruction.region.max[1] + 1, y_ranges);

            for y_index in y_start_index..y_end_index {
                let z_ranges = &mut y_ranges[y_index].contents;

                let z_start_index = Self::split_at(instruction.region.min[2], z_ranges);
                let z_end_index = Self::split_at(instruction.region.max[2] + 1, z_ranges);

                for z_index in z_start_index..z_end_index {
                    z_ranges[z_index].contents = instruction.on;
                }
            }
        }
    }

    fn num_cubes_on(&self) -> usize {
        let mut total =0;

        let x_ranges = &self.0;
        for (y_ranges, x_width) in Self::get_sections(x_ranges) {
            for (z_ranges, y_width) in Self::get_sections(y_ranges) {
                for (on, z_width) in Self::get_sections(z_ranges) {
                    if *on {
                        total += (x_width * y_width * z_width) as usize;
                    }
                }
            }
        }

        total
    }
}

fn parse_instructions<P: AsRef<Path>>(input: P) -> Box<[Instruction]> {
    let data = std::fs::read_to_string(input).unwrap();
    parsing::instructions(&data).unwrap().1
}

fn run<M: CubeMap>(instructions: &[Instruction]) {
    let mut cube_map = M::new();
    for instruction in instructions.iter() {
        cube_map.apply(instruction)
    }

    println!("{}", cube_map.num_cubes_on());

}

fn main() {
    let opt = Opt::from_args();

    let instructions = parse_instructions(opt.input);

    run::<SmallMap>(&instructions);
    run::<LargeMap>(&instructions);
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
