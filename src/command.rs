use std::{str::FromStr};

use crate::dungeon::{Dungeon,Passage,DoorState};
use crate::inventory::{ItemType, Key};
use crate::player::Player;
use crate::util::{Direction,Position};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Command {
    Go { direction: Direction },
    Look,
    Open { direction: Direction },
    Close { direction: Direction },
    Take { id: String },
    Put { id: String },
    Inventory,
    Use { item_id: String },
    Map,
    Unknown,
    Blank,
    Incomplete { reason: &'static str } ,
    Exit
}

impl FromStr for Command {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        const INVALID_DIRECTION: &str = "That's not a valid direction!";

        let s = s.trim().to_lowercase();
        let mut tokens = s.split_whitespace();

        let command = match tokens.next() {
            Some(c) => c,
            None => return Ok(Command::Blank)
        };

        match command {
            "go" => {
                match take_direction(&mut tokens) {
                    Ok(d) => Ok(Command::Go { direction: d }),
                    Err(TakeDirectionError::Incomplete) => Ok(Command::Incomplete { reason: "Go where?!" }),
                    Err(TakeDirectionError::Invalid) => Ok(Command::Incomplete { reason: INVALID_DIRECTION }),
                }
            },
            "look" => Ok(Command::Look),
            "open" => {
                match take_direction(&mut tokens) {
                    Ok(d) => Ok(Command::Open { direction: d }),
                    Err(TakeDirectionError::Incomplete) => Ok(Command::Incomplete { reason: "Open what?!" }),
                    Err(TakeDirectionError::Invalid) => Ok(Command::Incomplete { reason: INVALID_DIRECTION }),
                }
            },
            "close" => {
                match take_direction(&mut tokens) {
                    Ok(d) => Ok(Command::Close { direction: d }),
                    Err(TakeDirectionError::Incomplete) => Ok(Command::Incomplete { reason: "Close what?!" }),
                    Err(TakeDirectionError::Invalid) => Ok(Command::Incomplete { reason: INVALID_DIRECTION }),
                }
            },
            "take" => {
                match tokens.next() {
                    Some(id) => Ok(Command::Take { id: id.to_string() }),
                    None => Ok(Command::Incomplete { reason: "Take what?!" }),
                }
                
            },
            "put" => {
                match tokens.next() {
                    Some(id) => Ok(Command::Put { id: id.to_string() }),
                    None => Ok(Command::Incomplete { reason: "Put what?!" }),
                }
                
            },
            "map" => Ok(Command::Map),
            "inventory" => Ok(Command::Inventory),
            "use" => {
                match tokens.next() {
                    Some(id) => Ok(Command::Use { item_id: id.to_string() }),
                    None => Ok(Command::Incomplete { reason: "Use what?!" }),
                }
            },
            "exit" => Ok(Command::Exit),
            _ => Ok(Command::Unknown)
        }
    }
}

pub enum TakeDirectionError {
    Incomplete,
    Invalid
}

pub fn take_direction(tokens: &mut std::str::SplitWhitespace<'_>) -> Result<Direction, TakeDirectionError> {
    let argument = match tokens.next() {
        Some(a) => a,
        None => return Err(TakeDirectionError::Incomplete)
    };

    let direction = match argument.parse::<Direction>() {
        Ok(d) => d,
        Err(_) => return Err(TakeDirectionError::Invalid)
    };

    Ok(direction)
}

pub struct CommandResult {
    pub running: bool,
    pub output: Vec<String>
}

pub fn flip_door(
    dungeon: &mut Dungeon,
    current_room: Position,
    direction: Direction,
    target_state: DoorState,
) -> Vec<String> {
    let Some(room) = dungeon.get_room(current_room) else {
        panic!("Player is in an unknown room!")
    };

    let Some(passage_id) = room.get_passage(direction) else {
        return vec![format!("There's no door in that direction!")];
    };

    // Clone to detach from Dungeon lifetime.
    let passage_id = passage_id.clone();

    let current_state = match dungeon.get_passage_mut(&passage_id) {
        Some(Passage::Door { state, .. }) => state,
        _ => return vec![format!("That's not a door!")],
    };

    if target_state == DoorState::Locked {
        panic!("Cannot flip door to 'Locked'")
    } else if *current_state == DoorState::Locked {
        return vec![format!("The door is locked!")];
    }
    
    if target_state == *current_state {
        let current_state = if target_state == DoorState::Closed { "closed" } else { "open" };

        return vec![format!("The door is already {current_state}!")];
    }

    *current_state = target_state;

    vec![format!("You {} the door.", if target_state == DoorState::Closed { "close" } else { "open" })]
}

