use std::fmt::{Debug, Display};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::iter::Sum;
use std::ops::Add;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(parse(from_os_str))]
    input: PathBuf,
}

#[derive(Clone, PartialEq, Eq)]
enum Value {
    Literal(u64),
    Number(Number),
}

impl Value {
    fn is_literal(&self) -> bool {
        matches!(self, Value::Literal(_))
    }

    fn value_at(&self, path: &[Direction]) -> Option<&Value> {
        if let Some(dir) = path.first() {
            if let Value::Number(num) = self {
                let next: &Value = match dir {
                    Direction::Left => &num.left,
                    Direction::Right => &num.right,
                };

                next.value_at(&path[1..])
            } else {
                None
            }
        } else {
            Some(self)
        }
    }

    fn value_at_mut(&mut self, path: &[Direction]) -> Option<&mut Value> {
        if let Some(dir) = path.first() {
            if let Value::Number(num) = self {
                let next: &mut Value = match dir {
                    Direction::Left => &mut num.left,
                    Direction::Right => &mut num.right,
                };

                next.value_at_mut(&path[1..])
            } else {
                None
            }
        } else {
            Some(self)
        }
    }

    fn first_literal_path(&self) -> Box<[Direction]> {
        let mut value = self;
        let mut result = vec![];

        while let Value::Number(num) = value {
            result.push(Direction::Left);
            value = &num.left;
        }

        result.into_boxed_slice()
    }

    fn last_literal_path(&self) -> Box<[Direction]> {
        let mut value = self;
        let mut result = vec![];

        while let Value::Number(num) = value {
            result.push(Direction::Right);
            value = &num.right;
        }

        result.into_boxed_slice()
    }

