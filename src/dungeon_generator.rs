use std::{collections::{HashMap, HashSet}, unreachable};

use rand::RngExt;

use crate::{
    dungeon::{
        self,
        DoorState,
        Dungeon,
        Passage,
        Room
    },
    inventory::Key,
    util::{
        Direction,
        Position,
        Size
    }
};


pub fn generate_dungeon(size: Size, room_count: usize) -> (Dungeon, Position) {
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
            0..30 => {
                let should_be_locked = rng.random_range(0..100) < 50;
                passages.insert(
                    passage_id.clone(),
                    Passage::Door {
                        state: if should_be_locked { dungeon::DoorState::Locked } else { dungeon::DoorState::Closed },
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

    start_position.x -= min_x;

    let mut dungeon = Dungeon::new_with_size(size);

    let mut rooms = HashMap::new();

    for (position, room) in map {
        let shifted_position = Position::new(position.x - min_x, position.y);

        rooms.insert(shifted_position, room);
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
            room_1,
            room_2
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
            room_inventory.put_item(Key::new(&door_id));
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
            room_inventory.put_item(Key::new(&door_id));

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
            Passage::Door { state: DoorState::Locked, room_1, room_2 } => (room_1, room_2),
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
