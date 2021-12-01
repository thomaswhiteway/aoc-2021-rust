use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(parse(from_os_str))]
    input: PathBuf,
}

fn read_depths<P: AsRef<Path>>(path: &P) -> Box<[u64]> {
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);
    reader
        .lines()
        .map(Result::unwrap)
        .map(|line| line.parse().unwrap())
        .collect::<Vec<_>>()
        .into_boxed_slice()
}

fn count_increases(depths: &[u64]) -> usize {
    depths
        .iter()
        .zip(&depths[1..])
        .filter(|(before, after)| after > before)
        .count()
}

fn main() {
    let opt = Opt::from_args();

    let depths = read_depths(&opt.input);
    let num_increases = count_increases(&depths);
    println!("{}", num_increases);
}
