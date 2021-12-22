use itertools::Itertools;
use nalgebra::{vector, Vector3};
use std::path::{Path, PathBuf};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(parse(from_os_str))]
    input: PathBuf,
}

#[derive(Debug, Clone)]
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
}
#[derive(Debug, Clone)]
struct Instruction {
    on: bool,
    region: Region,
}

impl Instruction {
    fn restrict(&self, region: &Region) -> Self {
        Instruction {
            on: self.on,
            region: self.region.intersect(region),
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
struct Range<T> {
    start: i64,
    contents: T,
}

#[derive(Default, Clone, PartialEq, Eq)]
struct Partition<T>(Vec<Range<T>>);

impl<T: Default + Clone + Eq> Partition<T> {
    fn new() -> Self {
        Partition(Vec::new())
    }

    fn find_range_index(&self, val: i64) -> Option<usize> {
        if self.0.is_empty() || val < self.0[0].start {
            None
        } else {
            Some(
                self.0
                    .iter()
                    .enumerate()
                    .find_map(|(index, range)| {
                        if range.start > val {
                            Some(index - 1)
                        } else {
                            None
                        }
                    })
                    .unwrap_or_else(|| self.0.len() - 1),
            )
        }
    }

    fn split_at(&mut self, val: i64) -> usize {
        if let Some(index) = self.find_range_index(val) {
            if self.0[index as usize].start != val {
                self.0.insert(
                    index + 1,
                    Range {
                        start: val,
                        contents: self.0[index].contents.clone(),
                    },
                );
                index + 1
            } else {
                index
            }
        } else {
            self.0.insert(
                0,
                Range {
                    start: val,
                    contents: Default::default(),
                },
            );
            0
        }
    }

    fn normalize(&mut self) {
        let mut index = 0;
        while index < self.0.len() - 1 {
            if self.0[index].contents == self.0[index + 1].contents {
                self.0.remove(index + 1);
            }

            index += 1;
        }
    }

    fn sections(&self) -> impl Iterator<Item = (&T, i64)> {
        self.0
            .iter()
            .tuple_windows()
            .map(|(range, next_range)| (&range.contents, next_range.start - range.start))
    }
}

trait Update {
    fn update(&mut self, region: &Region, depth: usize, value: bool);
}

impl Update for bool {
    fn update(&mut self, _region: &Region, _depth: usize, value: bool) {
        *self = value;
    }
}

impl<T: Update + Clone + Default + Eq> Update for Partition<T> {
    fn update(&mut self, region: &Region, depth: usize, value: bool) {
        let start_index = self.split_at(region.min[depth]);
        let end_index = self.split_at(region.max[depth] + 1);

        for range in self.0.iter_mut().take(end_index).skip(start_index) {
            range.contents.update(region, depth + 1, value);
        }

        self.normalize();
    }
}

struct CubeMap(Partition<Partition<Partition<bool>>>);

impl CubeMap {
    fn new() -> Self {
        CubeMap(Partition::new())
    }

    fn apply(&mut self, instruction: &Instruction) {
        self.0.update(&instruction.region, 0, instruction.on);
    }

    fn num_cubes_on(&self) -> usize {
        let mut total = 0;

        let x_ranges = &self.0;
        for (y_ranges, x_width) in x_ranges.sections() {
            for (z_ranges, y_width) in y_ranges.sections() {
                for (on, z_width) in z_ranges.sections() {
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

fn run(instructions: &[Instruction], region: Option<Region>) {
    let mut cube_map = CubeMap::new();
    for instruction in instructions.iter() {
        if let Some(region) = &region {
            cube_map.apply(&instruction.restrict(region));
        } else {
            cube_map.apply(instruction);
        }
    }

    println!("{}", cube_map.num_cubes_on());
}

fn main() {
    let opt = Opt::from_args();

    let instructions = parse_instructions(opt.input);

    run(
        &instructions,
        Some(Region {
            min: vector![-50, -50, -50],
            max: vector![50, 50, 50],
        }),
    );
    run(&instructions, None);
}

mod parsing {
    use super::*;

    use nalgebra::vector;
    use nom::branch::alt;
    use nom::bytes::complete::tag;
    use nom::character::complete::one_of;
    use nom::combinator::{map, map_res, recognize};
    use nom::multi::{many1, separated_list1};
    use nom::sequence::separated_pair;
    use nom::IResult;
    use std::str::FromStr;

    fn number(input: &str) -> IResult<&str, i64> {
        map_res(recognize(many1(one_of("-0123456789"))), i64::from_str)(input)
    }

    pub fn range(input: &str) -> IResult<&str, (i64, i64)> {
        separated_pair(number, tag(".."), number)(input)
    }

    fn command(input: &str) -> IResult<&str, bool> {
        alt((map(tag("on"), |_| true), map(tag("off"), |_| false)))(input)
    }

    fn instruction(input: &str) -> IResult<&str, Instruction> {
        let (input, on) = command(input)?;
        let (input, _) = tag(" x=")(input)?;
        let (input, x_range) = range(input)?;
        let (input, _) = tag(",y=")(input)?;
        let (input, y_range) = range(input)?;
        let (input, _) = tag(",z=")(input)?;
        let (input, z_range) = range(input)?;
        Ok((
            input,
            Instruction {
                on,
                region: Region {
                    min: vector![x_range.0, y_range.0, z_range.0],
                    max: vector![x_range.1, y_range.1, z_range.1],
                },
            },
        ))
    }

    pub(super) fn instructions(input: &str) -> IResult<&str, Box<[Instruction]>> {
        map(
            separated_list1(tag("\n"), instruction),
            Vec::into_boxed_slice,
        )(input)
    }
}
