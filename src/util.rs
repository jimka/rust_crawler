use std::fmt::Display;
use std::str::FromStr;

#[derive(Debug, PartialEq)]
pub struct Size {
    pub width : usize,
    pub height: usize,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone, Hash)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

impl Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}x{}", self.x, self.y)
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Hash)]
pub enum Direction {
    North,
    South,
    West,
    East,
}

impl Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Direction::North => "north",
            Direction::South => "south",
            Direction::West  => "west",
            Direction::East  => "east",
        };

        f.write_str(s)
    }
}

impl FromStr for Direction {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.to_lowercase().as_str() {
            "north" => Ok(Direction::North),
            "south" => Ok(Direction::South),
            "west"  => Ok(Direction::West),
            "east"  => Ok(Direction::East),
            _       => Err(String::from("Unknown direction")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::{ assert_eq, collections::HashMap };

    #[test]
    fn position_display() {
        let p = Position { x: 4, y: 4 };
        let s = format!("{p}");

        assert_eq!(s, "4x4");
    }

    #[test]
    fn direction_display() {
        assert_eq!(format!("{}", Direction::North), "north");
        assert_eq!(format!("{}", Direction::South), "south");
        assert_eq!(format!("{}", Direction::West), "west");
        assert_eq!(format!("{}", Direction::East), "east");
    }

    #[test]
    fn direction_fromstr() {
        let map = HashMap::from([
            ( Direction::North, "north".to_string() ),
            ( Direction::South, "south".to_string() ),
            ( Direction::West, "west".to_string() ),
            ( Direction::East, "east".to_string() )
        ]);

        for (d, s) in map {
            let Ok(d2) = s.parse::<Direction>() else {
                panic!("Unable to parse direction.");
            };

            assert_eq!(d, d2);
        }

        let Err(s) = "sheep".to_string().parse::<Direction>() else {
            panic!("Sheep is a direction?!");
        };

        assert_eq!(s, "Unknown direction");
    }
}