pub fn process_command(dungeon: &mut Dungeon, player: &mut Player, command: Command) -> CommandResult {
    let (running, output) = match command {
        Command::Go { direction } => ( true, go(dungeon, player, direction) ),
        Command::Look => ( true, look(dungeon, player.get_position()) ),
        Command::Open { direction } => ( true, flip_door(dungeon, player.get_position(), direction, DoorState::Open) ),
        Command::Close { direction } => ( true, flip_door(dungeon, player.get_position(), direction, DoorState::Closed) ),
        Command::Take { id } => ( true, take(dungeon, player, &id) ),
        Command::Put { id } => ( true, put(dungeon, player, &id) ),
        Command::Inventory => ( true, inventory(player) ),
        Command::Map => ( true, map(dungeon, player) ),
        Command::Use { item_id } => ( true, use_item(dungeon, player, item_id) ),
        Command::Unknown => ( true, vec!["I don't understand what you want to do!".to_string()] ),
        Command::Blank => ( true, vec![] ),
        Command::Incomplete { reason } => ( true, vec![reason.to_string()] ),
        Command::Exit => ( false, vec![] ),
    };

    CommandResult { running, output }
}

pub fn go(dungeon: &Dungeon, player: &mut Player, direction: Direction) -> Vec<String> {
    let room = match dungeon.get_room(player.get_position()) {
        Some(r) => r,
        None => panic!("Room doesn't exist!")
    };

    let passage_id = match room.get_passage(direction) {
        Some(p) => p,
        None => return vec![String::from("You can't go in that direction from this room!")]
    };

    let mut output: Vec<String> = Vec::new();

    let Some(passage) = dungeon.get_passage(passage_id) else {
        panic!("Unknown passage referenced!")
    };

    let result: Option<&str> = match passage {
        Passage::Room { room_1, room_2 } => {
            let new_position = if player.get_position() == *room_1 { room_2 } else { room_1 };
            player.set_position(*new_position);

            Some("passage")
        }
        Passage::Door {
            state,
            room_1,
            room_2,
        } => {
            match state {
                DoorState::Closed => return vec!["The door is closed.".to_string()],
                DoorState::Locked => return vec!["The door is locked.".to_string()],
                DoorState::Open => {
                    let new_position = if player.get_position() == *room_1 { room_2 } else { room_1 };
                    player.set_position(*new_position);

                    Some("door")
                }
            }
        }
    };

    if let Some(passage_type) = result {
        output.push(format!("You go through the {direction} {passage_type}."));
    };

    if !output.is_empty() {
        output.push(String::from(""));
    }

    output.extend(look(dungeon, player.get_position()));

    output
}

pub fn look(dungeon: &Dungeon, current_room: Position) -> Vec<String> {
    let mut output: Vec<String> = vec![];

    let room = match dungeon.get_room(current_room) {
        Some(r) => r,
        None => panic!("Room doesn't exist!")
    };

    let available_directions: Vec<String> = room.get_passages()
        .iter()
        .filter(|(_, p)| !matches!(dungeon.get_passage(p).unwrap(), Passage::Door { .. }))
        .map(|(direction, _)| direction.to_string())
        .collect();

    let passages = match available_directions.as_slice() {
        [] => "".to_string(),
        [one] => format!("There is one passage leading {}.", one),
        [one, two] => format!("You see passages to the {} and {}.", one, two),
        [rest @ .., last] => format!(
            "You see passages to the {}, and {}.",
            rest.join(", "),
            last
        ),
    };

    let available_doors: Vec<String> = room.get_passages()
        .iter()
        .filter(|(_, p)| matches!(dungeon.get_passage(p).unwrap(), Passage::Door { .. }))
        .map(|(direction, _)| direction.to_string())
        .collect();

    let doors = match available_doors.as_slice() {
        [] => "".to_string(),
        [one] => format!("To the {} there is a door.", one),
        [rest @ .., last] => format!("You see doors to the {}, and {}.", rest.join(", "), last),
    };

    if !passages.is_empty() {
        output.push(passages);
    }

    if !doors.is_empty() {
        output.push(doors);
    }

    let inventory = room.get_inventory();
    if !inventory.is_empty() {
        list_inventory(&mut output, inventory);
    }

    output
}

