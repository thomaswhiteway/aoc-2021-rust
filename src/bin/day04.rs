use std::collections::HashSet;
use std::fs::read_to_string;
use std::path::{Path, PathBuf};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(parse(from_os_str))]
    input: PathBuf,
}

#[derive(Clone, Debug)]
struct Card {
    match_sets: Box<[HashSet<usize>]>,
}

impl Card {
    fn new(grid: &[Box<[usize]>]) -> Self {
        let rows = grid
            .iter()
            .map(|row| row.iter().cloned().collect::<HashSet<_>>());
        let cols =
            (0..grid[0].len()).map(|col| grid.iter().map(|row| row[col]).collect::<HashSet<_>>());
        let match_sets = rows.chain(cols).collect::<Vec<_>>().into_boxed_slice();
        Card { match_sets }
    }

    fn mark(&mut self, num: usize) {
        for set in self.match_sets.iter_mut() {
            set.remove(&num);
        }
    }

    fn unmarked(&self) -> HashSet<usize> {
        self.match_sets
            .iter()
            .fold(HashSet::new(), |current, next| {
                current.union(next).cloned().collect()
            })
    }

    fn has_won(&self) -> bool {
        self.match_sets.iter().any(|set| set.is_empty())
    }
}

type Numbers = Box<[usize]>;
type Cards = Box<[Card]>;

fn read_data<P: AsRef<Path>>(input: P) -> (Numbers, Cards) {
    parsing::game(&read_to_string(input).unwrap()).unwrap().1
}

fn find_winner(inputs: &[usize], cards: &mut [Card]) -> (Card, usize) {
    for num in inputs {
        for card in cards.iter_mut() {
            card.mark(*num);
        }

        if let Some(card) = cards.iter().find(|card| card.has_won()) {
            return (card.clone(), *num);
        }
    }
    panic!("No Winner");
}

fn main() {
    let opt = Opt::from_args();

    let (inputs, mut cards) = read_data(&opt.input);

    let (winning_card, last_number) = find_winner(&inputs, &mut cards);

    let total: usize = winning_card.unmarked().iter().sum();
    let score = total * last_number;

    println!("{}", score);
}

mod parsing {
    use {super::Card, super::Cards, super::Numbers};
    use nom::combinator::recognize;
    use nom::{
        character::complete::{char, one_of},
        combinator::{map, map_res},
        multi::{many0, many1, separated_list1},
        sequence::{preceded, terminated},
        IResult,
    };

    fn number(input: &str) -> IResult<&str, usize> {
        map_res(recognize(many1(one_of("0123456789"))), |val: &str| {
            val.parse()
        })(input)
    }

    fn numbers(input: &str) -> IResult<&str, Numbers> {
        map(
            terminated(separated_list1(char(','), number), char('\n')),
            Vec::into_boxed_slice,
        )(input)
    }

    fn row(input: &str) -> IResult<&str, Box<[usize]>> {
        map(
            terminated(many1(preceded(many0(char(' ')), number)), char('\n')),
            Vec::into_boxed_slice,
        )(input)
    }

    fn card(input: &str) -> IResult<&str, Card> {
        map(many1(row), |grid| Card::new(&grid))(input)
    }

    fn cards(input: &str) -> IResult<&str, Cards> {
        map(separated_list1(char('\n'), card), Vec::into_boxed_slice)(input)
    }

    pub(super) fn game(input: &str) -> IResult<&str, (Numbers, Cards)> {
        let (i, nums) = numbers(input)?;
        let (i, _) = char('\n')(i)?;
        let (i, cards) = cards(i)?;
        Ok((i, (nums, cards)))
    }
}
