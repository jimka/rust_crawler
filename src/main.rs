// ---- Module registry ----
mod dungeon;
mod util;
mod command;
mod player;
mod inventory;
// ---- Module registry ----

use std::io::{BufRead, BufReader, Write};

use dungeon::{Dungeon,Passage,Room};
use util::{Direction,Position};
use command::{Command,look,CommandResult,process_command};
use player::Player;

use crate::{dungeon::DoorState, inventory::Key};

fn build_dungeon() -> Dungeon {
    let mut dungeon = Dungeon::new();

    dungeon.add_passage("r1r2".to_string(), Passage::Room {
        room_1: Position { x: 0, y: 0 },
        room_2: Position { x: 0, y: 1 },
    }).add_passage(
        "r2r3".to_string(),
        Passage::Door {
            room_1: Position { x: 0, y: 1 },
            room_2: Position { x: 1, y: 1 },
            state: DoorState::Locked
        }
    ).add_passage(
        "r3r4".to_string(),
        Passage::Door {
            room_1: Position { x: 1, y: 1 },
            room_2: Position { x: 2, y: 1 },
            state: DoorState::Closed
        }
    ).add_passage(
        "r2r5".to_string(),
        Passage::Door {
            room_1: Position { x: 0, y: 1 },
            room_2: Position { x: 0, y: 2 },
            state: DoorState::Locked
        }
    );

    let mut r1 = Room::new();
    r1.add_passage(
        Direction::South,
        "r1r2".to_string()
    );

    r1.get_inventory_mut().add(
        Key::new("r2r3".to_string())
    );

    let mut r2 = Room::new();
    r2.add_passage(
        Direction::North,
        "r1r2".to_string(),
    ).add_passage(
        Direction::East,
        "r2r3".to_string(),
    ).add_passage(
        Direction::South,
        "r2r5".to_string(),
    );

    let mut r3 = Room::new();
    r3.add_passage(
        Direction::West,
        "r2r3".to_string(),
    ).add_passage(
        Direction::East,
        "r3r4".to_string(),
    );

    let mut r4 = Room::new();
    r4.add_passage(
        Direction::West,
        "r3r4".to_string(),
    );

    let mut r5 = Room::new();
    r5.add_passage(
        Direction::North,
        "r2r5".to_string(),
    );

    dungeon.add_room(Position { x: 0, y: 0 }, r1).unwrap()
           .add_room(Position { x: 0, y: 1 }, r2).unwrap()
           .add_room(Position { x: 1, y: 1 }, r3).unwrap()
           .add_room(Position { x: 2, y: 1 }, r4).unwrap()
           .add_room(Position { x: 0, y: 2 }, r5).unwrap();

    dungeon
}

fn main() {
    game_loop();
}

fn game_loop() {
    let mut dungeon = build_dungeon();
    let mut player = Player::new(Position { x: 0, y: 0 });

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
