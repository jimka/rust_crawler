use crate::glyph::Glyph;

#[derive(Debug, PartialEq, Clone)]
pub enum Item {
    Key { id: String, description: String, door: String, glyph: Glyph },
}

impl Item {

    pub fn new_key(door: &str, glyph: Glyph) -> Item {
        let glyph_string = glyph.to_string().to_lowercase();

        let is_an = glyph_string.starts_with(['a', 'e', 'i', 'o', 'u'])
                              && ![Glyph::Unicorn, Glyph::Ouroboros].contains(&glyph);

        Item::Key {
            id: glyph_string.clone() + " key",
            description: format!("A key marked with {} {} glyph.", if is_an { "an" } else { "a" }, glyph_string),
            door: door.to_string(),
            glyph
        }
    }

    pub fn matches(&self, text: &str) -> bool {
        match self {
            Item::Key { id, .. } => id.to_lowercase() == text.to_lowercase(),
        }
    }
    
    pub(crate) fn get_description(&self) -> &str {
        match self {
            Item::Key { description, .. } => description,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn key_new_with_argent() {
        let glyph = Glyph::Argent;
        let k = Item::new_key("666", glyph);

        let Item::Key { id, description, door, .. }  = &k;

        assert!(k.matches("Argent key"));
        assert_eq!(id, "argent key");
        assert_eq!(description, "A key marked with an argent glyph.");
        assert_eq!(door, "666");
    }

    #[test]
    fn key_new_with_unicorn() {
        let glyph = Glyph::Unicorn;
        let k = Item::new_key("666", glyph);

        let Item::Key { id, description, door, .. }  = &k;

        assert!(k.matches("Unicorn key"));
        assert_eq!(id, "unicorn key");
        assert_eq!(description, "A key marked with a unicorn glyph.");
        assert_eq!(door, "666");
    }

    #[test]
    fn key_new_with_basilisk() {
        let glyph = Glyph::Basilisk;
        let k = Item::new_key("666", glyph);

        let Item::Key { id, description, door, .. }  = &k;

        assert!(k.matches("Basilisk key"));
        assert_eq!(id, "basilisk key");
        assert_eq!(description, "A key marked with a basilisk glyph.");
        assert_eq!(door, "666");
    }
}
