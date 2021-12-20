use itertools::Itertools;
use std::fmt::Debug;
use std::fmt::Write;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(parse(from_os_str))]
    input: PathBuf,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Token {
    Open,
    Close,
    Literal(u32),
    Comma,
}

impl Token {
    fn value(&self) -> Option<u32> {
        match self {
            Token::Literal(x) => Some(*x),
            _ => None,
        }
    }
}

fn parse_number(line: &str) -> Vec<Token> {
    line.chars()
        .map(|c| match c {
            '[' => Token::Open,
            ']' => Token::Close,
            ',' => Token::Comma,
            x => Token::Literal(x.to_digit(10).unwrap()),
        })
        .collect()
}

fn number_to_string(number: &[Token]) -> String {
    let mut result = String::new();

    for tok in number {
        match tok {
            Token::Open => write!(result, "[").unwrap(),
            Token::Close => write!(result, "]").unwrap(),
            Token::Comma => write!(result, ",").unwrap(),
            Token::Literal(val) => write!(result, "{}", val).unwrap(),
        }
    }

    result
}

fn parse_numbers<P: AsRef<Path>>(input: P) -> Vec<Vec<Token>> {
    let file = File::open(&input).unwrap();
    BufReader::new(file)
        .lines()
        .map(Result::unwrap)
        .map(|line| parse_number(&line))
        .collect()
}

fn index_to_explode(number: &[Token]) -> Option<usize> {
    let mut depth = 0;

    for (index, token) in number.iter().enumerate() {
        match token {
            Token::Open => {
                depth += 1;
                if depth >= 5 {
                    return Some(index);
                }
            }
            Token::Close => {
                depth -= 1;
            }
            _ => {}
        }
    }

    None
}

fn explode(number: &mut Vec<Token>, index: usize) {
    let left = number[index + 1].value().unwrap();
    let right = number[index + 3].value().unwrap();

    for ix in (0..index).rev() {
        if let Token::Literal(val) = &mut number[ix] {
            *val += left;
            break;
        }
    }

    for token in number.iter_mut().skip(index + 5) {
        if let Token::Literal(val) = token {
            (*val) += right;
            break;
        }
    }

    number.splice(index..index + 5, [Token::Literal(0)]);
}

fn index_to_split(number: &[Token]) -> Option<usize> {
    for (index, token) in number.iter().enumerate() {
        if let Token::Literal(val) = token {
            if *val >= 10 {
                return Some(index);
            }
        }
    }

    None
}

fn split(number: &mut Vec<Token>, index: usize) {
    let num = number[index].value().unwrap();
    let left = num / 2;
    let right = left + num % 2;
    number.splice(
        index..index + 1,
        [
            Token::Open,
            Token::Literal(left),
            Token::Comma,
            Token::Literal(right),
            Token::Close,
        ],
    );
}

fn add(total: &mut Vec<Token>, num: &[Token]) {
    total.splice(0..0, [Token::Open]);
    total.push(Token::Comma);
    total.extend(num);
    total.push(Token::Close);

    reduce(total);
}

fn reduce(number: &mut Vec<Token>) {
    loop {
        if let Some(index) = index_to_explode(number) {
            explode(number, index);
        } else if let Some(index) = index_to_split(number) {
            split(number, index);
        } else {
            break;
        }
    }
}

fn get_magnitude(number: &[Token]) -> u32 {
    let mut total = 0;
    let mut mult = 1;

    for token in number {
        match token {
            Token::Open => mult *= 3,
            Token::Close => mult /= 2,
            Token::Comma => mult = (mult / 3) * 2,
            Token::Literal(val) => total += val * mult,
        }
    }

    total
}

fn main() {
    let opt = Opt::from_args();

    let numbers = parse_numbers(opt.input);

    let mut total = numbers[0].clone();
    for num in &numbers[1..] {
        add(&mut total, num);
    }

    println!("{}", number_to_string(&total));

    let magnitude = get_magnitude(&total);
    println!("{}", magnitude);

    let max_mag = numbers
        .iter()
        .cartesian_product(numbers.iter())
        .filter(|(x, y)| x != y)
        .map(|(x, y)| {
            let mut total = x.clone();
            add(&mut total, y);
            get_magnitude(&total)
        })
        .max()
        .unwrap();
    println!("{}", max_mag);
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_add_reduce() {
        let mut number: Vec<Token> = parse_number("[[[[[4,3],4],4],[7,[[8,4],9]]],[1,1]]");

        let index = index_to_explode(&number).unwrap();
        explode(&mut number, index);
        assert_eq!(
            &number_to_string(&number),
            "[[[[0,7],4],[7,[[8,4],9]]],[1,1]]"
        );

        let index = index_to_explode(&number).unwrap();
        explode(&mut number, index);
        assert_eq!(
            &number_to_string(&number),
            "[[[[0,7],4],[15,[0,13]]],[1,1]]"
        );
    }
}