fn list_inventory(output: &mut Vec<String>, inventory: &crate::inventory::Inventory) {
    output.push("You see the following items:".to_string());

    // How do I make inventory be an iterator?
    for item in inventory.get_items() {
        let description = item.get_description();
        output.push(format!("  * {description}"));
    }
}

fn take(dungeon: &mut Dungeon, player: &mut Player, item_id: &str) -> Vec<String> {
    let Some(room) = dungeon.get_room_mut(player.get_position()) else {
        panic!("Room doesn't exist!")
    };

    let from_inventory = room.get_inventory_mut();
    let to_inventory = player.get_inventory_mut();

    let Some(item) = from_inventory.take_item(item_id) else {
        return vec!("No such item exists!".to_string());
    };

    to_inventory.put_item(item);

    vec![format!("You take {item_id} and place it into your inventory.")]
}

fn put(dungeon: &mut Dungeon, player: &mut Player, item_id: &str) -> Vec<String> {
    let Some(room) = dungeon.get_room_mut(player.get_position()) else {
        panic!("Room doesn't exist!")
    };

    let from_inventory = player.get_inventory_mut();
    let to_inventory = room.get_inventory_mut();

    let Some(item) = from_inventory.take_item(item_id) else {
        return vec!("No such item exists!".to_string());
    };

    to_inventory.put_item(item);

    vec![format!("You take {item_id} from your inventory and put it in the room.")]
}

fn inventory(player: &Player) -> Vec<String> {
    let mut output: Vec<String> = Vec::new();

    let inventory = player.get_inventory();
    if !inventory.is_empty() {
        list_inventory(&mut output, inventory);
    }

    output
}

fn map(dungeon: &Dungeon, player: &Player) -> Vec<String> {
    let mut output = vec![];

    for y in 0..dungeon.size.height {
        let mut row = String::new();
        let mut sub_row: String = String::new();

        for x in 0..dungeon.size.width {
            let Some(room) = dungeon.get_room(Position { x, y }) else {
                row     += "    ";
                sub_row += "    ";

                continue;
            };

            if player.get_position() == (Position { x, y }) {
                row += "X";
            } else {
                row += "#";
            }

            if let Some(passage_id) = room.get_passage(Direction::East) {
                let Some(passage) = dungeon.get_passage(passage_id) else {
                    panic!("Unable to find passage")
                };

                row += match passage {
                    Passage::Room { .. }                           => " - ",
                    Passage::Door { state: DoorState::Locked, .. } => " & ",
                    Passage::Door { state: DoorState::Closed, .. } => " = ",
                    Passage::Door { state: DoorState::Open, .. }   => " + ",
                };
            } else {
                row += "   ";
            }

            if let Some(passage_id) = room.get_passage(Direction::South) {
                let Some(passage) = dungeon.get_passage(passage_id) else {
                    panic!("Unable to find passage")
                };

                sub_row += match passage {
                    Passage::Room { .. }                           => "|  ",
                    Passage::Door { state: DoorState::Locked, .. } => "&  ",
                    Passage::Door { state: DoorState::Closed, .. } => "=  ",
                    Passage::Door { state: DoorState::Open, .. }   => "+  ",
                };
            } else {
                sub_row += "   ";
            }
        }

        row = row.trim_end().to_string();
        if !row.is_empty() {
            output.push(row);
        }

        sub_row = sub_row.trim().to_string();
        if !sub_row.is_empty() {
            output.push(sub_row);
        }
    }

    output
}

fn use_item(dungeon: &mut Dungeon, player: &Player, item_id: String) -> Vec<String> {
    let Some(item) = player.get_inventory().get_item(&item_id) else {
        return vec![format!("You don't have {item_id}")];
    };

    match item.get_type() {
        ItemType::Key => use_key(dungeon, player, item.as_key().unwrap()),
    }
}

