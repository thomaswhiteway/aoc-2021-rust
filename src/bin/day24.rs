#![allow(dead_code)]
use aoc2021::tracker::{OperationTrack, Track};
use itertools::Itertools;
use std::collections::{HashMap, HashSet};
use std::fmt::{Display, Write};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(parse(from_os_str))]
    input: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Variable {
    W,
    X,
    Y,
    Z,
}

impl Variable {
    fn all() -> impl Iterator<Item = Variable> {
        use Variable::*;
        [W, X, Y, Z].into_iter()
    }
}

impl FromStr for Variable {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Variable::*;
        match s {
            "w" => Ok(W),
            "x" => Ok(X),
            "y" => Ok(Y),
            "z" => Ok(Z),
            _ => Err(format!("Invalid variable {}", s)),
        }
    }
}

impl Display for Variable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Variable::*;
        match self {
            W => write!(f, "W"),
            X => write!(f, "X"),
            Y => write!(f, "Y"),
            Z => write!(f, "Z"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Value {
    Variable(Variable),
    Literal(i64),
    Argument(usize),
}

impl Value {
    fn resolve(self, state: &State, arguments: &[i64]) -> i64 {
        use Value::*;
        match self {
            Variable(variable) => state.get(variable),
            Literal(value) => value,
            Argument(index) => arguments[index],
        }
    }

    fn extract_argument(&mut self, index: usize) -> Option<i64> {
        use Value::*;
        let value = if let Literal(value) = self {
            Some(*value)
        } else {
            None
        };

        if value.is_some() {
            *self = Argument(index);
        }

        value
    }

    fn remove_argument(&mut self, index: usize, value: i64) {
        use Value::*;
        if *self == Argument(index) {
            *self = Literal(value);
        } else if let Argument(ix) = self {
            if *ix > index {
                *ix -= 1
            }
        }
    }
}

impl FromStr for Value {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<i64>()
            .map(Value::Literal)
            .or_else(|_| s.parse::<Variable>().map(Value::Variable))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Instruction {
    Input(Variable),
    Add(Variable, Value),
    Mul(Variable, Value),
    Div(Variable, Value),
    Mod(Variable, Value),
    Eql(Variable, Value),
}

impl From<Variable> for Expression {
    fn from(v: Variable) -> Self {
        Expression::Variable(v)
    }
}

impl From<Value> for Expression {
    fn from(v: Value) -> Self {
        match v {
            Value::Literal(value) => Expression::Constant(value),
            Value::Variable(var) => Expression::Variable(var),
            Value::Argument(index) => Expression::Argument(index),
        }
    }
}

fn build_binary_expression<F, V1, V2>(cons: F, x: V1, y: V2) -> Expression
where
    F: Fn(Box<Expression>, Box<Expression>) -> Expression,
    V1: Into<Expression>,
    V2: Into<Expression>,
{
    cons(Box::new(x.into()), Box::new(y.into()))
}

impl Instruction {
    fn execute<I: Iterator<Item = i64>>(
        &self,
        state: &mut State,
        mut inputs: I,
        arguments: &[i64],
    ) {
        use Instruction::*;
        match *self {
            Input(out) => state.set(out, inputs.next().unwrap()),
            Add(x, y) => state.set(x, state.get(x) + y.resolve(state, arguments)),
            Mul(x, y) => state.set(x, state.get(x) * y.resolve(state, arguments)),
            Div(x, y) => state.set(x, state.get(x) / y.resolve(state, arguments)),
            Mod(x, y) => state.set(x, state.get(x) % y.resolve(state, arguments)),
            Eql(x, y) => state.set(
                x,
                if state.get(x) == y.resolve(state, arguments) {
                    1
                } else {
                    0
                },
            ),
        }
    }

    fn update<I: Iterator<Item = usize>>(&self, expression: &mut Expression, mut inputs: I) {
        let (var, new_expression) = match *self {
            Instruction::Input(out) => (out, Expression::Input(inputs.next().unwrap())),
            Instruction::Add(x, y) => (x, build_binary_expression(Expression::Add, x, y)),
            Instruction::Mul(x, y) => (x, build_binary_expression(Expression::Mul, x, y)),
            Instruction::Div(x, y) => (x, build_binary_expression(Expression::Div, x, y)),
            Instruction::Mod(x, y) => (x, build_binary_expression(Expression::Mod, x, y)),
            Instruction::Eql(x, y) => (x, build_binary_expression(Expression::Eql, x, y)),
        };
        expression.update_var(var, &new_expression)
    }

    fn extract_argument(&mut self, index: usize) -> Option<i64> {
        use Instruction::*;
        match self {
            Input(_) => None,
            Add(_, y) | Mul(_, y) | Div(_, y) | Mod(_, y) | Eql(_, y) => y.extract_argument(index),
        }
    }

    fn remove_argument(&mut self, index: usize, value: i64) {
        use Instruction::*;
        match self {
            Input(_) => {}
            Add(_, y) | Mul(_, y) | Div(_, y) | Mod(_, y) | Eql(_, y) => {
                y.remove_argument(index, value)
            }
        }
    }
}

fn read_unary_instruction<'a, F, I, V>(cons: F, iter: &mut I) -> Result<Instruction, String>
where
    F: Fn(V) -> Instruction,
    I: Iterator<Item = &'a str>,
    V: FromStr<Err = String>,
{
    Ok(cons(read_param(iter)?))
}

fn read_binary_instruction<'a, F, I, V1, V2>(cons: F, iter: &mut I) -> Result<Instruction, String>
where
    F: Fn(V1, V2) -> Instruction,
    I: Iterator<Item = &'a str>,
    V1: FromStr<Err = String>,
    V2: FromStr<Err = String>,
{
    Ok(cons(read_param(iter)?, read_param(iter)?))
}

fn read_param<'a, I, V>(iter: &mut I) -> Result<V, String>
where
    I: Iterator<Item = &'a str>,
    V: FromStr<Err = String>,
{
    iter.next()
        .ok_or_else(|| "Missing parameter".to_string())?
        .parse()
}

impl FromStr for Instruction {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Instruction::*;
        let mut parts = s.split(' ');
        match parts
            .next()
            .ok_or_else(|| "Empty instruction".to_string())?
        {
            "inp" => read_unary_instruction(Input, &mut parts),
            "add" => read_binary_instruction(Add, &mut parts),
            "mul" => read_binary_instruction(Mul, &mut parts),
            "div" => read_binary_instruction(Div, &mut parts),
            "mod" => read_binary_instruction(Mod, &mut parts),
            "eql" => read_binary_instruction(Eql, &mut parts),
            instruction => Err(format!("Unknown instruction {}", instruction)),
        }
    }
}

fn read_instructions<P: AsRef<Path>>(input: P) -> Box<[Instruction]> {
    BufReader::new(File::open(input).unwrap())
        .lines()
        .map(Result::unwrap)
        .map(|line| line.parse().unwrap())
        .collect::<Vec<_>>()
        .into_boxed_slice()
}

struct State {
    variables: [i64; 4],
}

impl State {
    fn new() -> Self {
        State { variables: [0; 4] }
    }

    fn set(&mut self, variable: Variable, value: i64) {
        self.variables[variable as usize] = value
    }

    fn get(&self, variable: Variable) -> i64 {
        self.variables[variable as usize]
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
enum Expression {
    Argument(usize),
    Variable(Variable),
    Constant(i64),
    Input(usize),
    Add(Box<Expression>, Box<Expression>),
    Mul(Box<Expression>, Box<Expression>),
    Div(Box<Expression>, Box<Expression>),
    Mod(Box<Expression>, Box<Expression>),
    Eql(Box<Expression>, Box<Expression>),
}

impl Expression {
    fn update_var(&mut self, variable: Variable, expression: &Expression) {
        use Expression::*;
        match self {
            Variable(v) if *v == variable => *self = expression.clone(),
            Add(x, y) | Mul(x, y) | Div(x, y) | Mod(x, y) | Eql(x, y) => {
                x.update_var(variable, expression);
                y.update_var(variable, expression);
            }
            _ => {}
        }
    }

    fn normalize(&mut self) {
        use Expression::*;
        match self {
            Add(x, y) | Mul(x, y) | Div(x, y) | Mod(x, y) | Eql(x, y) => {
                x.normalize();
                y.normalize();
            }
            _ => {}
        }

        match self {
            Add(x, y) => {
                if **x == Constant(0) {
                    *self = *y.clone();
                } else if **y == Constant(0) {
                    *self = *x.clone();
                }
            }
            Mul(x, y) => {
                if **x == Constant(1) {
                    *self = *y.clone();
                } else if **y == Constant(1) {
                    *self = *x.clone();
                } else if **x == Constant(0) || **y == Constant(0) {
                    *self = Constant(0);
                }
            }
            Div(x, y) => {
                if **y == Constant(1) {
                    *self = *x.clone();
                }
            }
            _ => {}
        }
    }

    fn size(&self) -> usize {
        use Expression::*;
        match self {
            Variable(_) | Constant(_) | Input(_) | Argument(_) => 1,
            Add(x, y) | Mul(x, y) | Div(x, y) | Mod(x, y) | Eql(x, y) => 1 + x.size() + y.size(),
        }
    }

    fn expand(&mut self, instructions: &[Instruction]) {
        let mut inputs = 0..;
        for instruction in instructions.iter().rev() {
            instruction.update(self, &mut inputs);
        }
    }

    fn is_compound(&self) -> bool {
        use Expression::*;
        matches!(self, Add(..) | Mul(..) | Div(..) | Mod(..) | Eql(..))
    }
}

fn write_binary_op<W: Write>(
    mut w: W,
    op: &'static str,
    x: &Expression,
    y: &Expression,
) -> std::fmt::Result {
    if !x.is_compound() {
        write!(w, "{}", x)?;
    } else {
        write!(w, "({})", x)?;
    }
    write!(w, " {} ", op)?;
    if !y.is_compound() {
        write!(w, "{}", y)
    } else {
        write!(w, "({})", y)
    }
}

impl Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Expression::*;
        match self {
            Variable(v) => write!(f, "{}", v),
            Constant(c) => write!(f, "{}", c),
            Argument(index) => write!(f, "args[{}]", index),
            Input(index) => write!(f, "input[{}]", index),
            Add(x, y) => write_binary_op(f, "+", x, y),
            Mul(x, y) => write_binary_op(f, "*", x, y),
            Div(x, y) => write_binary_op(f, "/", x, y),
            Mod(x, y) => write_binary_op(f, "%", x, y),
            Eql(x, y) => write_binary_op(f, "==", x, y),
        }
    }
}

fn output_for_digit(z: i64, digit: i64, a: i64, b: i64, c: i64) -> i64 {
    (if (z % 26) + b != digit {
        (z / a) * 25 + (digit + c)
    } else {
        0
    }) + (z / a)
}

fn run(instructions: &[Instruction], input: &[i64], arguments: &[i64], z: i64) -> i64 {
    let mut state = State::new();
    state.set(Variable::Z, z);
    let mut inputs = input.iter().cloned();

    for instruction in instructions.iter() {
        instruction.execute(&mut state, &mut inputs, arguments);
    }

    state.get(Variable::Z)
}

struct ModelNumberChecker<T> {
    instructions: Box<[Instruction]>,
    tracker: T,
}

impl<T: Track> ModelNumberChecker<T> {
    fn new(instructions: Box<[Instruction]>, tracker: T) -> Self {
        ModelNumberChecker {
            instructions,
            tracker,
        }
    }

    fn run(&self, input: &[i64]) -> i64 {
        run(&self.instructions, input, &[], 0)
    }

    fn is_valid_model_number(&self, model_number: i64) -> bool {
        let op = self.tracker.track_operation();

        let digits = {
            let _t = op.track_duration("format");
            model_number
                .to_string()
                .chars()
                .map(|c| c.to_digit(10).unwrap() as i64)
                .collect::<Vec<_>>()
        };

        let mut allowed = {
            let _t = op.track_duration("check");
            digits.iter().all(|d| *d != 0)
        };

        if allowed {
            let _t = op.track_duration("run");
            allowed = self.run(&digits) == 0
        }

        allowed
    }
}

fn extract_arguments(function: &mut [Instruction]) -> Vec<i64> {
    let mut args = vec![];

    for instruction in function {
        if let Some(arg) = instruction.extract_argument(args.len()) {
            args.push(arg);
        }
    }

    args
}

fn print_function_output(variable: Variable, function: &[Instruction]) {
    let mut exp = Expression::Variable(variable);
    exp.expand(function);
    exp.normalize();
    println!("{} = {}", variable, exp);
}

fn resolve_common_args(instructions: &mut [Instruction], arguments: &mut [Vec<i64>]) {
    let mut index = 0;
    while index < arguments[0].len() {
        if arguments
            .iter()
            .tuple_windows()
            .all(|(args1, args2)| args1[index] == args2[index])
        {
            for instruction in instructions.iter_mut() {
                instruction.remove_argument(index, arguments[0][index]);
            }

            for args in arguments.iter_mut() {
                args.remove(index);
            }
        } else {
            index += 1;
        }
    }
}

#[allow(clippy::type_complexity)]
fn extract_function(
    instructions: &[Instruction],
    length: usize,
) -> (Box<[Instruction]>, Box<[Box<[i64]>]>) {
    let mut function: Option<Box<[Instruction]>> = None;
    let mut arguments = vec![];

    for index in (0..instructions.len()).step_by(length) {
        let mut func = instructions[index..index + length]
            .to_vec()
            .into_boxed_slice();
        let args = extract_arguments(&mut func);
        arguments.push(args);

        if let Some(function) = &function {
            assert_eq!(*function, func);
        } else {
            function = Some(func);
        }
    }

    let mut function = function.unwrap();

    resolve_common_args(&mut function, &mut arguments);

    (
        function,
        arguments
            .into_iter()
            .map(Vec::into_boxed_slice)
            .collect::<Vec<_>>()
            .into_boxed_slice(),
    )
}

fn main() {
    let opt = Opt::from_args();
    let instructions = read_instructions(opt.input);

    let (function, arguments) = extract_function(&instructions, 18);

    for a in [1, 26] {
        for b in -16..=13 {
            for c in 2..=15 {
                for digit in 1..10 {
                    for z in 0..26 {
                        assert_eq!(
                            output_for_digit(z, digit, a, b, c),
                            run(&function, &[digit], &[a, b, c], z)
                        );
                    }
                }
            }
        }
    }

    for instruction in function.iter() {
        println!("{:?}", instruction);
    }
    println!();

    for variable in Variable::all() {
        print_function_output(variable, &function);
    }

    println!();
    println!("Arguments:");
    for args in arguments.iter() {
        println!("{:?}", args);
    }

    println!();

    println!("Calculating possible zs");
    let mut zs = vec![[0_i64].into_iter().collect::<HashSet<_>>()];

    for (index, args) in arguments[..arguments.len() - 1].iter().enumerate() {
        let last_zs = zs.last().unwrap();
        let new_zs: HashSet<i64> = last_zs
            .iter()
            .flat_map(|z| {
                (1..10).map(|digit|
            //output_for_digit(*z, digit, args[0], args[1], args[2])
            run(&function, &[digit], args, *z))
            })
            .collect();
        println!("{}: {}", index, new_zs.len());
        zs.push(new_zs);
    }

    println!("Calculating potential valid nums");
    let mut candidates: HashMap<i64, Vec<Vec<i64>>> = [(0, vec![vec![]])].into_iter().collect();
    for (index, args) in arguments.iter().enumerate().rev() {
        let mut new_candidates: HashMap<i64, Vec<Vec<i64>>> = HashMap::new();

        for z_in in zs[index].iter() {
            for digit in 1..10 {
                let z_out = run(&function, &[digit], args, *z_in);
                if let Some(seqs) = candidates.get(&z_out) {
                    for seq in seqs {
                        let mut seq = seq.clone();
                        seq.push(digit);
                        new_candidates.entry(*z_in).or_default().push(seq);
                    }
                }
            }
        }

        candidates = new_candidates;
        println!("{}: {}", index, candidates.len());
    }

    let mut nums = candidates
        .get(&0)
        .unwrap()
        .iter()
        .map(|num| {
            num.iter()
                .rev()
                .map(|d| char::from_digit(*d as u32, 10).unwrap())
                .collect::<String>()
        })
        .collect::<Vec<_>>();
    println!("Have {} valid membership numbers", nums.len());
    nums.sort();
    println!("Highest: {}", nums.last().unwrap());
    println!("Lowest: {}", nums.first().unwrap());
}

#[cfg(test)]
mod test {
    use super::*;

    fn op<F>(op: F, x: Expression, y: Expression) -> Expression
    where
        F: Fn(Box<Expression>, Box<Expression>) -> Expression,
    {
        op(Box::new(x), Box::new(y))
    }

    #[test]
    fn test_normalize() {
        use self::Variable::*;
        use Expression::*;
        let mut exp = op(Mul, Variable(X), Constant(0));
        exp.normalize();
        assert_eq!(exp, Constant(0));
    }

    #[test]
    fn test_normalize_large() {
        use self::Variable::*;
        use Expression::*;
        let mut exp = op(
            Eql,
            op(
                Eql,
                op(
                    Add,
                    op(
                        Mod,
                        op(Add, op(Mul, Variable(X), Constant(0)), Variable(Z)),
                        Constant(26),
                    ),
                    Argument(1),
                ),
                Input(0),
            ),
            Constant(0),
        );
        exp.normalize();
        assert_eq!(
            exp,
            op(
                Eql,
                op(
                    Eql,
                    op(Add, op(Mod, Variable(Z), Constant(26)), Argument(1)),
                    Input(0)
                ),
                Constant(0)
            )
        );
    }
}
