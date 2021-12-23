use std::collections::{BinaryHeap, HashSet};
use std::fmt::Display;
use std::fs::File;
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

    fn spot_position(spot: usize) -> usize {
        if spot == 0 {
            0
        } else if spot < 6 {
            2 * spot - 1
        } else {
            10
        }
    }

    fn room_entrance_position(room: usize) -> usize {
        2 * room + 2
    }

    fn distance_to_room(&self, spot: usize, room: usize) -> usize {
        abs_diff(
            Self::spot_position(spot),
            Self::room_entrance_position(room),
        ) + self.room_depth
            - self.rooms[room].len()
    }

    fn distance_between_rooms(&self, room1: usize, room2: usize) -> usize {
        if room1 != room2 {
            abs_diff(
                Self::room_entrance_position(room1),
                Self::room_entrance_position(room2),
            ) + 2 * self.room_depth
                + 1
                - self.rooms[room1].len()
                - self.rooms[room2].len()
        } else {
            0
        }
    }

    fn distance_from_room(&self, room: usize, spot: usize) -> usize {
        abs_diff(
            Self::spot_position(spot),
            Self::room_entrance_position(room),
        ) + self.room_depth
            + 1
            - self.rooms[room].len()
    }

    fn distance_between_spots(&self, spot1: usize, spot2: usize) -> usize {
        abs_diff(Self::spot_position(spot1), Self::spot_position(spot2))
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

    fn spots_between(source: usize, dest: usize) -> impl Iterator<Item = usize> {
        let positions = if source < dest {
            source + 1..=dest
        } else {
            dest..=source - 1
        };
        positions.filter_map(Self::get_spot)
    }

    fn is_clear(&self, from: usize, to: usize) -> bool {
        Self::spots_between(from, to).all(|spot| self.corridor[spot].is_none())
    }

    fn can_move_from_corridor_to_room(&self, spot: usize, room: usize) -> bool {
        self.is_clear(
            Self::spot_position(spot),
            Self::room_entrance_position(room),
        ) && self.rooms[room].len() < self.room_depth
            && self.rooms[room]
                .iter()
                .all(|amphipod| amphipod.room() == room)
    }

    fn can_move_from_room_to_corridor(&self, room: usize, spot: usize) -> bool {
        self.is_clear(
            Self::room_entrance_position(room),
            Self::spot_position(spot),
        )
    }

    fn can_move_in_corridor(&self, spot1: usize, spot2: usize) -> bool {
        self.is_clear(Self::spot_position(spot1), Self::spot_position(spot2))
    }

    fn can_move_between_rooms(&self, from: usize, to: usize) -> bool {
        from != to
            && self.is_clear(
                Self::room_entrance_position(from),
                Self::room_entrance_position(to),
            )
            && self.rooms[to].len() < self.room_depth
            && self.rooms[to].iter().all(|amphipod| amphipod.room() == to)
    }

    fn amphipods_in_corridor(&self) -> impl Iterator<Item = (usize, Amphipod)> + '_ {
        self.corridor
            .iter()
            .enumerate()
            .filter_map(|(spot, contents)| contents.map(|x| (spot, x)))
    }

    fn amphipods_in_rooms(&self) -> impl Iterator<Item = (usize, usize, usize, Amphipod)> + '_ {
        self.rooms.iter().enumerate().flat_map(|(room, contents)| {
            contents
                .iter()
                .enumerate()
                .map(move |(height, amphipod)| (room, height, contents.len(), *amphipod))
        })
    }

    fn min_energy_to_solve(&self) -> usize {
        self.amphipods_in_corridor()
            .map(|(spot, amphipod)| {
                amphipod.energy_to_move()
                    * (self
                        .distance_to_room(spot, amphipod.room())
                        .saturating_sub((self.room_depth * (self.room_depth - 1)) / 2))
            })
            .sum::<usize>()
            + self
                .amphipods_in_rooms()
                .map(|(room, _, _, amphipod)| {
                    amphipod.energy_to_move()
                        * (self
                            .distance_between_rooms(room, amphipod.room())
                            .saturating_sub((self.room_depth * (self.room_depth - 1)) / 2))
                })
                .sum::<usize>()
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

#[derive(PartialEq, Eq, Debug)]
struct Candidate {
    layout: Layout,
    energy: usize,
    min_energy_remaining: usize,
    history: Option<Vec<(Layout, usize)>>,
}

impl Candidate {
    fn new(layout: Layout, energy: usize, track_history: bool) -> Self {
        let min_energy_remaining = layout.min_energy_to_solve();
        Candidate {
            layout,
            energy,
            min_energy_remaining,
            history: if track_history { Some(vec![]) } else { None },
        }
    }

    fn successor(&self, layout: Layout, new_energy: usize) -> Self {
        let min_energy_remaining = layout.min_energy_to_solve();

        let history = self.history.as_ref().map(|history| {
            let mut history = history.clone();
            history.push((self.layout.clone(), new_energy));
            history
        });

        Candidate {
            layout,
            energy: self.energy + new_energy,
            min_energy_remaining,
            history,
        }
    }

    fn move_from_corridor(&self, spot: usize) -> impl Iterator<Item = Candidate> {
        let mut new_layout = self.layout.clone();
        let amphipod = new_layout.corridor[spot].take().unwrap();

        let mut candidates = vec![];

        let target_room = amphipod.room();
        if self
            .layout
            .can_move_from_corridor_to_room(spot, target_room)
        {
            let mut new_layout = new_layout.clone();
            new_layout.rooms[target_room].push(amphipod);

            candidates.push(self.successor(
                new_layout,
                amphipod.energy_to_move() * self.layout.distance_to_room(spot, target_room),
            ));
        }

        for other_spot in 0..7 {
            if other_spot != spot && self.layout.can_move_in_corridor(spot, other_spot) {
                let mut new_layout = new_layout.clone();
                new_layout.corridor[other_spot] = Some(amphipod);

                candidates.push(self.successor(
                    new_layout,
                    amphipod.energy_to_move()
                        * self.layout.distance_between_spots(spot, other_spot),
                ));
            }
        }

        candidates.into_iter()
    }

    fn move_from_room(&self, room: usize) -> impl Iterator<Item = Candidate> {
        let mut new_layout = self.layout.clone();
        let amphipod = new_layout.rooms[room].pop().unwrap();

        let mut candidates = vec![];

        let target_room = amphipod.room();
        if self.layout.can_move_between_rooms(room, target_room) {
            let mut new_layout = new_layout.clone();
            new_layout.rooms[target_room].push(amphipod);

            candidates.push(self.successor(
                new_layout,
                amphipod.energy_to_move() * self.layout.distance_between_rooms(room, target_room),
            ));
        }

        for spot in 0..7 {
            if self.layout.can_move_from_room_to_corridor(room, spot) {
                let mut new_layout = new_layout.clone();
                new_layout.corridor[spot] = Some(amphipod);

                candidates.push(self.successor(
                    new_layout,
                    amphipod.energy_to_move() * self.layout.distance_from_room(room, spot),
                ));
            }
        }

        candidates.into_iter()
    }

    fn successors(&self) -> impl Iterator<Item = Candidate> + '_ {
        self.layout
            .amphipods_in_corridor()
            .flat_map(|(spot, _)| self.move_from_corridor(spot))
            .chain(
                self.layout
                    .rooms
                    .iter()
                    .enumerate()
                    .filter(|(room, contents)| {
                        contents.iter().any(|amphipod| amphipod.room() != *room)
                    })
                    .flat_map(|(room, _)| self.move_from_room(room)),
            )
    }

    fn print_history(&self) {
        if let Some(ref history) = self.history {
            for (layout, energy) in history.iter() {
                println!("{}", layout);
                println!("Energy: {}", energy);
                println!();
            }
            println!("{}", self.layout);
        }
    }
}

