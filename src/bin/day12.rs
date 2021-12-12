use std::collections::HashMap;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(parse(from_os_str))]
    input: PathBuf,
}

type Tunnels = HashMap<String, Vec<String>>;

struct Tunnel {
    start: String,
    end: String,
}

impl FromStr for Tunnel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split('-');

        let start = parts.next().unwrap().to_string();
        let end = parts
            .next()
            .ok_or(format!("Invalid tunnel {:?}", s))?
            .to_string();

        if parts.next() != None {
            return Err(format!("Invalid tunnel {:?}", s));
        }

        Ok(Tunnel { start, end })
    }
}

fn parse_tunnels<P: AsRef<Path>>(input: P) -> Tunnels {
    let mut tunnels: Tunnels = HashMap::new();

    let file = File::open(input).unwrap();

    for line in BufReader::new(file).lines() {
        let Tunnel { start, end } = line.unwrap().parse::<Tunnel>().unwrap();

        tunnels.entry(start.clone()).or_default().push(end.clone());
        tunnels.entry(end).or_default().push(start);
    }

    tunnels
}

fn is_large_cave(name: &str) -> bool {
    name.chars().all(|c| c.is_uppercase())
}

fn find_num_routes<F, S>(
    tunnels: &Tunnels,
    start: &str,
    end: &str,
    initial_state: S,
    can_visit: F,
) -> usize
where
    F: Fn(&[&str], &str, &S) -> Option<S>,
    S: Clone,
{
    let mut stack = vec![(vec![start], initial_state)];
    let mut num_routes = 0;

    while let Some((route, state)) = stack.pop() {
        let last = *route.last().unwrap();
        if last == end {
            num_routes += 1;
        } else {
            for next in tunnels.get(last).unwrap() {
                if let Some(new_state) = can_visit(&route, next.as_str(), &state) {
                    let mut new_route = route.clone();
                    new_route.push(next);
                    stack.push((new_route, new_state));
                }
            }
        }
    }

    num_routes
}

fn main() {
    let opt = Opt::from_args();

    let tunnels = parse_tunnels(opt.input);
    let num_simple_routes = find_num_routes(&tunnels, "start", "end", (), |route, next, _| {
        if is_large_cave(next) || !route.contains(&next) {
            Some(())
        } else {
            None
        }
    });
    println!("{}", num_simple_routes);

    let num_complex_routes = find_num_routes(
        &tunnels,
        "start",
        "end",
        true,
        |route, next, &can_visit_small_cave_twice| {
            if is_large_cave(next) || !route.contains(&next) {
                Some(can_visit_small_cave_twice)
            } else if can_visit_small_cave_twice && next != "start" {
                Some(false)
            } else {
                None
            }
        },
    );
    println!("{}", num_complex_routes);
}
