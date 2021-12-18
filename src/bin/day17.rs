use itertools::Itertools;
use std::fs;
use std::num::ParseIntError;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(parse(from_os_str))]
    input: PathBuf,
}

#[derive(Debug, Clone, Copy)]
struct Range {
    min: i64,
    max: i64,
}

impl Range {
    fn contains(&self, val: i64) -> bool {
        self.min <= val && val <= self.max
    }
}

impl FromStr for Range {
    type Err = ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s[2..]
            .split("..")
            .map(i64::from_str)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Range {
            min: parts[0],
            max: parts[1],
        })
    }
}

fn parse_ranges<P: AsRef<Path>>(input: P) -> (Range, Range) {
    let text = fs::read_to_string(input).unwrap();
    text[13..]
        .trim_end()
        .split(", ")
        .map(Range::from_str)
        .map(Result::unwrap)
        .collect_tuple()
        .unwrap()
}

fn max_x_distance(velocity: i64) -> i64 {
    velocity * (velocity + 1) / 2
}

fn find_min_x_velocity(x_range: Range) -> i64 {
    for v in 1.. {
        if max_x_distance(v) >= x_range.min {
            return v;
        }
    }
    panic!("Unhittable")
}
fn find_max_x_velocity(x_range: Range) -> i64 {
    x_range.max
}

fn find_min_y_velocity(y_range: Range) -> i64 {
    y_range.min
}
fn find_max_y_velocity(y_range: Range) -> i64 {
    assert!(y_range.min < 0);
    -y_range.min - 1
}

fn max_height_for_velocity(velocity: i64) -> i64 {
    velocity * (velocity + 1) / 2
}

fn find_max_height(y_range: Range) -> i64 {
    let max_velocity = find_max_y_velocity(y_range);
    max_height_for_velocity(max_velocity)
}

fn y_intercepts(y_range: Range, dy: i64) -> impl Iterator<Item = i64> {
    let (base_t, v) = if dy > 0 {
        (dy * 2 + 1, dy + 1)
    } else {
        (0, -dy)
    };

    (0..)
        .map(move |dt| (base_t + dt, -(dt * v + (dt - 1) * dt / 2)))
        .skip_while(move |&(_, y)| y > y_range.max)
        .take_while(move |&(_, y)| y >= y_range.min)
        .map(|(t, _)| t)
}

fn x_after(dx: i64, t: i64) -> i64 {
    if t >= dx {
        max_x_distance(dx)
    } else {
        max_x_distance(dx) - max_x_distance(dx - t)
    }
}

fn hits(dx: i64, dy: i64, x_range: Range, y_range: Range) -> bool {
    y_intercepts(y_range, dy).any(|t| x_range.contains(x_after(dx, t)))
}

fn find_intercept(init_dx: i64, init_dy: i64, x_range: Range, y_range: Range) -> Option<i64> {
    let mut x = 0;
    let mut y = 0;
    let mut dx = init_dx;
    let mut dy = init_dy;

    for t in 0.. {
        if x > x_range.max || y < y_range.min {
            return None;
        }

        if x_range.contains(x) && y_range.contains(y) {
            return Some(t);
        }

        x += dx;
        y += dy;

        if dx > 0 {
            dx -= 1;
        }
        dy -= 1;
    }
    panic!("Unhittable");
}

#[allow(clippy::suspicious_map)]
fn num_valid_velocities(x_range: Range, y_range: Range) -> usize {
    let min_x_velocity = find_min_x_velocity(x_range);
    let max_x_velocity = find_max_x_velocity(x_range);
    let min_y_velocity = find_min_y_velocity(y_range);
    let max_y_velocity = find_max_y_velocity(y_range);

    (min_x_velocity..=max_x_velocity)
        .cartesian_product(min_y_velocity..=max_y_velocity)
        .filter(|&(dx, dy)| hits(dx, dy, x_range, y_range))
        .map(|(dx, dy)| {
            find_intercept(dx, dy, x_range, y_range)
                .unwrap_or_else(|| panic!("{}, {} missed target", dx, dy))
        })
        .count()
}

fn main() {
    let opt = Opt::from_args();
    let (x_range, y_range) = parse_ranges(opt.input);
    let max_height = find_max_height(y_range);
    println!("{}", max_height);

    let num_velocities = num_valid_velocities(x_range, y_range);
    println!("{}", num_velocities);
}
