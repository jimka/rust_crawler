pub struct Inventory {
    items: Vec<Box<dyn Item>>
}

impl Inventory {

    pub fn new() -> Self {
        Inventory {
            items: vec![]
        }
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn get_item(&self, item_id: &str) -> Option<&dyn Item> {
        self.items
            .iter()
            .find(|x| -> bool { x.matches(item_id) })
            .map(|x| x.as_ref())
    }

    pub fn get_items(&self) -> &Vec<Box<dyn Item>> {
        &self.items
    }

    pub fn take_item(&mut self, item_id: &str) -> Option<Box<dyn Item>> {
        let opt = self.items
            .iter()
            .enumerate()
            .find(|x| -> bool { x.1.matches(item_id) });

        let (idx, _) = opt?;

        let item = self.items.remove(idx);

        Some(item)
    }
    
    pub fn put_item(&mut self, item: Box<dyn Item>) {
        self.items.push(item);
    }
}

#[derive(Debug, PartialEq)]
pub enum ItemType {
    Key,
}

pub trait Item {

    fn get_type(&self) -> ItemType;

    fn get_description(&self) -> &str;

    fn matches(&self, id: &str) -> bool;

    fn as_key(&self) -> Option<&Key>;
}

pub struct Key {

    id         : String,
    description: String,
    door       : String,
}

impl Key {

    pub fn get_door(&self) -> &str {
        &self.door
    }
}

impl Item for Key {

    fn get_type(&self) -> ItemType {
        ItemType::Key
    }

    fn get_description(&self) -> &str {
        self.description.as_str()
    }

    fn matches(&self, id: &str) -> bool {
        self.id == id
    }
    
    fn as_key(&self) -> Option<&Key> {
        Some(self)
    }

    
}

#[cfg(test)]
mod test {
    use std::assert_eq;

use super::*;

    #[test]
    fn inventory_add() {
        let mut inventory = Inventory {
            items: vec![]
        };

        let k = Key::new("666".to_string());

        inventory.add(k);

        assert_eq!(inventory.items.len(), 1);
    }

    #[test]
    fn key_new() {
        let k = Key::new("666".to_string());

        assert!(k.matches("key"));
        assert_eq!(k.get_description(), "a normal key");
        assert_eq!(k.get_door(), "666");
    }

    #[test]
    fn key_get_type() {
        let expected = ItemType::Key;
        let k = Key::new("666".to_string());

        let result = k.get_type();

        assert_eq!(result, expected);
    }
}