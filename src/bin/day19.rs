use itertools::Itertools;
use nalgebra::{SVector, SMatrix, matrix};
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
    beacons: HashSet<Position>,
}

impl Scanner {
    fn rotate(&self, rotation: &SMatrix<i32, 3, 3>) -> Self {
        let beacons = self.beacons.iter().map(|pos| rotation * pos).collect();
        Scanner { index: self.index, beacons }
    }

    fn all_translations<'a>(&'a self, other: &'a Self) -> impl Iterator<Item=SVector<i32, 3>> + 'a {
        other.beacons.iter().flat_map(|to_beacon| self.beacons.iter().map(move |from_beacon| to_beacon - from_beacon))
    }

    fn translate(&self, translation: &SVector<i32, 3>) -> Scanner {
        let beacons = self.beacons.iter().map(|pos| pos + translation).collect();
        Scanner { index: self.index, beacons }
    }

    fn overlapping_beacons<'a>(&'a self, other: &'a Self) -> impl Iterator<Item=&Position> + 'a {
        self.beacons.union(&other.beacons)
    }
}

fn all_x_rotations() -> impl Iterator<Item=SMatrix<i32, 3, 3>> + Clone {
    [matrix![1,  0,  0; 
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
    ].into_iter()
}

fn all_y_rotations() -> impl Iterator<Item=SMatrix<i32, 3, 3>> + Clone {
    [matrix![ 1,  0,  0; 
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
    ].into_iter()
}

fn all_z_rotations() -> impl Iterator<Item=SMatrix<i32, 3, 3>> + Clone {
    [matrix![ 1,  0,  0; 
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
    ].into_iter()
}

fn all_rotations() -> impl Iterator<Item=SMatrix<i32, 3, 3>> {
    all_x_rotations().cartesian_product(all_y_rotations()).map(|(a, b)| a * b).cartesian_product(all_z_rotations()).map(|(a, b)| a * b)
}

fn parse_scanners<P: AsRef<Path>>(input: P) -> Box<[Scanner]> {
    let text = std::fs::read_to_string(input).unwrap();
    parsing::scanners(&text).unwrap().1
}

fn find_scanner_to_place(placed_scanners: &[Scanner], remaining_scanners: &[Scanner]) -> Option<(usize, Scanner)> {
    let rotations = all_rotations().collect::<Vec<_>>();
    for (index, scanner) in remaining_scanners.iter().enumerate() {
        for placed_scanner in placed_scanners {
            for rotation in rotations.iter() {
                let scanner = scanner.rotate(&rotation);
                for translation in scanner.all_translations(&placed_scanner) {
                    let scanner = scanner.translate(&translation);
                    if scanner.overlapping_beacons(&placed_scanner).count() >= 12 {
                        println!("Placed scanner {} at {:?}", scanner.index, translation);
                        return Some((index, scanner));
                    }
                }
            }
        }
    }

    None
}

fn find_all_positions(scanners: &[Scanner]) -> HashSet<Position> {
    let mut placed_scanners = vec![scanners[0].clone()];
    let mut remaining_scanners = scanners[1..].to_vec();

    while !remaining_scanners.is_empty() {
        let (index, scanner) = find_scanner_to_place(&placed_scanners, &remaining_scanners).unwrap();
        remaining_scanners.remove(index);
        placed_scanners.push(scanner);
    }

    placed_scanners.into_iter().fold(HashSet::new(), |x, y| x.union(&y.beacons).cloned().collect())
    
}

fn main() {
    let opt = Opt::from_args();
    let scanners = parse_scanners(opt.input);

    let all_positions = find_all_positions(&scanners);
    println!("{}", all_positions.len());
}


mod parsing {
    use super::*;

    use nom::bytes::complete::tag;
    use nom::character::complete::one_of;
    use nom::combinator::{map, map_res, recognize};
    use nom::multi::{many1, separated_list1};
    use nom::sequence::tuple;
    use nom::IResult;
    use std::str::FromStr;
    use nalgebra::vector;

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
        Ok((input, Scanner {
            index,
            beacons: positions.into_iter().collect()
        }))
    }

    pub(super) fn scanners(input: &str) -> IResult<&str, Box<[Scanner]>> {
        map(separated_list1(tag("\n"), scanner), Vec::into_boxed_slice)(input)
    }
}