impl PartialOrd for Candidate {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(
            (self.energy + self.min_energy_remaining)
                .cmp(&(other.energy + other.min_energy_remaining))
                .reverse(),
        )
    }
}

impl Ord for Candidate {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.partial_cmp(other).unwrap()
    }
}

fn find_lowest_energy(start_layout: &Layout, track_history: bool) -> Option<usize> {
    let mut heap: BinaryHeap<Candidate> = BinaryHeap::new();
    let mut visited: HashSet<Layout> = HashSet::new();

    heap.push(Candidate::new(start_layout.clone(), 0, track_history));

    while let Some(candidate) = heap.pop() {
        if candidate.layout.is_complete() {
            candidate.print_history();
            return Some(candidate.energy);
        }

        if visited.contains(&candidate.layout) {
            continue;
        }

        visited.insert(candidate.layout.clone());

        for next_candidate in candidate.successors() {
            if !visited.contains(&next_candidate.layout) {
                heap.push(next_candidate);
            }
        }
    }

    None
}

fn main() {
    let opt = Opt::from_args();
    let mut layout = Layout::read(opt.input);
    let total_energy = find_lowest_energy(&layout, false).unwrap();
    println!("{}", total_energy);

    use Amphipod::*;
    layout.insert_row(1, &[Desert, Copper, Bronze, Amber]);
    layout.insert_row(1, &[Desert, Bronze, Amber, Copper]);

    let total_energy = find_lowest_energy(&layout, false).unwrap();
    println!("{}", total_energy);
}

#[cfg(test)]
mod test {
    use super::*;

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
        let candidate = Candidate::new(layout, 0, false);
        let successors = candidate.successors().collect::<Vec<_>>();
        assert_eq!(successors.len(), 28);
    }

    #[test]
    fn test_distance_from_room_to_spot() {
        use Amphipod::*;

        let layout = Layout {
            corridor: [None, None, None, None, None, Some(Desert), None],
            room_depth: 2,
            rooms: [
                vec![Amber, Bronze],
                vec![Desert, Copper],
                vec![Copper, Bronze],
                vec![Amber],
            ],
        };
        assert_eq!(layout.distance_from_room(3, 1), 9);
    }
}
