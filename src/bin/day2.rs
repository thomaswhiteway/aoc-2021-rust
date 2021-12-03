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
struct Position {
    x: isize,
    y: isize,
    aim: isize,
}

#[derive(Debug)]
enum Command {
    Forward(isize),
    Down(isize),
    Up(isize),
}

fn parse_arg(value: &str) -> Result<isize, String> {
    value.parse::<isize>().map_err(|e| e.to_string())
}

impl TryFrom<String> for Command {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let parts: Vec<_> = value.split(' ').collect();
        if parts.len() != 2 {
            return Err(format!("Invalid command {}", value));
        }

        use Command::*;
        match parts[0] {
            "forward" => Ok(Forward(parse_arg(parts[1])?)),
            "down" => Ok(Down(parse_arg(parts[1])?)),
            "up" => Ok(Up(parse_arg(parts[1])?)),
            _ => Err(format!("Unknown command {}", parts[0])),
        }
    }
}

fn read_commands<P: AsRef<Path>>(input: P) -> Box<[Command]> {
    BufReader::new(File::open(input).unwrap())
        .lines()
        .map(Result::unwrap)
        .map(Command::try_from)
        .map(Result::unwrap)
        .collect::<Vec<_>>()
        .into_boxed_slice()
}

fn execute_command(command: &Command, position: &mut Position) {
    use Command::*;
    match command {
        Forward(x) => {
            position.x += x;
            position.y += x * position.aim;
        }
        Down(x) => position.aim += x,
        Up(x) => position.aim -= x,
    }
}

fn execute_commands(commands: &[Command]) -> Position {
    let mut position = Position { x: 0, y: 0, aim: 0 };

    for command in commands {
        execute_command(command, &mut position);
    }

    position
}

fn main() {
    let opt = Opt::from_args();

    let commands = read_commands(&opt.input);
    let end_pos = execute_commands(&commands);
    println!("{}", end_pos.x * end_pos.y);
}
