use std::{error::Error, fmt::Display};

#[derive(Debug, PartialEq)]
pub enum AddRoomError {
    OutOfBounds,
    Occupied
}

impl Display for AddRoomError {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AddRoomError::OutOfBounds => write!(f, "That position is outside of the dungeon."),
            AddRoomError::Occupied => write!(f, "That position is already occupied."),
        }
    }
}

#[derive(Debug)]
pub enum TakeDirectionError {
    Incomplete,
    Invalid
}

impl Display for TakeDirectionError {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TakeDirectionError::Incomplete => write!(f, "No direction given."),
            TakeDirectionError::Invalid => write!(f, "Not a valid direction."),
        }
    }
}

#[derive(Debug)]
pub enum GameError {
    AddRoom(AddRoomError),
    Direction(TakeDirectionError),
    Io(std::io::Error),
}

impl Display for GameError {

    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameError::AddRoom(error) => error.fmt(f),
            GameError::Direction(error) => error.fmt(f),
            GameError::Io(error) => error.fmt(f),
        }
    }
}

impl From<std::convert::Infallible> for GameError {

    fn from(never: std::convert::Infallible) -> Self {
        match never {}
    }
}

impl From<TakeDirectionError> for GameError {

    fn from(value: TakeDirectionError) -> Self {
        GameError::Direction(value)
    }
}

impl From<AddRoomError> for GameError {

    fn from(value: AddRoomError) -> Self {
        GameError::AddRoom(value)
    }
}

impl From<std::io::Error> for GameError {

    fn from(value: std::io::Error) -> Self {
        GameError::Io(value)
    }
}

impl Error for GameError {}