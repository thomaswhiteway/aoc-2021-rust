use either::Either;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(parse(from_os_str))]
    input: PathBuf,
}

type CucumberMap = Vec<Vec<char>>;

fn read_map<P: AsRef<Path>>(input: P) -> CucumberMap {
    BufReader::new(File::open(input).unwrap())
        .lines()
        .map(Result::unwrap)
        .map(|line| line.chars().collect::<Vec<_>>())
        .collect::<Vec<_>>()
}

fn set(map: &mut CucumberMap, (x, y): (usize, usize), c: char) {
    map[y][x] = c;
}

fn get(map: &CucumberMap, (x, y): (usize, usize)) -> char {
    map[y][x]
}

fn move_cucumbers(map: &mut CucumberMap, direction: char) -> bool {
    let width = map[0].len();
    let height = map.len();

    let potential_moves = if direction == '>' {
        Either::Left((0..height).flat_map(|y| {
            (0..width)
                .zip((0..width).cycle().skip(1))
                .map(move |(x, next_x)| ((x, y), (next_x, y)))
        }))
    } else {
        Either::Right((0..width).flat_map(|x| {
            (0..height)
                .zip((0..height).cycle().skip(1))
                .map(move |(y, next_y)| ((x, y), (x, next_y)))
        }))
    };

    let moves = potential_moves
        .filter(|&(from, to)| get(map, from) == direction && get(map, to) == '.')
        .collect::<Vec<_>>();

    let moved = !moves.is_empty();

    for (from, to) in moves {
        set(map, from, '.');
        set(map, to, direction);
    }

    moved
}

#[allow(dead_code)]
fn print_map(map: &CucumberMap) {
    for row in map {
        for c in row {
            print!("{}", c)
        }
        println!()
    }
    println!()
}

fn move_until_gridlock(map: &CucumberMap) -> usize {
    let mut map = map.clone();

    for step in 1.. {
        let mut updated = false;
        updated |= move_cucumbers(&mut map, '>');
        updated |= move_cucumbers(&mut map, 'v');

        if !updated {
            print_map(&map);
            return step;
        }
    }

    unreachable!()
}

fn main() {
    let opt = Opt::from_args();
    let map = read_map(opt.input);

    let step = move_until_gridlock(&map);
    println!("{}", step);
}
