// ---- Module registry ----
mod dungeon;
mod util;
mod command;
mod player;
mod inventory;
// ---- Module registry ----

use std::{collections::HashMap, io::{BufRead, BufReader, Write}, unreachable};


use dungeon::{Dungeon,Passage,Room};
use rand::RngExt;
use util::{Direction,Position};
use command::{Command,look,CommandResult,process_command};
use player::Player;

use crate::util::Size;

fn generate_dungeon(size: Size, room_count: usize) -> (Dungeon, Position) {
    if room_count > size.width * size.height {
        panic!("Can't fit that many rooms in a dungeon of that size.")
    }

    let mut rng = rand::rng();

    let mut room_count = room_count - 1;
    let mut map = HashMap::new();
    let mut passages = HashMap::new();
    let mut start_position = Position::random(size.width,size.height);

    let start_room = Room::new();

    map.insert(start_position, start_room);

    while room_count != 0 {
        let positions: Vec<Position> = map
            .keys()
            .copied()
            .collect();

        let branch_from_room = rng.random_range(0..positions.len());
        let branch_from_room_position = *positions.get(branch_from_room).unwrap();

        let src_room = map.get_mut(&branch_from_room_position).unwrap();

        let branches = src_room.get_passages();
        let current_branch_count = branches.len();

        if current_branch_count == 4 {
            // Unable to branch more from this room.
            continue;
        }

        let chance_of_new_branch: usize = match current_branch_count {
            0 => 100,
            1 => 20,
            2 => 10,
            3 => 5,
            _ => unreachable!("Shouldn't be possible")
        };

        let should_add_branch = rng.random_range(0..100) < chance_of_new_branch;
        if !should_add_branch {
            continue;
        }

        let available_branches: Vec<Direction> = Direction::all()
            .iter()
            .filter(|&x| !branches.contains_key(x))
            .filter(|&x| {
                match *x {
                    Direction::North => branch_from_room_position.y != 0,
                    Direction::South => branch_from_room_position.y != size.height - 1,
                    Direction::West  => branch_from_room_position.x != 0,
                    Direction::East  => branch_from_room_position.x != size.width - 1,
                }
            })
            .copied()
            .collect();

        if available_branches.is_empty() {
            continue;
        }

        let branch_direction_idx = rng.random_range(0..available_branches.iter().len());
        let branch_direction = *available_branches.get(branch_direction_idx).unwrap();
        let return_direction = branch_direction.opposite();
        let branch_to_room_position = branch_from_room_position.step(branch_direction);

        if branch_from_room_position == branch_to_room_position {
            continue;
        }

        let passage_id: String = format!("passage_{}", passages.len() + 1);

        match rng.random_range(0..100) {
            0..25 => {
                passages.insert(
                    passage_id.clone(),
                    Passage::Door {
                        state: dungeon::DoorState::Closed,
                        room_1: branch_from_room_position,
                        room_2: branch_to_room_position
                    }
                );
            },
            _ => {
                passages.insert(
                    passage_id.clone(),
                    Passage::Room {
                        room_1: branch_from_room_position,
                        room_2: branch_to_room_position
                    }
                );
            }
        }

        src_room.add_passage(branch_direction, passage_id.clone());

        let mut preexisting_room = false;

        let dst_room = if let Some(room) = map.get_mut(&branch_to_room_position) {
            preexisting_room = true;

            room
        } else {
            map.insert(branch_to_room_position, Room::new());

            let Some(room) = map.get_mut(&branch_to_room_position) else {
                unreachable!("Oh my, why?!")
            };

            room
        };

        dst_room.add_passage(return_direction, passage_id.clone());

        if !preexisting_room {
            room_count -= 1;
        }
    }

    let min_x = map.keys().map(|x| x.x).reduce(|acc, x| if acc < x { acc } else { x }).unwrap();

    let mut dungeon = Dungeon::new_with_size(size);

    for (position, room) in map {
        let shifted_position = Position::new(position.x - min_x, position.y);

        let _ = dungeon.add_room(shifted_position, room);
    }

    for (_, passage) in passages.iter_mut() {
        if let Passage::Room { room_1, room_2 } = passage {
            room_1.x -= min_x;
            room_2.x -= min_x;
        } else if let Passage::Door { room_1, room_2, .. } = passage {
            room_1.x -= min_x;
            room_2.x -= min_x;
        };
    }

    println!("Passages:");
    for (passage_id, passage) in passages {
             let _ = dungeon.add_passage(passage_id, passage);
    }

    start_position.x -= min_x;

    (dungeon, start_position)
}

fn main() {
    game_loop();
}

fn game_loop() {
    let (mut dungeon, start_position) = generate_dungeon(Size::new(16, 16), 16);
    let mut player = Player::new(start_position);

    println!("You enter the dungeon!");
    println!();

    look(&dungeon, player.get_position())
        .iter()
        .for_each(|x| println!("{x}"));

    let mut reader = BufReader::new(std::io::stdin());
    let mut writer = std::io::stdout();

    loop {
        let result = step(
            &mut reader,
            &mut writer,
            &mut dungeon,
            &mut player
        );

        match result {
            Ok(CommandResult { running: false, .. } ) => break,
            Ok(..) => continue,
            Err(s) => {
                println!("{s}");

                break
            }
        };
    }

    println!("You exit the dungeon!");
}

fn step<R, W>(
    reader : &mut R,
    writer : &mut W,
    dungeon: &mut Dungeon,
    player : &mut Player
) -> Result<CommandResult, String>
where
    R: BufRead,
    W: Write
{
    let _ = writeln!(writer);

    let command: Command = query_input(reader, writer)?.parse::<Command>()?;

    let _ = writeln!(writer);

    let result = process_command(
        dungeon,
        player,
        command
    );

    for s in &result.output {
        let _ = writeln!(writer, "    {s}");
    }

    Ok(result)
}

fn query_input<R, W>(reader: &mut R, writer: &mut W) -> Result<String, String>
where
    R: BufRead,
    W: Write
{
    let _ = writeln!(writer, "What do you want to do?");
    let _ = write!(writer, "> ");

    let result = writer.flush();
    if result.is_err() {
        return Err("Unable to flush output!".to_string());
    }

    let mut input = String::new();
    let result = reader.read_line(&mut input);
    if result.is_err() {
        return Err("Unable to read input!".to_string());
    }

    Ok(input)
}
