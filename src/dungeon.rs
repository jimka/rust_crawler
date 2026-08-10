use std::{collections::{BTreeMap, HashMap}, fmt::Display};

use crate::{glyph::Glyph, inventory::Inventory, util::{Direction,Position,Size}};

pub struct Dungeon {
    pub size: Size,
    rooms: HashMap<Position, Room>,
    passages: HashMap<String, Passage>
}

#[derive(Debug, PartialEq)]
pub enum AddRoomError {
    OutOfBounds,
    Occupied
}

impl Dungeon {

    pub fn new_with_size(size: Size) -> Dungeon {
        Dungeon {
            size,
            rooms: HashMap::new(),
            passages: HashMap::new(),
        }
    }

    pub fn add_passage(&mut self, id: String, passage: Passage) -> &mut Self{
        self.passages.insert(id, passage);

        self
    }

    pub fn get_passage(&self, id: &str) -> Option<&Passage> {
        self.passages.get(id)
    }

    pub fn get_passage_mut(
        &mut self,
        passage_id: &str,
    ) -> Option<&mut Passage> {
        self.passages.get_mut(passage_id)
    }

    pub fn add_room(&mut self, position: Position, room: Room) -> Result<&mut Self, AddRoomError> {
        if position.x >= self.size.width || position.y >= self.size.height {
            return Err(AddRoomError::OutOfBounds);
        }

        if self.rooms.contains_key(&position) {
            return Err(AddRoomError::Occupied);
        }

        self.rooms.insert(position, room);

        Ok(self)
    }

    #[cfg(test)]
    pub fn get_room_count(&self) -> usize {
        self.rooms.len()
    }

    pub fn get_room(&self, position: Position) -> Option<&Room> {
        self.rooms.get(&position)
    }

    pub fn get_room_mut(&mut self, position: Position) -> Option<&mut Room> {
        self.rooms.get_mut(&position)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum DoorState {
    Open,
    Closed,
    Locked
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Passage {
    Room { room_1: Position, room_2: Position },
    Door { state: DoorState, glyph: Glyph, room_1: Position, room_2: Position },
}

impl Passage {

    pub fn get_description(&self) -> String {
        match &self {
            Passage::Room { .. } => describe_passage(self),
            Passage::Door { .. } => describe_door(self),
        }
    }
}

fn describe_passage(arg: &Passage) -> String{
    let Passage::Room { .. } = arg else {
        panic!("Should only be rooms here!")
    };

    "You see a dark passage.".to_string()
}

fn describe_door(arg: &Passage)  -> String {
    let Passage::Door { state, glyph, .. } = arg else {
        panic!("Should only be doors here!")
    };

    let state_string = match state {
        DoorState::Open => "an opened",
        DoorState::Closed => "a closed",
        DoorState::Locked => "a locked",
    };

        let glyph_string = glyph.to_string().to_lowercase();

        let is_an = glyph_string.starts_with(['a', 'e', 'i', 'o', 'u'])
                              && ![Glyph::Unicorn, Glyph::Ouroboros].contains(glyph);

    format!("You see {state_string} door marked with {} {} glyph.",
        if is_an { "an" } else { "a" },
        glyph_string
    )
}

impl Display for Passage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Passage::Room { room_1, room_2 } => write!(f, "{room_1} <-> {room_2}"),
            Passage::Door { room_1, room_2, .. } => write!(f, "{room_1} <-> {room_2}"),
        }
    }
}

pub struct Room {
    pub passage: BTreeMap<Direction, String>,
    pub inventory: Inventory
}

impl Room {
    pub fn new() -> Self {
        Self {
            passage: BTreeMap::new(),
            inventory: Inventory::new()
        }
    }

    pub fn add_passage(&mut self, direction: Direction, passage: String) -> &mut Self {
        self.passage.insert(direction, passage);

        self
    }

    pub fn get_passage(&self, direction: Direction) -> Option<&String> {
        self.passage.get(&direction)
    }
    