fn use_key(dungeon: &mut Dungeon, player: &Player, key: &Key) -> Vec<String> {
    let door = key.get_door();
    let Some(Passage::Door { state, room_1, room_2 }) = dungeon.get_passage_mut(door) else {
        panic!("Couldn't find door!")
    };

    if *room_1 != player.get_position() && *room_2 != player.get_position(){
        return vec!["The key doesn't fit any door in this room!".to_string()];
    }
   
    let response = match state {
        DoorState::Open => "The door is open and you cannot lock or unlock an open door.".to_string(),
        DoorState::Closed => {
            *state = DoorState::Locked;

            "You lock the door.".to_string()
        },
        DoorState::Locked => {
            *state = DoorState::Closed;

            "You unlock the door.".to_string()
        },
    };

    vec![response]
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::{ assert_eq, collections::{HashMap} };

    use crate::dungeon::{Dungeon,Passage,Room};
    use crate::util::{Direction,Position};

    #[test]
    fn command_fromstr() {
        let map = HashMap::from([
            ( "go north"       , Command::Go { direction: Direction::North } ),
            ( "go"             , Command::Incomplete { reason: "Go where?!" } ),
            ( "look"           , Command::Look ),
            ( "open east"      , Command::Open { direction: Direction::East } ),
            ( "open up"        , Command::Incomplete { reason: "That's not a valid direction!" } ),
            ( "open"           , Command::Incomplete { reason: "Open what?!" } ),
            ( "close west"     , Command::Close { direction: Direction::West } ),
            ( "close down"     , Command::Incomplete { reason: "That's not a valid direction!" } ),
            ( "close"          , Command::Incomplete { reason: "Close what?!" } ),
            ( "hello world"    , Command::Unknown ),
            ( ""               , Command::Blank ),
            ( "    "           , Command::Blank ),
            ( "go up!"         , Command::Incomplete { reason: "That's not a valid direction!" } ),
            ( "exit"           , Command::Exit ),
        ]);

        for (s, d) in map {
            let Ok(d2) = s.parse::<Command>() else {
                panic!("Unable to parse command.");
            };

            assert_eq!(d, d2);
        }
    }

    fn connect_passage(dungeon: &mut Dungeon, passage_id: String, room_1: Position, room_2: Position) {
        let diff = (
            (room_1.x as i8 - room_2.x as i8),
            (room_1.y as i8 - room_2.y as i8)
        );
    
        println!("room_1: {room_1}");
        println!("room_2: {room_2}");

        if diff.0 == diff.1 || diff.0 < -1 || diff.0 > 1 || diff.1 < -1 || diff.1 > 1 {
            panic!("Rooms are not next to each other.")
        }
    
        let direction = if diff.0 == -1 {
            Direction::East
        } else if diff.0 == 1 {
            Direction::West
        } else if diff.1 == -1 {
            Direction::South
        } else {
            Direction::North
        };
    
        let opposite_direction = match direction {
            Direction::North => Direction::South,
            Direction::South => Direction::North,
            Direction::West => Direction::East,
            Direction::East => Direction::West,
        };

        println!("Direction: {direction}");
        println!("Opposite Direction: {opposite_direction}");

        let room_1 = if let Some(room) = dungeon.get_room_mut(room_1) {
            room
        } else {
            let _ = dungeon.add_room(room_1, Room::new());
    
            dungeon.get_room_mut(room_1).unwrap()
        };
    
        room_1.add_passage(direction, passage_id.clone());
    
        let room_2 = if let Some(room) = dungeon.get_room_mut(room_2) {
            room
        } else {
            let _ = dungeon.add_room(room_2, Room::new());
    
            dungeon.get_room_mut(room_2).unwrap()
        };
    
        room_2.add_passage(opposite_direction, passage_id.clone());
    }

    fn connect_room(
        dungeon: &mut Dungeon,
        passage_id: String,
        room_1: Position,
        room_2: Position
    ) {
        dungeon.add_passage(passage_id.clone(), Passage::Room { room_1, room_2 });
        
        connect_passage(dungeon, passage_id, room_1, room_2);
    }

    fn connect_door(
        dungeon: &mut Dungeon,
        passage_id: String,
        state: DoorState,
        room_1: Position,
        room_2: Position
    ) {
        dungeon.add_passage(passage_id.clone(), Passage::Door { state, room_1, room_2 });
        
        connect_passage(dungeon, passage_id, room_1, room_2);
    }

    #[test]
    #[should_panic(expected="Player is in an unknown room!")]
    fn flip_door_missing_room() {
        let mut dungeon = Dungeon::new();
        let current_room = Position { x: 10, y: 10 };
        let direction = Direction::North;
        let should_be_closed = DoorState::Open;

        let _ = super::flip_door(
            &mut dungeon,
            current_room,
            direction,
            should_be_closed
        );
    }

    #[test]
    fn flip_door_missing_door() {
        let mut dungeon = Dungeon::new();
        let _ = dungeon.add_room(
            Position {
                x: 10,
                y: 10
            },
            Room::new()
        );

        let current_room = Position { x: 10, y: 10 };
        let target_state = DoorState::Open;
        let direction = Direction::North;

        let result = super::flip_door(
            &mut dungeon,
            current_room,
            direction,
            target_state
        );

        let expected: Vec<String> = vec![
            "There's no door in that direction!".to_string()
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn flip_door_already_open() {
        let mut dungeon = Dungeon::new();
        connect_door(&mut dungeon, 
            "passage".to_string(),
            DoorState::Open,
            Position {
                x: 10,
                y: 10
            }, Position {
                x: 10,
                y: 9
            }
        );

        let current_room = Position { x: 10, y: 10 };
        let direction = Direction::North;
        let target_state = DoorState::Open;

        let result = super::flip_door(
            &mut dungeon,
            current_room,
            direction,
            target_state
        );

        let expected: Vec<String> = vec![
            "The door is already open!".to_string()
        ];

        assert_eq!(result, expected);
    }

    #[test]
    fn flip_door_success() {
        let passage_id = "passage".to_string();
        let mut dungeon = Dungeon::new();
        connect_door(&mut dungeon, 
            passage_id.clone(),
            DoorState::Closed,
            Position {
                x: 10,
                y: 10
            }, Position {
                x: 10,
                y: 9
            }
        );

        let current_room = Position { x: 10, y: 10 };
        let direction = Direction::North;
        let target_state = DoorState::Open;

        let result = super::flip_door(
            &mut dungeon,
            current_room,
            direction,
            target_state
        );

        let expected: Vec<String> = vec![
            "You open the door.".to_string()
        ];

        assert_eq!(result, expected);

        assert!(matches!(
            dungeon.get_passage(&passage_id),
            Some(Passage::Door { state: DoorState::Open, .. })
        ));
    }

    #[test]
    #[should_panic(expected="Player is in an unknown room!")]
    fn flip_door_destination_doesnt_exist() {
        let mut dungeon = Dungeon::new();
        let mut room = Room::new();

        dungeon.add_passage(
            "passage".to_string(),
            Passage::Door {
                state: DoorState::Open,
                room_1: Position {
                    x: 10,
                    y: 10
                },
                room_2: Position {
                    x: 10,
                    y: 11
                }
            }
        );
        
        room.add_passage(Direction::North, "passage".to_string());

        let current_room = Position { x: 10, y: 10 };
        let direction = Direction::North;
        let target_state = DoorState::Open;

        let _ = super::flip_door(
            &mut dungeon,
            current_room,
            direction,
            target_state
        );
    }

    #[test]
    #[should_panic(expected="Room doesn't exist!")]
    fn go_unknown_room() {
        let dungeon = Dungeon::new();
        let mut player = Player::new(
            Position { x: 10, y: 10 }
        );
        let direction = Direction::North;

        let _ = go(&dungeon, &mut player, direction);
    }

    #[test]
    fn go_no_passage() {
        let mut dungeon = Dungeon::new();
        let _ = dungeon.add_room(
            Position {
                x: 10,
                y: 10
            },
            Room::new()
        );

        let mut player = Player::new(
            Position { x: 10, y: 10 }
        );
        let direction = Direction::North;
        let expected: Vec<String> = vec!["You can't go in that direction from this room!".to_string()];

        let result = go(&dungeon, &mut player, direction);

        assert_eq!(result, expected);
    }

    #[test]
    fn go_through_closed_door() {
        let passage_id = "passage".to_string();
        let mut dungeon = Dungeon::new();
        connect_door(&mut dungeon, 
            passage_id.clone(),
            DoorState::Closed,
            Position {
                x: 10,
                y: 10
            }, Position {
                x: 10,
                y: 9
            }
        );

        let mut player = Player::new(
            Position { x: 10, y: 10 }
        );
        let direction = Direction::North;
        let expected: Vec<String> = vec![
            "The door is closed.".to_string(),
        ];

        let result = go(&dungeon, &mut player, direction);

        assert_eq!(result, expected);
    }

   #[test]
    fn go_through_opened_door() {
        let passage_id = "passage".to_string();
        let mut dungeon = Dungeon::new();
        connect_door(&mut dungeon,
            passage_id.clone(),
            DoorState::Open,
            Position {
                x: 10,
                y: 10
            }, Position {
                x: 10,
                y: 9
            }
        );

        let mut player = Player::new(
            Position { x: 10, y: 10 }
        );
        let direction = Direction::North;
        let expected: Vec<String> = vec![
            "You go through the north door.".to_string(),
            "".to_string(),
            "To the south there is a door.".to_string(),
        ];

        let result = go(&dungeon, &mut player, direction);

        assert_eq!(result, expected);
    }

    #[test]
    #[should_panic(expected = "Room doesn't exist")]
    fn go_through_passage_to_missing_room() {
        let passage_id = "passage".to_string();
        let room_1 = Position { x: 10, y: 10 };
        let room_2 = Position { x: 10, y: 9 };
        let mut dungeon = Dungeon::new();

        dungeon.add_passage(passage_id.clone(), Passage::Room { room_1, room_2 });

        let mut r = Room::new();
        r.add_passage(Direction::North, passage_id.clone());
        let _ = dungeon.add_room(room_1, r);

        let mut player = Player::new(
            Position { x: 10, y: 10 }
        );
        let direction = Direction::North;

        let _ = go(&dungeon, &mut player, direction);
    }

    #[test]
    fn go_through_passage() {
        let passage_id = "passage".to_string();
        let mut dungeon = Dungeon::new();
        connect_room(&mut dungeon,
            passage_id.clone(),
            Position {
                x: 10,
                y: 10
            }, Position {
                x: 10,
                y: 9
            }
        );

        let mut player = Player::new(
            Position { x: 10, y: 10 }
        );
        let direction = Direction::North;
        let expected: Vec<String> = vec![
            "You go through the north passage.".to_string(),
            "".to_string(),
            "There is one passage leading south.".to_string(),
        ];

        let result = go(&dungeon, &mut player, direction);

        assert_eq!(result, expected);
    }


    #[test]
    #[should_panic(expected = "Room doesn't exist")]
    fn look_unknown_room() {
        let dungeon = Dungeon::new();
        let current_room = Position { x: 10, y: 10 };

        super::look(&dungeon, current_room);
    }

    #[test]
    fn look_no_passages() {
        let mut dungeon = Dungeon::new();
        let _ = dungeon.add_room(
            Position {
                x: 10,
                y: 10
            },
            Room::new()
        );
        let current_room = Position { x: 10, y: 10 };
        let expected: Vec<String> = vec![
        ];

        let result = super::look(&dungeon, current_room);

        assert_eq!(expected, result);
    }

    #[test]
    fn look_one_passage() {
        let passage_id = "passage".to_string();
        let mut dungeon = Dungeon::new();
        connect_room(&mut dungeon,
            passage_id.clone(),
            Position {
                x: 10,
                y: 10
            }, Position {
                x: 10,
                y: 9
            }
        );

        let current_room = Position { x: 10, y: 10 };
        let expected: Vec<String> = vec![
            "There is one passage leading north.".to_string()
        ];

        let result = super::look(&dungeon, current_room);

        assert_eq!(expected, result);
    }

    #[test]
    fn look_two_passages() {
        let passage_id_1 = "passage_1".to_string();
        let passage_id_2 = "passage_2".to_string();
        let mut dungeon = Dungeon::new();
        connect_room(
            &mut dungeon,
            passage_id_1.clone(),
            Position {
                x: 10,
                y: 10
            }, Position {
                x: 10,
                y: 9
            }
        );

        connect_room(
            &mut dungeon,
            passage_id_2.clone(),
            Position {
                x: 10,
                y: 10
            }, Position {
                x: 10,
                y: 11
            }
        );

        let current_room = Position { x: 10, y: 10 };
        let expected: Vec<String> = vec![
            "You see passages to the north and south.".to_string()
        ];

        let result = super::look(&dungeon, current_room);

        assert_eq!(expected, result);
    }

    #[test]
    fn look_three_passages() {
        let passage_id_1 = "passage_1".to_string();
        let passage_id_2 = "passage_2".to_string();
        let passage_id_3 = "passage_3".to_string();
        let mut dungeon = Dungeon::new();
        connect_room(
            &mut dungeon,
            passage_id_1.clone(),
            Position {
                x: 10,
                y: 10
            }, Position {
                x: 10,
                y: 9
            }
        );

        connect_room(
            &mut dungeon,
            passage_id_2.clone(),
            Position {
                x: 10,
                y: 10
            }, Position {
                x: 10,
                y: 11
            }
        );

        connect_room(
            &mut dungeon,
            passage_id_3.clone(),
            Position {
                x: 10,
                y: 10
            }, Position {
                x: 9,
                y: 10
            }
        );

        let current_room = Position { x: 10, y: 10 };
        let expected: Vec<String> = vec![
            "You see passages to the north, south, and west.".to_string()
        ];

        let result = super::look(&dungeon, current_room);

        assert_eq!(expected, result);
    }

    #[test]
    fn look_one_door() {
        let passage_id_1 = "passage_1".to_string();
        let mut dungeon = Dungeon::new();
        connect_door(
            &mut dungeon,
            passage_id_1.clone(),
            DoorState::Closed,
            Position {
                x: 10,
                y: 10
            }, Position {
                x: 10,
                y: 9
            }
        );

        let current_room = Position { x: 10, y: 10 };
        let expected: Vec<String> = vec![
            "To the north there is a door.".to_string()
        ];

        let result = super::look(&dungeon, current_room);

        assert_eq!(expected, result);
    }

    #[test]
    fn look_two_doors() {
        let passage_id_1 = "passage_1".to_string();
        let passage_id_2 = "passage_2".to_string();
        let mut dungeon = Dungeon::new();
        connect_door(
            &mut dungeon,
            passage_id_1.clone(),
            DoorState::Closed,
            Position {
                x: 10,
                y: 10
            }, Position {
                x: 10,
                y: 9
            }
        );

        connect_door(
            &mut dungeon,
            passage_id_2.clone(),
            DoorState::Closed,
            Position {
                x: 10,
                y: 10
            }, Position {
                x: 10,
                y: 11
            }
        );
        let current_room = Position { x: 10, y: 10 };
        let expected: Vec<String> = vec![
            "You see doors to the north, and south.".to_string()
        ];

        let result = super::look(&dungeon, current_room);

        assert_eq!(expected, result);
    }

    #[test]
    fn process_command_exits() {
        let mut dungeon = Dungeon::new();
        let mut player = Player::new(
            Position { x: 10, y: 0 }
        );
        let command = Command::Exit;

        let result = process_command(&mut dungeon, &mut player, command);

        assert!(!result.running);
        assert!(result.output.is_empty());
    }

    #[test]
    fn process_command_go_north() {
        let passage_id_1 = "passage_1".to_string();
        let mut dungeon = Dungeon::new();
        connect_room(
            &mut dungeon,
            passage_id_1.clone(),
            Position {
                x: 10,
                y: 10
            }, Position {
                x: 10,
                y: 9
            }
        );

        let mut player = Player::new(
            Position { x: 10, y: 10 }
        );

        let command = Command::Go { direction: Direction::North };

        let result = process_command(&mut dungeon, &mut player, command);

        assert!(result.running);
        assert!(!result.output.is_empty());
        assert_eq!(player.get_position().x, 10);
        assert_eq!(player.get_position().y, 9);
    }


    #[test]
    fn process_command_blank() {
        let mut dungeon = Dungeon::new();
        let mut player = Player::new(
            Position { x: 10, y: 10 }
        );
        let command = Command::Blank;

        let result = process_command(&mut dungeon, &mut player, command);

        assert!(result.running);
        assert!(result.output.is_empty());
        assert_eq!(player.get_position().x, 10);
        assert_eq!(player.get_position().y, 10);
    }

    #[test]
    fn process_command_unknown() {
        let mut dungeon = Dungeon::new();
        let mut player = Player::new(
            Position { x: 10, y: 10 }
        );
        let command = Command::Unknown;

        let result = process_command(&mut dungeon, &mut player, command);

        assert!(result.running);
        assert_eq!(result.output.len(), 1);
        assert_eq!(result.output[0], "I don't understand what you want to do!");
    }

    #[test]
    fn process_command_incomplete() {
        let mut dungeon = Dungeon::new();
        let mut player = Player::new(
            Position { x: 10, y: 10 }
        );

        let command = Command::Incomplete { reason: "Hello World!" };

        let result = process_command(&mut dungeon, &mut player, command);

        assert!(result.running);
        assert_eq!(result.output.len(), 1);
        assert_eq!(result.output[0], "Hello World!");
    }
}