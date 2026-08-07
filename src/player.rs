use crate::util::Position;
use crate::inventory::Inventory;

pub struct Player {
    position: Position,
    inventory: Inventory
}

impl Player {

    pub fn new(position: Position) -> Self {
        Player {
            position,
            inventory: Inventory::new()
        }
    }

    pub fn get_position(&self) -> Position {
        self.position
    }

    pub(crate) fn set_position(&mut self, new_position: Position) {
        self.position = new_position;
    }

    pub fn get_inventory(&self) -> &Inventory {
        &self.inventory
    }

    pub fn get_inventory_mut(&mut self) -> &mut Inventory {
        &mut self.inventory
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_new() {
        let p = Player::new(Position { x: 10, y: 10 });

        assert_eq!(p.get_position(), Position { x: 10, y: 10 });
        assert!(p.get_inventory().is_empty());
    }

    #[test]
    fn player_get_position() {
        let p = Player::new(Position { x: 10, y: 10 });

        assert!(matches!(
            p.get_position(),
            Position { x: 10, y: 10 })
        );
    }

    #[test]
    fn player_set_position() {
        let mut p = Player::new(Position { x: 0, y: 0 });

        p.set_position(Position { x: 10, y: 10 });

        assert!(matches!(
            p.get_position(),
            Position { x: 10, y: 10 })
        );
    }

    #[test]
    fn player_get_inventory() {
        let p = Player::new(Position { x: 0, y: 0 });

        let inventory = p.get_inventory();

        assert!(inventory.is_empty());
    }
}