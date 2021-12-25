use std::collections::HashMap;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Position {
    pub x: i64,
    pub y: i64,
}

impl Position {
    pub fn new(x: i64, y: i64) -> Self {
        Position { x, y }
    }

    pub fn offset(self, dx: i64, dy: i64) -> Self {
        Position {
            x: self.x + dx,
            y: self.y + dy,
        }
    }

    pub fn step(self, direction: Direction) -> Self {
        use Direction::*;
        match direction {
            North => self.offset(0, -1),
            East => self.offset(1, 0),
            South => self.offset(0, 1),
            West => self.offset(-1, 0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    North,
    East,
    South,
    West,
}

impl TryFrom<char> for Direction {
    type Error = ();

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            '^' => Ok(Direction::North),
            '>' => Ok(Direction::East),
            'v' => Ok(Direction::South),
            '<' => Ok(Direction::West),
            _ => Err(()),
        }
    }
}

impl From<Direction> for char {
    fn from(direction: Direction) -> Self {
        match direction {
            Direction::North => '^',
            Direction::East => '>',
            Direction::South => 'v',
            Direction::West => '<',
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct TorusMap<T> {
    map: HashMap<Position, T>,
    width: i64,
    height: i64,
}

impl<T> TorusMap<T> {
    pub fn new(map: HashMap<Position, T>, width: i64, height: i64) -> Self {
        TorusMap { map, width, height }
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Position, &T)> + '_ {
        self.map.iter()
    }

    fn wrap(&self, position: &Position) -> Position {
        Position {
            x: position.x % self.width,
            y: position.y % self.height,
        }
    }

    pub fn width(&self) -> i64 {
        self.width
    }

    pub fn height(&self) -> i64 {
        self.height
    }

    pub fn get(&self, position: &Position) -> Option<&T> {
        self.map.get(&self.wrap(position))
    }

    pub fn insert(&mut self, position: Position, contents: T) -> Option<T> {
        self.map.insert(self.wrap(&position), contents)
    }

    pub fn remove(&mut self, position: &Position) -> Option<T> {
        self.map.remove(&self.wrap(position))
    }

    pub fn contains_key(&self, position: &Position) -> bool {
        self.map.contains_key(&self.wrap(position))
    }

    pub fn map<F>(&self, update: F) -> Self
    where
        F: FnMut((&Position, &T)) -> (Position, T),
    {
        let map = self
            .map
            .iter()
            .map(update)
            .map(|(position, val)| (self.wrap(&position), val))
            .collect();
        Self::new(map, self.width, self.height)
    }

    pub fn make_moves<I>(&mut self, moves: I)
    where
        I: IntoIterator<Item = (Position, Position)>,
    {
        for (from, to) in moves {
            if let Some(contents) = self.remove(&from) {
                self.insert(to, contents);
            }
        }
    }
}
