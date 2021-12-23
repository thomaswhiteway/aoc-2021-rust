use aoc2021::a_star;
use std::fmt::Display;
use std::fs::File;
use std::hash::Hash;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opt {
    #[structopt(parse(from_os_str))]
    input: PathBuf,
}

#[derive(Clone, Debug, Copy, PartialEq, Eq, Hash)]
enum Amphipod {
    Amber,
    Bronze,
    Copper,
    Desert,
}

impl Amphipod {
    fn room(&self) -> usize {
        use Amphipod::*;
        match self {
            Amber => 0,
            Bronze => 1,
            Copper => 2,
            Desert => 3,
        }
    }

    fn energy_to_move(&self) -> usize {
        use Amphipod::*;
        match self {
            Amber => 1,
            Bronze => 10,
            Copper => 100,
            Desert => 1000,
        }
    }
}

impl Display for Amphipod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Amphipod::*;
        match self {
            Amber => write!(f, "A"),
            Bronze => write!(f, "B"),
            Copper => write!(f, "C"),
            Desert => write!(f, "D"),
        }
    }
}

impl TryFrom<char> for Amphipod {
    type Error = char;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        use Amphipod::*;
        match value {
            'A' => Ok(Amber),
            'B' => Ok(Bronze),
            'C' => Ok(Copper),
            'D' => Ok(Desert),
            _ => Err(value),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Location {
    Room { room: usize, depth: usize },
    Corridor { spot: usize },
}

impl From<Location> for Position {
    fn from(location: Location) -> Self {
        use Location::*;

        match location {
            Room { room, depth } => Position {
                x: 2 + 2 * room,
                y: depth + 1,
            },
            Corridor { spot } => {
                if spot == 0 {
                    Position { x: 0, y: 0 }
                } else if spot < 6 {
                    Position {
                        x: 2 * spot - 1,
                        y: 0,
                    }
                } else {
                    Position { x: 10, y: 0 }
                }
            }
        }
    }
}

impl Location {
    fn distance_to(&self, other: Location) -> usize {
        Position::from(*self).distance_to(other.into())
    }

    fn room(&self) -> Option<usize> {
        use Location::*;
        match self {
            Room { room, .. } => Some(*room),
            Corridor { .. } => None,
        }
    }

    fn same_room(&self, other: Location) -> bool {
        self.room()
            .zip(other.room())
            .map(|(room, other_room)| room == other_room)
            .unwrap_or(false)
    }
}

impl TryFrom<Position> for Location {
    type Error = ();

    fn try_from(position: Position) -> Result<Self, Self::Error> {
        use Location::*;
        if position.y == 0 {
            if position.x == 0 {
                Ok(Corridor { spot: 0 })
            } else if position.x < 10 {
                if position.x % 2 == 1 {
                    Ok(Corridor {
                        spot: (position.x + 1) / 2,
                    })
                } else {
                    Err(())
                }
            } else {
                Ok(Corridor { spot: 6 })
            }
        } else {
            Ok(Room {
                room: position.x / 2 - 1,
                depth: position.y - 1,
            })
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Position {
    x: usize,
    y: usize,
}

impl Position {
    fn distance_to(&self, other: Position) -> usize {
        abs_diff(self.x, other.x) + self.y + other.y
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
struct Layout {
    room_depth: usize,
    corridor: [Option<Amphipod>; 7],
    rooms: [Vec<Amphipod>; 4],
}

fn abs_diff(x: usize, y: usize) -> usize {
    if x >= y {
        x - y
    } else {
        y - x
    }
}

impl Layout {
    fn read<P: AsRef<Path>>(input: P) -> Layout {
        let reader = BufReader::new(File::open(input).unwrap());
        let lines = reader.lines();

        let rows = lines
            .map(Result::unwrap)
            .skip(2)
            .take(2)
            .map(|line| Self::parse_row(&line))
            .collect::<Vec<_>>();

        let mut rooms: [Vec<Amphipod>; 4] = Default::default();

        for amphipods in rows.iter().rev() {
            for (&amphipod, room) in amphipods.iter().zip(rooms.iter_mut()) {
                room.push(amphipod)
            }
        }

        Layout {
            room_depth: 2,
            corridor: Default::default(),
            rooms,
        }
    }

    fn parse_row(line: &str) -> [Amphipod; 4] {
        line.chars()
            .filter_map(|c| c.try_into().ok())
            .collect::<Vec<_>>()
            .try_into()
            .unwrap()
    }

    fn insert_row(&mut self, index: usize, row: &[Amphipod; 4]) {
        for (amphipod, room) in row.iter().zip(self.rooms.iter_mut()) {
            room.insert(index, *amphipod);
        }
        self.room_depth += 1;
    }

    fn is_complete(&self) -> bool {
        self.rooms.iter().enumerate().all(|(room, amphipods)| {
            amphipods.len() == self.room_depth
                && amphipods.iter().all(|amphipod| amphipod.room() == room)
        })
    }

    fn get_spot(position: usize) -> Option<usize> {
        if position == 0 {
            Some(0)
        } else if position < 10 {
            if position % 2 == 1 {
                Some((position + 1) / 2)
            } else {
                None
            }
        } else {
            Some(6)
        }
    }

    fn get_room(position: usize) -> Option<usize> {
        if position == 0 {
            None
        } else if position < 10 {
            if position % 2 == 0 {
                Some((position / 2) - 1)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn locations_between<A: Into<Position>, B: Into<Position>>(
        from: A,
        to: B,
    ) -> impl Iterator<Item = Location> {
        let from: Position = from.into();
        let to: Position = to.into();

        let xs = if from.x < to.x {
            from.x + 1..=to.x
        } else {
            to.x..=from.x - 1
        };

        (0..from.y)
            .rev()
            .map(move |y| Position { x: from.x, y })
            .chain(xs.map(|x| Position { x, y: 0 }))
            .chain((1..=to.y).map(move |y| Position { x: to.x, y }))
            .filter_map(|pos| pos.try_into().ok())
    }

    fn location_clear(&self, location: Location) -> bool {
        use Location::*;
        match location {
            Room { room, depth } => self.rooms[room].len() < self.room_depth - depth,
            Corridor { spot } => self.corridor[spot].is_none(),
        }
    }

    fn is_clear(&self, from: Location, to: Location) -> bool {
        Self::locations_between(from, to).all(|spot| self.location_clear(spot))
    }

    fn available_room_location(&self, room: usize) -> Option<Location> {
        if self.rooms[room].len() < self.room_depth {
            Some(Location::Room {
                room,
                depth: self.room_depth - self.rooms[room].len() - 1,
            })
        } else {
            None
        }
    }

    fn can_move_to_room(&self, location: Location, room: usize) -> Option<Location> {
        self.available_room_location(room).filter(|&room_location| {
            //self.can_move(location) &&
            !location.same_room(room_location)
                && self.is_clear(location, room_location)
                && self.rooms[room]
                    .iter()
                    .all(|amphipod| amphipod.room() == room)
        })
    }

    fn can_move_to_corridor(&self, location: Location, spot: usize) -> Option<Location> {
        let corridor_location = Location::Corridor { spot };
        if location != corridor_location && self.is_clear(location, corridor_location) {
            Some(corridor_location)
        } else {
            None
        }
    }

    fn amphipods(&self) -> impl Iterator<Item = (Location, Amphipod)> + '_ {
        self.amphipods_in_corridor()
            .chain(self.amphipods_in_rooms())
    }

    fn amphipods_in_corridor(&self) -> impl Iterator<Item = (Location, Amphipod)> + '_ {
        self.corridor
            .iter()
            .enumerate()
            .filter_map(|(spot, contents)| {
                contents.map(|amphipod| (Location::Corridor { spot }, amphipod))
            })
    }

    fn amphipods_in_rooms(&self) -> impl Iterator<Item = (Location, Amphipod)> + '_ {
        self.rooms
            .iter()
            .enumerate()
            .flat_map(move |(room, contents)| {
                contents.iter().enumerate().map(move |(height, amphipod)| {
                    (
                        Location::Room {
                            room,
                            depth: self.room_depth - height - 1,
                        },
                        *amphipod,
                    )
                })
            })
    }

    fn min_energy_to_solve(&self) -> usize {
        self.amphipods()
            .map(|(location, amphipod)| {
                amphipod.energy_to_move()
                    * location.distance_to(Location::Room {
                        room: amphipod.room(),
                        depth: 0,
                    })
            })
            .sum()
    }

    fn remove(&mut self, location: Location) -> Option<Amphipod> {
        use Location::*;
        match location {
            Corridor { spot } => self.corridor[spot].take(),
            Room { room, depth } => {
                // Can only remove the top amphipod in a room.
                if self.room_depth - depth == self.rooms[room].len() {
                    self.rooms[room].pop()
                } else {
                    None
                }
            }
        }
    }

    fn add(&mut self, location: Location, amphipod: Amphipod) {
        use Location::*;
        match location {
            Corridor { spot } => {
                assert!(self.corridor[spot].is_none());
                self.corridor[spot] = Some(amphipod)
            }
            Room { room, depth } => {
                // Can only add to the top of the amphipods in a room.
                assert_eq!(self.room_depth - depth, self.rooms[room].len() + 1);
                self.rooms[room].push(amphipod);
            }
        }
    }

    fn moves_to_corridor(&self, location: Location) -> impl Iterator<Item = Location> + '_ {
        (0..7).filter_map(move |spot| self.can_move_to_corridor(location, spot))
    }

    fn do_move(&self, from: Location, to: Location) -> Self {
        let mut layout = self.clone();
        let amphipod = layout.remove(from).unwrap();
        layout.add(to, amphipod);
        layout
    }
}

impl Display for Layout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "#############")?;
        write!(f, "#")?;
        for position in 0..=10 {
            if let Some(spot) = Self::get_spot(position) {
                if let Some(amphipod) = self.corridor[spot] {
                    write!(f, "{}", amphipod)?;
                } else {
                    write!(f, ".")?;
                }
            } else {
                write!(f, ".")?
            }
        }
        writeln!(f, "#")?;

        for index in 0..self.room_depth {
            if index == 0 {
                write!(f, "###")?;
            } else {
                write!(f, "  #")?;
            }
            for position in 2..=8 {
                if let Some(room) = Self::get_room(position) {
                    if let Some(amphipod) = self.rooms[room].get(self.room_depth - index - 1) {
                        write!(f, "{}", amphipod)?;
                    } else {
                        write!(f, ".")?;
                    }
                } else {
                    write!(f, "#")?
                }
            }
            if index == 0 {
                writeln!(f, "###")?;
            } else {
                writeln!(f, "#")?;
            }
        }

        writeln!(f, "  #########   ")
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
struct AmphipodState {
    layout: Layout,
}

impl AmphipodState {
    fn new(layout: Layout) -> Self {
        AmphipodState { layout }
    }

    fn successor(&self, amphipod: Amphipod, from: Location, to: Location) -> (Self, usize) {
        let layout = self.layout.do_move(from, to);
        let new_energy = amphipod.energy_to_move() * from.distance_to(to);

        (AmphipodState { layout }, new_energy)
    }

    fn moves_to_room(&self) -> impl Iterator<Item = (AmphipodState, usize)> + '_ {
        self.layout.amphipods().filter_map(|(location, amphipod)| {
            self.layout
                .can_move_to_room(location, amphipod.room())
                .map(|new_location| self.successor(amphipod, location, new_location))
        })
    }

    fn moves_to_corridor(&self) -> impl Iterator<Item = (AmphipodState, usize)> + '_ {
        self.layout
            .amphipods()
            .flat_map(move |(location, amphipod)| {
                self.layout
                    .moves_to_corridor(location)
                    .map(move |new_location| self.successor(amphipod, location, new_location))
            })
    }
}

impl a_star::State for AmphipodState {
    fn min_remaining_cost(&self) -> usize {
        self.layout.min_energy_to_solve()
    }

    fn successors(&self) -> Box<dyn Iterator<Item = (Self, usize)> + '_> {
        // If an amphipod can move into their final room always do that as
        // they have to do that at some point, it's never going to get cheaper
        // to do so, and once they're in the room they can't affect anything
        // else.
        if let Some(candidate) = self.moves_to_room().next() {
            Box::new([candidate].into_iter()) as Box<dyn Iterator<Item = (AmphipodState, usize)>>
        } else {
            Box::new(self.moves_to_corridor()) as Box<dyn Iterator<Item = (AmphipodState, usize)>>
        }
    }

    fn is_complete(&self) -> bool {
        self.layout.is_complete()
    }
}

fn main() {
    let opt = Opt::from_args();
    let mut layout = Layout::read(opt.input);
    let state = AmphipodState::new(layout.clone());
    let total_energy = a_star::solve(state).unwrap();
    println!("{}", total_energy);

    use Amphipod::*;
    layout.insert_row(1, &[Desert, Copper, Bronze, Amber]);
    layout.insert_row(1, &[Desert, Bronze, Amber, Copper]);

    let state = AmphipodState::new(layout);
    let total_energy = a_star::solve(state).unwrap();
    println!("{}", total_energy);
}

#[cfg(test)]
mod test {
    use super::*;
    use aoc2021::a_star::State;

    #[test]
    fn test_successors_from_rooms() {
        use Amphipod::*;

        let layout = Layout {
            corridor: Default::default(),
            room_depth: 2,
            rooms: [
                vec![Amber, Bronze],
                vec![Desert, Copper],
                vec![Copper, Bronze],
                vec![Amber, Desert],
            ],
        };
        let state = AmphipodState::new(layout);
        let successors = state.successors().collect::<Vec<_>>();
        assert_eq!(successors.len(), 28);
    }
}
