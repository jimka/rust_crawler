use std::{collections::{HashMap, HashSet}};

use rand::{Rng, RngExt};

use crate::{
    dungeon::{
        self,
        DoorState,
        Dungeon,
        Passage,
        Room
    }, glyph::Glyph, item::Item, util::{
        Direction,
        Position,
        Size
    }
};


pub fn generate_dungeon(size: Size, room_count: usize) -> (Dungeon, Position) {
    let mut rng = rand::rng();

    generate_dungeon_with_rng(&mut rng, size, room_count)
}

pub fn generate_dungeon_with_rng<T>(rng: &mut T, size: Size, room_count: usize) -> (Dungeon, Position) 
    where T: Rng {
    if room_count > size.width * size.height {
        panic!("Can't fit that many rooms in a dungeon of that size.")
    }

    let mut room_count = room_count - 1;
    let mut map = HashMap::new();
    let mut passages = HashMap::new();
    let mut start_position = Position::new(
        rng.random_range(0..size.width),
        rng.random_range(0..size.height)
    );

    let start_room = Room::new();
    let mut used_glyphs = vec![];

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

        let branch_direction_idx = rng.random_range(0..available_branches.len());
        let branch_direction = *available_branches.get(branch_direction_idx).unwrap();
        let return_direction = branch_direction.opposite();
        let branch_to_room_position = branch_from_room_position.step(branch_direction);

        if branch_from_room_position == branch_to_room_position {
            continue;
        }

        let passage_id: String = format!("passage_{}", passages.len() + 1);

        match rng.random_range(0..100) {
            0..30 => {
                let should_be_locked = rng.random_range(0..100) < 50;
                let mut glyph = Glyph::random();

                while used_glyphs.contains(&glyph) {
                    glyph = Glyph::random();
                }

                used_glyphs.push(glyph);

                passages.insert(
                    passage_id.clone(),
                    Passage::Door {
                        state: if should_be_locked { dungeon::DoorState::Locked } else { dungeon::DoorState::Closed },
                        glyph,
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

    let min_x = map.keys().map(|x| x.x).min().unwrap();

    start_position.x -= min_x;

    let mut dungeon = Dungeon::new_with_size(size);
    let mut rooms = HashMap::new();

    rooms.extend(
        map
            .into_iter()
            .map(|(position, room)| {
                (Position::new(position.x - min_x, position.y), room)
            })
    );

    for passage in passages.values_mut() {
        if let Passage::Room { room_1, room_2 } = passage {
            room_1.x -= min_x;
            room_2.x -= min_x;
        } else if let Passage::Door { room_1, room_2, .. } = passage {
            room_1.x -= min_x;
            room_2.x -= min_x;
        };
    }

    place_keys(start_position, &mut rooms, &passages);

    for (position, room) in rooms {
        let _ = dungeon.add_room(position, room);
    }

    for (passage_id, passage) in passages {
             let _ = dungeon.add_passage(passage_id, passage);
    }

    (dungeon, start_position)
}

fn place_keys(
    start_position: Position,
    rooms: &mut HashMap<Position, Room>,
    passages: &HashMap<String, Passage>
) {
    let mut rng = rand::rng();

    let partitions = partition_rooms(rooms, passages);
    let mut partitions_with_doors = group_doors_with_partitions(passages, partitions);

    let room_count_1: usize = rooms.len();
    let room_count_2: usize = partitions_with_doors.iter().map(|(partition, _)| partition.len()).sum();

    if room_count_1 != room_count_2 {
        panic!("Room count does not match; {room_count_1} != {room_count_2}");
    }

    let start_idx = partitions_with_doors
        .iter()
        .position(|(partition, _)| partition.contains(&start_position))
        .unwrap();

    let (mut partition, mut door_ids) = partitions_with_doors.remove(start_idx);

    while let Some(door_id) = door_ids.pop() {
        let passage = passages.get(&door_id).expect("Where's my passage?!");

        let Passage::Door {
            state: DoorState::Locked,
            glyph,
            room_1,
            room_2,
        } = passage else {
            unreachable!("Should only be locked doors here!");
        };

        let destination_room = if partition.contains(room_1) { room_2 } else { room_1 };

        if partition.contains(destination_room) {
            // This means the destination room is in the same partition as the origin room,
            // ie. you can reach both sides of the door from the same partition. We still
            // need to add a key for this door, in this partition.

            // Get random room in the partition where we can place the key.
            let room_position = partition.get(rng.random_range(0..partition.len())).expect("What?!");

            let room = rooms.get_mut(room_position).expect("Where's my room?!");

            let room_inventory = room.get_inventory_mut();
            room_inventory.put_item(Item::new_key(&door_id, *glyph));
        } else {
            let destination_idx = partitions_with_doors
                .iter()
                .position(|(destination_partition, _)| destination_partition.contains(destination_room))
                .expect("Destination room must be in some partition.");

            let (mut destination_partition, mut destination_door_ids) = partitions_with_doors.swap_remove(destination_idx);

            let room_count = partition.len();

            // Get random room in the partition where we can place the key.
            let room_position = partition.get_mut(rng.random_range(0..room_count)).expect("What?!");

            let room = rooms.get_mut(room_position).expect("Where's my room?!");

            let room_inventory = room.get_inventory_mut();
            room_inventory.put_item(Item::new_key(&door_id, *glyph));

            destination_door_ids.retain(|x| *x != door_id);

            partition.append(&mut destination_partition);
            door_ids.append(&mut destination_door_ids);
        }
    }
}

fn group_doors_with_partitions(passages: &HashMap<String, Passage>, partitions: Vec<Vec<Position>>) -> Vec<(Vec<Position>, Vec<String>)> {
    let mut partitions_with_passages: Vec<(Vec<Position>, Vec<String>)> = vec![];

    for partition in partitions {
        let passages = fetch_doors(&partition, passages);

        partitions_with_passages.push((partition, passages));
    }

    partitions_with_passages
}

fn fetch_doors(partition: &[Position], passages: &HashMap<String, Passage>) -> Vec<String> {
    let mut output = vec![];

    for (passage_id, passage) in passages {
        let (room_1, room_2) = match passage {
            Passage::Door { state: DoorState::Locked, room_1, room_2, .. } => (room_1, room_2),
            _ => continue,
        };

        if partition.contains(room_1) || partition.contains(room_2) {
            output.push(passage_id.clone());
        }
    }

    output
}

fn partition_rooms(rooms: &mut HashMap<Position, Room>, passages: &HashMap<String, Passage>) -> Vec<Vec<Position>> {
    let mut rng = rand::rng();
    let mut partitions: Vec<Vec<Position>> = Vec::new();

    loop {
        let partitioned_rooms: Vec<Position> = partitions
            .iter()
            .flatten()
            .copied()
            .collect();

        let available_rooms: Vec<Position> = rooms
            .keys()
            .filter(|x| !partitioned_rooms.contains(x))
            .copied()
            .collect();

        if available_rooms.is_empty() {
            break;
        }

        let initial_room = available_rooms
            .get(rng.random_range(0..available_rooms.len()))
            .unwrap();

        let mut partition: HashSet<Position> = HashSet::new();
        let mut stack: Vec<Position> = vec![*initial_room];

        while let Some(pos) = stack.pop() {
            partition.insert(pos);

            let room = rooms.get_mut(&pos).unwrap();

            let accessible_rooms: Vec<Position> = room
                .get_passages()
                .values()
                .filter(|&passage_id| {
                        let p = passages.get(passage_id).unwrap();

                        !matches!(p, Passage::Door { state: DoorState::Locked, .. })
                    })
                .map(|passage_id| {
                        let (room_1, room_2) = match passages.get(passage_id).unwrap() {
                            Passage::Room { room_1, room_2 } => (room_1, room_2),
                            Passage::Door { room_1, room_2, .. } => (room_1, room_2),
                        };

                        if pos == *room_1 { *room_2 } else { *room_1 }
                    }
                )
                .collect();

            let mut new_positions: Vec<Position> = accessible_rooms
                .iter()
                .filter(|x| !partition.contains(x))
                .filter(|x| !partitioned_rooms.contains(x))
                .copied()
                .collect();

            stack.append(&mut new_positions);
        }

        let as_vec: Vec<Position> = partition.iter().copied().collect();
        partitions.push(as_vec);
    }

    partitions
}

#[cfg(test)]
mod test {
    use rand::{SeedableRng, rngs::StdRng};

use super::*;
    #[test]
    fn dungeon_generation_test() {
        let mut rng = StdRng::seed_from_u64(42);
        let expected_room_count = 32;

        for _ in 0..500 {
            let (dungeon, start_position) = generate_dungeon_with_rng(
                &mut rng,
                Size::new(16, 16), expected_room_count
            );

            let visited_rooms = evaluate_solvability(&dungeon, start_position);

            assert_eq!(dungeon.get_room_count(), visited_rooms.len());
            assert_eq!(dungeon.get_room_count(), expected_room_count);
        }
    }

    fn evaluate_solvability(dungeon: &Dungeon, start_position: Position) -> Vec<Position> {
        let mut accessible_rooms = vec![];
        let mut visited_rooms = vec![];
        let mut locked_doors  = vec![];
        let mut keys = vec![];

        accessible_rooms.push(start_position);

        while let Some(position) = accessible_rooms.pop() {
            let room = dungeon.get_room(position).expect("Unable to find room");
            take_keys(&mut keys, room);
            let mut passages = get_room_passages(dungeon, room);

            while let Some(passage) = passages.pop() {
                match passage {
                    Passage::Door { state: DoorState::Locked, .. } => check_door_and_push(&mut locked_doors, passage),
                    Passage::Door { room_1, room_2, .. } => check_room_and_push(&mut accessible_rooms, &visited_rooms, room_1, room_2, position),
                    Passage::Room { room_1, room_2 } => check_room_and_push(&mut accessible_rooms, &visited_rooms, room_1, room_2, position),
                }
            }

            visited_rooms.push(position);

            unlock_doors(
                &mut accessible_rooms,
                &visited_rooms,
                &mut locked_doors,
                &mut keys
            );
        }
        visited_rooms
    }

    fn check_door_and_push<'a>(
        locked_doors: &mut Vec<&'a Passage>,
        passage     : &'a Passage
    ) {
        if !locked_doors.contains(&passage) {
            locked_doors.push(passage)
        }
    }

    fn unlock_doors<'a>(
        accessible_rooms: &'a mut Vec<Position>,
        visited_rooms   : &[Position],
        locked_doors    : &'a mut Vec<&Passage>,
        keys            : &'a mut Vec<Item>
    ) {
        let glyphs: Vec<Glyph> = keys
            .iter()
            .filter_map(|x| if let Item::Key { glyph, .. } = x { Some(*glyph) } else { None })
            .collect();

        let matched_pairs: Vec<(Glyph, Passage)> = locked_doors
            .iter()
            .filter_map(|x| if let Passage::Door { glyph, .. } = x && glyphs.contains(glyph) { Some((*glyph, **x)) } else { None })
            .collect();

        for (g, passage) in matched_pairs {
            let pos = keys
                .iter()
                .position(|x| if let Item::Key { glyph, .. } = x { *glyph == g } else { false })
                .expect("Where's the key?!");

            keys.swap_remove(pos);

            let pos = locked_doors
                .iter()
                .position(|x| *x == &passage).expect("Where's my passage?!");

            locked_doors.swap_remove(pos);

            if let Passage::Door { room_1, room_2, .. } = passage {
                if !visited_rooms.contains(&room_1) && !accessible_rooms.contains(&room_1) {
                    accessible_rooms.push(room_1);
                }

                if !visited_rooms.contains(&room_2) && !accessible_rooms.contains(&room_2) {
                    accessible_rooms.push(room_2);
                }
            }
        }
    }

    fn take_keys(keys: &mut Vec<Item>, room: &Room) {
        let inventory = room.get_inventory();
        let mut room_keys: Vec<Item> = inventory
            .iter()
            .filter_map(|x| {
                if let Item::Key { .. } = x { Some(x) } else { None }
            })
            .cloned()
            .collect();

        if !room_keys.is_empty() {
            keys.append(&mut room_keys);
        }
    }

    fn check_room_and_push(
        accessible_rooms: &mut Vec<Position>,
        visited_rooms   : &[Position],
        room_1          : &Position,
        room_2          : &Position,
        position        : Position
    ) {
        let destination_room = if position == *room_1 { *room_2 } else { *room_1 };

        if !visited_rooms.contains(&destination_room) && !accessible_rooms.contains(&destination_room){
            accessible_rooms.push(destination_room);
        }
    }

    fn get_room_passages<'a>(dungeon: &'a Dungeon, room: &'a Room) -> Vec<&'a Passage> {
        room
            .get_passages()
            .values()
            .map(|passage_id| dungeon.get_passage(passage_id).unwrap())
            .collect()
    }
}