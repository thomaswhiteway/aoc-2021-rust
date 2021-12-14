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

fn apply_rules(rules: &Rules, current: HashMap<(char, char), usize>) -> HashMap<(char, char), usize> {
    let mut new_counts = HashMap::new();

    for ((a, b), num) in current {
        if let Some(&c) = rules.get(&(a, b)) {
            *new_counts.entry((a, c)).or_default() += num;
            *new_counts.entry((c, b)).or_default() += num;
        } else {
            *new_counts.entry((a, b)).or_default() += num;
        }
    }

    new_counts
}

fn count<V: Eq + Clone + Hash, I: IntoIterator<Item = V>>(sequence: I) -> HashMap<V, usize> {
    let mut counts = HashMap::new();

    for v in sequence {
        *counts.entry(v.clone()).or_default() += 1;
    }

    counts
}

fn count_chars_in_pairs(pair_counts: &HashMap<(char, char), usize>) -> HashMap<char, usize> {
    let mut counts = HashMap::new();

    for ((a, b), num) in pair_counts {
        *counts.entry(*a).or_default() += num;
        *counts.entry(*b).or_default() += num;
    }

    counts
}

fn display_offset(steps: usize, template: &[char], pair_counts: &HashMap<(char, char), usize>) {
    let mut char_counts = count_chars_in_pairs(&pair_counts);
    // All chars except for the first and last in the sequence appear twice.
    *char_counts.entry(template[0]).or_default() += 1;
    *char_counts.entry(template[template.len()-1]).or_default() += 1;
    char_counts = char_counts.into_iter().map(|(c, count)| (c, count / 2)).collect();

    let max = char_counts.values().max().unwrap();
    let min = char_counts.values().min().unwrap();

    println!("After {} steps: {}", steps, max - min);
}

fn main() {
    let opt = Opt::from_args();

    let (template, rules) = parse_input(opt.input);

    let mut pair_counts = count(template.iter().cloned().tuple_windows::<(_, _)>());

    for _ in 0..10 {
        pair_counts = apply_rules(&rules, pair_counts);
    }

    display_offset(10, &template, &pair_counts);

    for _ in 0..30 {
        pair_counts = apply_rules(&rules, pair_counts);
    }

    display_offset(40, &template, &pair_counts);

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