    pub fn get_passages(&self) -> &BTreeMap<Direction, String> {
        &self.passage
    }
    
    pub(crate) fn get_inventory(&self) -> &Inventory {
        &self.inventory
    }

    pub(crate) fn get_inventory_mut(&mut self) -> &mut Inventory {
        &mut self.inventory
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::{ assert_eq };

    use crate::util::{Direction,Position,Size};

    #[test]
    fn room_add_passage() {
        let mut room = Room::new();
        room.add_passage(
            Direction::South,
            "passage".to_string()
        );

        assert_eq!(room.get_passages().len(), 1);
        let Some(_) = room.get_passage(Direction::South) else {
            panic!("Added passage could not be retrieved.");
        };
    }

    #[test]
    fn dungeon_new() {
        let dungeon = Dungeon::new_with_size(Size::new(16, 16));

        assert_eq!(dungeon.rooms.len(), 0);
        assert_eq!(dungeon.size, Size { width: 16, height: 16 });
    }

    #[test]
    fn dungeon_new_with_size() {
        let dungeon = Dungeon::new_with_size(Size { width: 5, height: 5 });

        assert_eq!(dungeon.rooms.len(), 0);
        assert_eq!(dungeon.size, Size { width: 5, height: 5 });
    }

    #[test]
    fn add_room_success() {
        let room1 = Room::new();

        let mut dungeon = Dungeon::new_with_size(Size::new(16, 16));
        
        let Ok(_) = dungeon.add_room(Position { x: 0, y: 0 }, room1) else {
            panic!("Generated an error.");
        };

        assert_eq!(dungeon.rooms.len(), 1);
        assert!(dungeon.get_room(Position { x: 0, y: 0 }).is_some());
    }

    #[test]
    fn dungeon_add_room_out_of_bounds() {
        let room = Room::new();

        let mut dungeon = Dungeon::new_with_size(Size::new(16, 16));
        
        let Err(e) = dungeon.add_room(Position { x: 20, y: 20 }, room) else {
            panic!("Did not generate an OutOfBounds error.");
        };

        assert_eq!(e, AddRoomError::OutOfBounds);
    }

    #[test]
    fn dungeon_add_room_occupied() {
        let room1 = Room::new();
        let room2 = Room::new();

        let mut dungeon = Dungeon::new_with_size(Size::new(16, 16));
        
        let Ok(_) = dungeon.add_room(Position { x: 0, y: 0 }, room1) else {
            panic!("Generated an error.");
        };

        let Err(e) = dungeon.add_room(Position { x: 0, y: 0 }, room2) else {
            panic!("Did not generate an occupied error.");
        };

        assert_eq!(e, AddRoomError::Occupied);
    }

    #[test]
    fn dungeon_get_room_mut() {
        let room1 = Room::new();

        let mut dungeon = Dungeon::new_with_size(Size::new(16, 16));
        
        let Ok(_) = dungeon.add_room(Position { x: 0, y: 0 }, room1) else {
            panic!("Generated an error.");
        };

        assert!(dungeon.get_room_mut(Position { x: 0, y: 0 }).is_some());
    }

    #[test]
    fn dungeon_get_passage_mut() {
        let mut dungeon = Dungeon::new_with_size(Size::new(16, 16));

        dungeon.add_passage(
            "passage".to_string(),
            Passage::Door {
                state: DoorState::Open,
                glyph: Glyph::Alerion,
                room_1: Position { x: 0, y: 0 },
                room_2: Position { x: 1, y: 1 },
            }
        );
        
        assert!(matches!(
            dungeon.get_passage_mut("passage"),
            Some(Passage::Door {
                state: DoorState::Open,
                glyph: Glyph::Alerion,
                room_1: Position { x: 0, y: 0 },
                room_2: Position { x: 1, y: 1 },
            })
        ));
    }

    #[test]
    fn position_display() {
        let p = Position { x: 4, y: 4 };
        let s = format!("{p}");

        assert_eq!(s, "4x4");
    }
}