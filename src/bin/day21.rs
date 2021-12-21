use std::cmp::Ord;
use std::collections::{hash_map, BinaryHeap, HashMap};
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
struct DeterministicOutcome {
    winner: usize,
    loser: usize,
    scores: [usize; 2],
    num_die_rolls: usize,
}

struct DeterministicDie {
    last_roll: usize,
    num_rolls: usize,
}

impl DeterministicDie {
    fn new() -> Self {
        DeterministicDie {
            last_roll: 100,
            num_rolls: 0,
        }
    }
}

impl Iterator for DeterministicDie {
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

fn play_deterministic_game(start_pos: [usize; 2]) -> DeterministicOutcome {
    let mut positions = start_pos;
    let mut scores = [0; 2];
    let mut die = DeterministicDie::new();
    let mut player = 0;

    loop {
        let rolls = [
            die.next().unwrap(),
            die.next().unwrap(),
            die.next().unwrap(),
        ];
        let next_move = rolls.iter().sum::<usize>();
        positions[player] += next_move;
        while positions[player] > 10 {
            positions[player] -= 10;
        }
        scores[player] += positions[player];

        if scores[player] >= 1000 {
            break DeterministicOutcome {
                winner: player,
                loser: other_player(player),
                scores,
                num_die_rolls: die.num_rolls,
            };
        }

        player = other_player(player);
    }
}

struct QuantumOutcome {
    winning_universes: [usize; 2],
}

#[derive(Hash, PartialEq, Eq, Debug, Clone, Copy)]
struct PlayerState {
    position: usize,
    score: usize,
}

impl PlayerState {
    fn new(position: usize) -> Self {
        PlayerState { position, score: 0 }
    }
}

#[derive(Hash, PartialEq, Eq, Debug, Clone, Copy)]
struct UniverseState {
    next_player: usize,
    players: [PlayerState; 2],
}

impl UniverseState {
    fn new(start_pos: [usize; 2]) -> Self {
        UniverseState {
            next_player: 0,
            players: [
                PlayerState::new(start_pos[0]),
                PlayerState::new(start_pos[1]),
            ],
        }
    }

    fn winning_player(&self) -> Option<usize> {
        self.players.iter().enumerate().find_map(|(index, player)| {
            if player.score >= 21 {
                Some(index)
            } else {
                None
            }
        })
    }

    fn with_roll(&self, roll: usize) -> Self {
        let mut new_state = *self;
        {
            let player = &mut new_state.players[self.next_player];
            player.position += roll;
            while player.position > 10 {
                player.position -= 10;
            }
            player.score += player.position;
        }
        new_state.next_player = other_player(self.next_player);
        new_state
    }
}

impl PartialOrd for UniverseState {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(
            Ord::cmp(&self.players[0].score, &other.players[0].score)
                .then_with(|| Ord::cmp(&self.players[1].score, &other.players[1].score))
                .then_with(|| Ord::cmp(&self.players[0].position, &other.players[0].position))
                .then_with(|| Ord::cmp(&self.players[1].position, &other.players[1].position))
                .then_with(|| Ord::cmp(&self.next_player, &other.next_player))
                .reverse(),
        )
    }
}

impl Ord for UniverseState {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

fn get_splits() -> [usize; 10] {
    let mut splits = [0; 10];

    for x in 1..=3 {
        for y in 1..=3 {
            for z in 1..=3 {
                splits[x + y + z] += 1;
            }
        }
    }

    splits
}

fn count_winning_universes(universes: &HashMap<UniverseState, usize>, player: usize) -> usize {
    universes
        .iter()
        .filter(|(state, _)| state.winning_player() == Some(player))
        .map(|(_, count)| count)
        .sum()
}

fn play_quantum_game(start_pos: [usize; 2]) -> QuantumOutcome {
    let splits = get_splits();

    let initial_state = UniverseState::new(start_pos);

    let mut universes: HashMap<UniverseState, usize> = HashMap::new();
    universes.insert(initial_state, 1);

    let mut in_play_states = BinaryHeap::new();
    in_play_states.push(initial_state);

    while let Some(state) = in_play_states.pop() {
        let num_universes = universes.remove(&state).unwrap();

        for (roll, &num_new_universes) in splits.iter().enumerate() {
            let new_state = state.with_roll(roll);
            let total_new_universes = num_universes * num_new_universes;

            match universes.entry(new_state) {
                hash_map::Entry::Occupied(mut entry) => *entry.get_mut() += total_new_universes,
                hash_map::Entry::Vacant(entry) => {
                    entry.insert(total_new_universes);

                    if new_state.winning_player().is_none() {
                        in_play_states.push(new_state);
                    }
                }
            }
        }
    }

    QuantumOutcome {
        winning_universes: [
            count_winning_universes(&universes, 0),
            count_winning_universes(&universes, 1),
        ],
    }
}

fn main() {
    let opt = Opt::from_args();

    let start_pos = parse_player_starts(opt.input);

    let outcome = play_deterministic_game(start_pos);
    println!("{}", outcome.scores[outcome.loser] * outcome.num_die_rolls);

    let outcome = play_quantum_game(start_pos);
    println!("{}", outcome.winning_universes.iter().max().unwrap());
}
