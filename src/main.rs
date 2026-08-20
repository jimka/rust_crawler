// ---- Module registry ----
mod error;
mod dungeon;
mod util;
mod command;
mod player;
mod item;
mod inventory;
mod dungeon_generator;
mod glyph;
// ---- Module registry ----

use std::io::{BufRead, BufReader, Write};

use command::{Command,look,CommandResult,process_command};
use player::Player;

use dungeon::Dungeon;
use util::Size;

use crate::{dungeon_generator::generate_dungeon, error::GameError};

fn main() {
    game_loop();
}

fn game_loop() {
    let (mut dungeon, start_position) = generate_dungeon(Size::new(16, 16), 16);
    let mut player = Player::new(start_position);

    println!("You enter the dungeon!");
    println!();

    look(&dungeon, player.get_position(), None)
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
) -> Result<CommandResult, GameError>
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

fn query_input<R, W>(reader: &mut R, writer: &mut W) -> Result<String, GameError>
where
    R: BufRead,
    W: Write
{
    let _ = writeln!(writer, "What do you want to do?");
    let _ = write!(writer, "> ");

    writer.flush()?;

    let mut input = String::new();
    let _ = reader.read_line(&mut input)?;

    Ok(input)
}
