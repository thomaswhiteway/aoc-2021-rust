use itertools::Itertools;
use std::collections::HashMap;
use std::fs;
use std::hash::Hash;
use std::path::{Path, PathBuf};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(parse(from_os_str))]
    input: PathBuf,
}

type Rules = HashMap<(char, char), char>;

type Inputs = (Box<[char]>, Rules);

fn parse_input<P: AsRef<Path>>(input: P) -> Inputs {
    parsing::parse_input(&fs::read_to_string(input).unwrap()).unwrap()
}

fn apply_rules(rules: &Rules, current: Box<[char]>) -> Box<[char]> {
    current
        .iter()
        .map(Option::Some)
        .interleave(
            current
                .iter()
                .tuple_windows()
                .map(|(&a, &b)| rules.get(&(a, b))),
        )
        .flatten()
        .cloned()
        .collect::<Vec<_>>()
        .into_boxed_slice()
}

fn count<V: Eq + Clone + Hash, I: IntoIterator<Item = V>>(sequence: I) -> HashMap<V, usize> {
    let mut counts = HashMap::new();

    for v in sequence {
        *counts.entry(v.clone()).or_default() += 1;
    }

    counts
}

fn main() {
    let opt = Opt::from_args();

    let (template, rules) = parse_input(opt.input);

    let mut current = template;
    for _ in 0..10 {
        current = apply_rules(&rules, current);
    }

    let counts = count(current.iter().cloned());
    let max = counts.values().max().unwrap();
    let min = counts.values().min().unwrap();

    println!("After 10 steps: {}", max - min);
}

mod parsing {
    use crate::Inputs;

    use nom::bytes::complete::tag;
    use nom::character::complete::one_of;
    use nom::combinator::map;
    use nom::multi::many1;
    use nom::sequence::{pair, terminated};
    use nom::IResult;

    fn template(input: &str) -> IResult<&str, Box<[char]>> {
        map(terminated(many1(upper), tag("\n")), Vec::into_boxed_slice)(input)
    }

    fn upper(input: &str) -> IResult<&str, char> {
        one_of("ABCDEFGHIJKLMNOPQRSTUVWXYZ")(input)
    }

    fn rule(input: &str) -> IResult<&str, ((char, char), char)> {
        let (input, pattern) = pair(upper, upper)(input)?;
        let (input, _) = tag(" -> ")(input)?;
        let (input, insert) = upper(input)?;
        let (input, _) = tag("\n")(input)?;
        Ok((input, (pattern, insert)))
    }

    pub(super) fn parse_input(input: &str) -> Result<Inputs, Box<dyn std::error::Error + '_>> {
        let (input, template) = template(input).map_err(Box::new)?;
        let (input, _) = tag::<_, _, ()>("\n")(input).map_err(Box::new)?;
        let (_, rules) = many1(rule)(input).map_err(Box::new)?;
        Ok((template, rules.into_iter().collect()))
    }
}
