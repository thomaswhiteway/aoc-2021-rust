use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(parse(from_os_str))]
    input: PathBuf,
}

#[derive(Debug)]
#[allow(dead_code)]
struct Outcome {
    winner: usize,
    loser: usize,
    scores: [usize; 2],
    num_die_rolls: usize,
}

struct Die {
    last_roll: usize,
    num_rolls: usize,
}

impl Die {
    fn new() -> Self {
        Die { last_roll: 100, num_rolls: 0 }
    }
}

impl Iterator for Die {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        self.last_roll = if self.last_roll < 100 {
            self.last_roll + 1
        } else {
            1
        };
        self.num_rolls += 1;
        Some(self.last_roll)
    }
}

fn parse_player_starts<P: AsRef<Path>>(input: P) -> [usize; 2] {
    let reader = BufReader::new(File::open(input).unwrap());
    reader
        .lines()
        .map(Result::unwrap)
        .map(|line| line.split(": ").nth(1).unwrap().parse().unwrap())
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
}

fn other_player(player: usize) -> usize {
    1 - player
}

fn play_game(start_pos: [usize; 2]) -> Outcome {
    let mut positions = start_pos;
    let mut scores = [0; 2];
    let mut die = Die::new();
    let mut player = 0;

    loop {
        let rolls = [die.next().unwrap(), die.next().unwrap(), die.next().unwrap()];
        let next_move = rolls.iter().sum::<usize>();
        positions[player] += next_move;
        while positions[player] > 10 {
            positions[player] -= 10;
        }
        scores[player] += positions[player];

        if scores[player] >= 1000 {
            break Outcome {
                winner: player,
                loser: other_player(player),
                scores,
                num_die_rolls: die.num_rolls,
            };
        }

        player = other_player(player);
    }
}
fn main() {
    let opt = Opt::from_args();

    let start_pos = parse_player_starts(opt.input);
    let outcome = play_game(start_pos);
    println!("{}", outcome.scores[outcome.loser] * outcome.num_die_rolls);
}
