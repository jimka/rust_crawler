use crate::item::Item;

pub struct Inventory {
    items: Vec<Item>
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

    pub fn get_item(&self, item_id: &str) -> Option<&Item> {
        self.items
            .iter()
            .find(|x| -> bool { x.matches(item_id) })
    }

    pub fn take_item(&mut self, item_id: &str) -> Option<Item> {
        let opt = self.items
            .iter()
            .enumerate()
            .find(|x| -> bool { x.1.matches(item_id) });

        let (idx, _) = opt?;

        let item = self.items.remove(idx);

        Some(item)
    }

    pub fn put_item(&mut self, item: Item) {
        self.items.push(item);
    }

    pub(crate) fn iter(&self) -> std::slice::Iter<'_, Item> {
        self.items.iter()
    }
}

impl<'a> IntoIterator for &'a Inventory {
    type Item = &'a Item;
    type IntoIter = std::slice::Iter<'a, Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::glyph::Glyph;

    #[test]
    fn inventory_add() {
        let mut inventory = Inventory {
            items: vec![]
        };

        let k = Item::new_key("666", Glyph::random());

        inventory.put_item(k);

        assert_eq!(inventory.items.len(), 1);
    }
}