    fn literal_paths(&self) -> impl Iterator<Item = Box<[Direction]>> + '_ {
        LiteralPaths::new(self)
    }

    fn first_number_path(&self) -> Option<Box<[Direction]>> {
        let mut value = self;

        if value.is_literal() {
            return None;
        }

        let mut result = vec![];

        while let Value::Number(num) = value {
            if num.left.is_literal() {
                break;
            }

            result.push(Direction::Left);
            value = &num.left;
        }

        Some(result.into_boxed_slice())
    }

    fn next_number_path(&self, path: &[Direction]) -> Option<Box<[Direction]>> {
        let mut next_path = path.to_vec();

        let number = self.value_at(path).unwrap().as_number().unwrap();
        if let Some(path) = number.right.first_number_path() {
            next_path.push(Direction::Right);
            next_path.extend(path.iter());
        } else {
            loop {
                match next_path.pop() {
                    None => return None,
                    Some(Direction::Left) => {
                        break;
                    }
                    Some(Direction::Right) => {}
                }
            }
        }

        Some(next_path.into_boxed_slice())
    }

    fn number_paths(&self) -> impl Iterator<Item = Box<[Direction]>> + '_ {
        NumberPaths::new(self)
    }

    fn should_split(&self, path: &[Direction]) -> bool {
        if let Value::Literal(val) = self.value_at(path).unwrap() {
            *val >= 10
        } else {
            false
        }
    }

    fn into_number(self) -> Option<Number> {
        match self {
            Value::Literal(_) => None,
            Value::Number(number) => Some(number),
        }
    }

    fn as_number(&self) -> Option<&Number> {
        match self {
            Value::Literal(_) => None,
            Value::Number(number) => Some(number),
        }
    }

    fn as_literal(&self) -> Option<&u64> {
        match self {
            Value::Literal(val) => Some(val),
            Value::Number(_) => None,
        }
    }

    fn as_literal_mut_ref(&mut self) -> Option<&mut u64> {
        match self {
            Value::Literal(val) => Some(val),
            Value::Number(_) => None,
        }
    }

    fn prev_literal_path(&self, path: &[Direction]) -> Option<Box<[Direction]>> {
        let mut prev_path = path.to_vec();

        loop {
            match prev_path.pop() {
                None => return None,
                Some(Direction::Right) => {
                    prev_path.push(Direction::Left);
                    prev_path.extend(
                        self.value_at(&prev_path)
                            .unwrap()
                            .last_literal_path()
                            .iter(),
                    );
                    break;
                }
                Some(Direction::Left) => {}
            }
        }

        Some(prev_path.into_boxed_slice())
    }

    fn next_literal_path(&self, path: &[Direction]) -> Option<Box<[Direction]>> {
        let mut next_path = path.to_vec();

        loop {
            match next_path.pop() {
                None => return None,
                Some(Direction::Left) => {
                    next_path.push(Direction::Right);
                    next_path.extend(
                        self.value_at(&next_path)
                            .unwrap()
                            .first_literal_path()
                            .iter(),
                    );
                    break;
                }
                Some(Direction::Right) => {}
            }
        }

        Some(next_path.into_boxed_slice())
    }

    fn prev_literal_mut(&mut self, path: &[Direction]) -> Option<&mut u64> {
        self.prev_literal_path(path)
            .and_then(|path| self.value_at_mut(&path))
            .and_then(|val| val.as_literal_mut_ref())
    }

    fn next_literal_mut(&mut self, path: &[Direction]) -> Option<&mut u64> {
        self.next_literal_path(path)
            .and_then(|path| self.value_at_mut(&path))
            .and_then(|val| val.as_literal_mut_ref())
    }

    fn path_to_explode(&self) -> Option<Box<[Direction]>> {
        self.number_paths().find(|path| path.len() >= 4)
    }

    fn explode(&mut self, path: &[Direction]) {
        let Number { left, right } = self.value_at(path).unwrap().as_number().unwrap();

        let left = *left.as_literal().unwrap();
        let right = *right.as_literal().unwrap();

        let left_path = path
            .iter()
            .chain(&[Direction::Left])
            .cloned()
            .collect::<Vec<_>>();
        if let Some(val) = self.prev_literal_mut(&left_path) {
            *val += left;
        }

        let right_path = path
            .iter()
            .chain(&[Direction::Right])
            .cloned()
            .collect::<Vec<_>>();
        if let Some(val) = self.next_literal_mut(&right_path) {
            *val += right;
        }

        *self.value_at_mut(path).unwrap() = Value::Literal(0);
    }

    fn path_to_split(&self) -> Option<Box<[Direction]>> {
        self.literal_paths().find(|path| self.should_split(path))
    }

    fn split(&mut self, path: &[Direction]) {
        let val = self.value_at_mut(path).unwrap();
        let num = *val.as_literal().unwrap();
        let left = num / 2;
        let right = left + num % 2;

        *val = Value::Number(Number {
            left: Box::new(Value::Literal(left)),
            right: Box::new(Value::Literal(right)),
        });
    }

    fn magnitude(&self) -> u64 {
        match self {
            Value::Literal(val) => *val,
            Value::Number(number) => number.magnitude(),
        }
    }
}

impl FromStr for Value {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parsing::value(s)
            .map(|(_, num)| num)
            .map_err(|err| err.to_string())
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Literal(val) => Display::fmt(val, f),
            Value::Number(val) => Display::fmt(val, f),
        }
    }
}

impl Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Direction {
    Left,
    Right,
}

struct LiteralPaths<'a> {
    value: &'a Value,
    prev_path: Option<Box<[Direction]>>,
}

impl<'a, 'b> LiteralPaths<'a> {
    fn new(value: &'a Value) -> Self {
        LiteralPaths {
            value,
            prev_path: None,
        }
    }
}

impl<'a> Iterator for LiteralPaths<'a> {
    type Item = Box<[Direction]>;

    fn next(&mut self) -> Option<Self::Item> {
        self.prev_path = if let Some(path) = &self.prev_path {
            self.value.next_literal_path(path)
        } else {
            Some(self.value.first_literal_path())
        };

        self.prev_path.clone()
    }
}

struct NumberPaths<'a> {
    value: &'a Value,
    prev_path: Option<Box<[Direction]>>,
}

impl<'a> NumberPaths<'a> {
    fn new(value: &'a Value) -> Self {
        NumberPaths {
            value,
            prev_path: None,
        }
    }
}

impl<'a> Iterator for NumberPaths<'a> {
    type Item = Box<[Direction]>;

