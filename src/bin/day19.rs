use itertools::Itertools;
use nalgebra::{matrix, vector, SMatrix, SVector};
use std::cmp::{max, min};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(parse(from_os_str))]
    input: PathBuf,
}

type Position = SVector<i32, 3>;

#[derive(Clone)]
struct Scanner {
    index: i32,
    position: Position,
    beacons: HashSet<Position>,
}

impl Scanner {
    fn rotate(&self, rotation: &SMatrix<i32, 3, 3>) -> Self {
        let beacons = self.beacons.iter().map(|pos| rotation * pos).collect();
        Scanner {
            index: self.index,
            position: self.position,
            beacons,
        }
    }

    fn all_translations<'a>(
        &'a self,
        other: &'a Self,
    ) -> impl Iterator<Item = SVector<i32, 3>> + 'a {
        other.beacons.iter().flat_map(|to_beacon| {
            self.beacons
                .iter()
                .map(move |from_beacon| to_beacon - from_beacon)
        })
    }

    fn translate(&self, translation: &SVector<i32, 3>) -> Scanner {
        let position = self.position + translation;
        let beacons = self.beacons.iter().map(|pos| pos + translation).collect();
        Scanner {
            index: self.index,
            position,
            beacons,
        }
    }

    fn overlapping_beacons<'a>(&'a self, other: &'a Self) -> impl Iterator<Item = &Position> + 'a {
        self.beacons.intersection(&other.beacons)
    }

    fn distance_to(&self, other: &Self) -> i32 {
        (self.position - other.position).abs().sum()
    }

    fn translated_overlap(
        &self,
        other: &Self,
        translation: &SVector<i32, 3>,
    ) -> (SVector<i32, 3>, SVector<i32, 3>) {
        let position = self.position + translation;
        (
            vector![
                max(position[0], other.position[0]) - 1000,
                max(position[1], other.position[1]) - 1000,
                max(position[2], other.position[2]) - 1000
            ],
            vector![
                min(position[0], other.position[0]) + 1000,
                min(position[1], other.position[1]) + 1000,
                min(position[2], other.position[2]) + 1000
            ],
        )
    }

    fn beacons_in_range<'a>(
        &'a self,
        overlap: &'a (SVector<i32, 3>, SVector<i32, 3>),
    ) -> impl Iterator<Item = &Position> + 'a {
        let (min, max) = overlap;
        self.beacons.iter().filter(|position| {
            ((position[0] >= min[0])
                && (position[1] >= min[1])
                && (position[2] >= min[2])
                && (position[0] <= max[0])
                && (position[1] <= max[1])
                && (position[2] <= max[2]))
        })
    }
}

fn all_x_rotations() -> impl Iterator<Item = SMatrix<i32, 3, 3>> + Clone {
    [
        matrix![1,  0,  0;
             0,  1,  0;
             0,  0,  1],
        matrix![1,  0,  0;
             0,  0, -1;
             0,  1,  0],
        matrix![1,  0,  0;
             0, -1,  0;
             0,  0, -1],
        matrix![1,  0,  0;
             0,  0,  1;
             0, -1,  0],
    ]
    .into_iter()
}

fn all_y_rotations() -> impl Iterator<Item = SMatrix<i32, 3, 3>> + Clone {
    [
        matrix![ 1,  0,  0;
              0,  1,  0;
              0,  0,  1],
        matrix![ 0,  0, -1;
              0,  1,  0;
              1,  0,  0],
        matrix![-1,  0,  0;
              0,  1,  0;
              0,  0, -1],
        matrix![ 0,  0,  1;
              0,  1,  0;
             -1,  0,  0],
    ]
    .into_iter()
}

fn all_z_rotations() -> impl Iterator<Item = SMatrix<i32, 3, 3>> + Clone {
    [
        matrix![ 1,  0,  0;
              0,  1,  0;
              0,  0,  1],
        matrix![ 0, -1,  0;
              1,  0,  0;
              0,  0,  1],
        matrix![-1,  0,  0;
              0, -1,  0;
              0,  0,  1],
        matrix![ 0,  1,  0;
             -1,  0,  0;
              0,  0,  1],
    ]
    .into_iter()
}

fn all_rotations() -> impl Iterator<Item = SMatrix<i32, 3, 3>> {
    all_x_rotations()
        .cartesian_product(all_y_rotations())
        .map(|(a, b)| a * b)
        .cartesian_product(all_z_rotations())
        .map(|(a, b)| a * b)
}

