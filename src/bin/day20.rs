use itertools::Itertools;
use std::collections::HashSet;
use std::fmt::Display;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(parse(from_os_str))]
    input: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Pixel {
    Light,
    Dark,
}

impl Pixel {
    fn other(&self) -> Self {
        match self {
            Pixel::Light => Pixel::Dark,
            Pixel::Dark => Pixel::Light,
        }
    }
}

impl Display for Pixel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Pixel::Dark => write!(f, "."),
            Pixel::Light => write!(f, "#"),
        }
    }
}

impl TryFrom<char> for Pixel {
    type Error = String;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '.' => Ok(Pixel::Dark),
            '#' => Ok(Pixel::Light),
            _ => Err(format!("Invalid Pixel {}", value)),
        }
    }
}

#[derive(Clone, Copy, Hash, Eq, PartialEq, Debug)]
struct Position {
    x: isize,
    y: isize,
}

impl Position {
    fn new(x: isize, y: isize) -> Self {
        Position { x, y }
    }

    fn offset(&self, dx: isize, dy: isize) -> Self {
        Position {
            x: self.x + dx,
            y: self.y + dy,
        }
    }

    fn region(&self) -> impl Iterator<Item = Position> {
        let me = *self;
        (-1..=1)
            .cartesian_product(-1..=1)
            .map(move |(dx, dy)| me.offset(dx, dy))
    }
}

struct Algorithm(Box<[Pixel]>);

impl Algorithm {
    fn all_dark_region() -> usize {
        0
    }

    fn all_light_region() -> usize {
        0x1FF
    }

    fn get(&self, key: usize) -> Pixel {
        self.0[key]
    }

    fn key_for_pos(image: &Image, pos: &Position) -> usize {
        (-1..=1)
            .flat_map(|y| (-1..=1).map(move |x| pos.offset(x, y)))
            .map(|pos| {
                if image.pixel_at(&pos) == Pixel::Light {
                    1
                } else {
                    0
                }
            })
            .fold(0, |acc, bit| (acc << 1) | bit)
    }

    fn next_pixel(&self, image: &Image, pos: &Position) -> Pixel {
        self.get(Algorithm::key_for_pos(image, pos))
    }
}

impl FromStr for Algorithm {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Algorithm(
            s.chars()
                .map(Pixel::try_from)
                .map(Result::unwrap)
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        ))
    }
}

struct Image {
    default: Pixel,
    non_default: HashSet<Position>,
}

impl Image {
    fn apply_algorithm(&self, algorithm: &Algorithm) -> Image {
        let default = algorithm.get(match self.default {
            Pixel::Dark => Algorithm::all_dark_region(),
            Pixel::Light => Algorithm::all_light_region(),
        });

        let to_consider = self
            .non_default
            .iter()
            .flat_map(|pos| pos.region())
            .collect::<HashSet<_>>();
        let non_default = to_consider
            .into_iter()
            .filter(|pos| algorithm.next_pixel(self, pos) != default)
            .collect();

        Image {
            default,
            non_default,
        }
    }

    fn num_light_pixels(&self) -> Option<usize> {
        if self.default == Pixel::Dark {
            Some(self.non_default.len())
        } else {
            None
        }
    }

    fn pixel_at(&self, pos: &Position) -> Pixel {
        if self.non_default.contains(pos) {
            self.default.other()
        } else {
            self.default
        }
    }

    fn y_range(&self) -> impl Iterator<Item = isize> {
        let min_y = self.non_default.iter().map(|pos| pos.y).min().unwrap();
        let max_y = self.non_default.iter().map(|pos| pos.y).max().unwrap();
        min_y..=max_y
    }

    fn x_range(&self) -> impl Iterator<Item = isize> {
        let min_x = self.non_default.iter().map(|pos| pos.x).min().unwrap();
        let max_x = self.non_default.iter().map(|pos| pos.x).max().unwrap();
        min_x..=max_x
    }
}

fn read_image_enhancement_algorithm(reader: &mut impl BufRead) -> Algorithm {
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    line.trim_end().parse().unwrap()
}

fn read_image(reader: impl BufRead) -> Image {
    let non_default = reader
        .lines()
        .map(Result::unwrap)
        .enumerate()
        .flat_map(|(y, line)| {
            line.trim_end()
                .chars()
                .map(Pixel::try_from)
                .map(Result::unwrap)
                .enumerate()
                .filter_map(|(x, pixel)| {
                    if pixel != Pixel::Dark {
                        Some(Position::new(x as isize, y as isize))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        })
        .collect();
    Image {
        default: Pixel::Dark,
        non_default,
    }
}

fn parse_input<P: AsRef<Path>>(input: P) -> (Algorithm, Image) {
    let file = File::open(input).unwrap();
    let mut reader = BufReader::new(file);
    let algo = read_image_enhancement_algorithm(&mut reader);
    reader.read_line(&mut String::new()).unwrap();
    let image = read_image(&mut reader);
    (algo, image)
}

#[allow(dead_code)]
fn display_image(image: &Image) {
    for y in image.y_range() {
        for x in image.x_range() {
            print!("{}", image.pixel_at(&Position::new(x, y)))
        }
        println!();
    }
    println!();
}

fn main() {
    let opt = Opt::from_args();

    let (algo, mut image) = parse_input(opt.input);

    for index in 1..=50 {
        image = image.apply_algorithm(&algo);
        if let Some(num) = image.num_light_pixels() {
            println!("{}: {}", index, num);
        } else {
            println!("{}: inf", index);
        }
    }
}