    fn next(&mut self) -> Option<Self::Item> {
        self.prev_path = if let Some(path) = &self.prev_path {
            self.value.next_number_path(path)
        } else {
            self.value.first_number_path()
        };

        self.prev_path.clone()
    }
}

#[derive(Clone, PartialEq, Eq)]
struct Number {
    left: Box<Value>,
    right: Box<Value>,
}

impl Number {
    fn reduce(self) -> Number {
        let mut output = Value::Number(self);

        loop {
            if let Some(to_explode) = output.path_to_explode() {
                output.explode(&to_explode);
            } else if let Some(to_split) = output.path_to_split() {
                output.split(&to_split);
            } else {
                break;
            }
        }

        output.into_number().unwrap()
    }

    fn magnitude(&self) -> u64 {
        3 * self.left.magnitude() + 2 * self.right.magnitude()
    }
}

impl Add for Number {
    type Output = Number;

    fn add(self, rhs: Self) -> Self::Output {
        Number {
            left: Box::new(Value::Number(self)),
            right: Box::new(Value::Number(rhs)),
        }
        .reduce()
    }
}

impl Sum<Number> for Number {
    fn sum<I>(iter: I) -> Self
    where
        I: Iterator<Item = Number>,
    {
        iter.reduce(Number::add).unwrap()
    }
}

impl FromStr for Number {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parsing::number(s)
            .map(|(_, num)| num)
            .map_err(|err| err.to_string())
    }
}

impl std::fmt::Display for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{},{}]", self.left, self.right)
    }
}

impl Debug for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

fn parse_numbers<P: AsRef<Path>>(input: P) -> impl Iterator<Item = Number> {
    BufReader::new(File::open(input).unwrap())
        .lines()
        .map(Result::unwrap)
        .map(|value| value.parse().unwrap())
}

fn main() {
    let opt = Opt::from_args();

    let numbers = parse_numbers(opt.input);
    let total = numbers.sum::<Number>();
    println!("{}", total);
    println!("{}", total.magnitude());
}

mod parsing {
    use crate::{Number, Value};

    use nom::branch::alt;
    use nom::bytes::complete::tag;
    use nom::character::complete::one_of;
    use nom::combinator::{map, map_res, recognize};
    use nom::multi::many1;
    use nom::sequence::{delimited, separated_pair};
    use nom::IResult;
    use std::str::FromStr;

    fn literal(input: &str) -> IResult<&str, u64> {
        map_res(recognize(many1(one_of("0123456789"))), u64::from_str)(input)
    }

    pub(super) fn value(input: &str) -> IResult<&str, Value> {
        alt((map(literal, Value::Literal), map(number, Value::Number)))(input)
    }

    pub(super) fn number(input: &str) -> IResult<&str, Number> {
        let (rest, (left, right)) =
            delimited(tag("["), separated_pair(value, tag(","), value), tag("]"))(input)?;
        Ok((
            rest,
            Number {
                left: Box::new(left),
                right: Box::new(right),
            },
        ))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_path_to_explode() {
        use Direction::*;

        let value: Value = "[[[[[9,8],1],2],3],4]".parse().unwrap();
        assert_eq!(
            value.path_to_explode(),
            Some([Left, Left, Left, Left].to_vec().into_boxed_slice())
        )
    }

    #[test]
    fn test_number_paths() {
        let value: Value = "[[[[0,7],4],[7,[[8,4],9]]],[1,1]]".parse().unwrap();

        assert_eq!(value.number_paths().count(), 8);
    }

    #[test]
    fn test_add_reduce() {
        let left: Number = "[[[[4,3],4],4],[7,[[8,4],9]]]".parse().unwrap();
        let right: Number = "[1,1]".parse().unwrap();

        let mut value = Value::Number(Number {
            left: Box::new(Value::Number(left)),
            right: Box::new(Value::Number(right)),
        });
        value.explode(&value.path_to_explode().unwrap());
        assert_eq!(&value.to_string(), "[[[[0,7],4],[7,[[8,4],9]]],[1,1]]");

        value.explode(&value.path_to_explode().unwrap());
        assert_eq!(&value.to_string(), "[[[[0,7],4],[15,[0,13]]],[1,1]]");
    }
}