fn parse_scanners<P: AsRef<Path>>(input: P) -> Box<[Scanner]> {
    let text = std::fs::read_to_string(input).unwrap();
    parsing::scanners(&text).unwrap().1
}

fn find_scanner_to_place(
    placed_scanners: &[Scanner],
    remaining_scanners: &[Scanner],
) -> Option<Scanner> {
    for scanner in remaining_scanners.iter() {
        for placed_scanner in placed_scanners {
            for translation in scanner.all_translations(&placed_scanner) {
                let translated_overlap = scanner.translated_overlap(&placed_scanner, &translation);
                let placed_overlapped_beacons = placed_scanner
                    .beacons_in_range(&translated_overlap)
                    .cloned()
                    .collect::<HashSet<_>>();
                if placed_overlapped_beacons.len() < 12 {
                    continue;
                }
                let orig_overlap = (
                    translated_overlap.0 - translation,
                    translated_overlap.1 - translation,
                );
                let mut translated_overlapped_beacons = scanner
                    .beacons_in_range(&orig_overlap)
                    .map(|pos| pos + translation);
                //     .collect::<HashSet<_>>();

                //if placed_overlapped_beacons == translated_overlapped_beacons {
                if translated_overlapped_beacons.all(|pos| placed_overlapped_beacons.contains(&pos))
                    && scanner.beacons_in_range(&orig_overlap).count()
                        == placed_overlapped_beacons.len()
                {
                    println!("Placed scanner {} at {:?}", scanner.index, translation);
                    return Some(scanner.translate(&translation));
                }
            }
        }
    }

    None
}

fn place_scanners(scanners: &[Scanner]) -> Box<[Scanner]> {
    let rotations = all_rotations().collect::<Vec<_>>();
    let mut placed_scanners = vec![scanners[0].clone()];
    let mut possible_scanners = scanners[1..]
        .iter()
        .flat_map(|scanner| rotations.iter().map(|rotation| scanner.rotate(rotation)))
        .collect::<Vec<_>>();

    while !possible_scanners.is_empty() {
        let scanner = find_scanner_to_place(&placed_scanners, &possible_scanners).unwrap();
        possible_scanners.retain(|s| s.index != scanner.index);
        placed_scanners.push(scanner);
    }

    placed_scanners.into_boxed_slice()
}

fn find_all_positions(scanners: &[Scanner]) -> HashSet<Position> {
    scanners.into_iter().fold(HashSet::new(), |x, y| {
        x.union(&y.beacons).cloned().collect()
    })
}

fn find_max_distance(scanners: &[Scanner]) -> i32 {
    scanners
        .iter()
        .cartesian_product(scanners)
        .map(|(x, y)| x.distance_to(y))
        .max()
        .unwrap()
}

fn main() {
    let opt = Opt::from_args();
    let scanners = parse_scanners(opt.input);

    let placed_scanners = place_scanners(&scanners);
    let all_positions = find_all_positions(&placed_scanners);
    println!("{}", all_positions.len());

    let max_distance = find_max_distance(&placed_scanners);
    println!("{}", max_distance);
}

mod parsing {
    use super::*;

    use nalgebra::vector;
    use nom::bytes::complete::tag;
    use nom::character::complete::one_of;
    use nom::combinator::{map, map_res, recognize};
    use nom::multi::{many1, separated_list1};
    use nom::sequence::tuple;
    use nom::IResult;
    use std::str::FromStr;

    fn number(input: &str) -> IResult<&str, i32> {
        map_res(recognize(many1(one_of("-0123456789"))), i32::from_str)(input)
    }

    pub fn position(input: &str) -> IResult<&str, Position> {
        let (input, x) = number(input)?;
        let (input, _) = tag(",")(input)?;
        let (input, y) = number(input)?;
        let (input, _) = tag(",")(input)?;
        let (input, z) = number(input)?;
        let (input, _) = tag("\n")(input)?;
        Ok((input, vector![x, y, z]))
    }

    fn scanner(input: &str) -> IResult<&str, Scanner> {
        let (input, (_, index, _)) = tuple((tag("--- scanner "), number, tag(" ---\n")))(input)?;
        let (input, positions) = many1(position)(input)?;
        Ok((
            input,
            Scanner {
                index,
                position: vector![0, 0, 0],
                beacons: positions.into_iter().collect(),
            },
        ))
    }

    pub(super) fn scanners(input: &str) -> IResult<&str, Box<[Scanner]>> {
        map(separated_list1(tag("\n"), scanner), Vec::into_boxed_slice)(input)
    }
}
