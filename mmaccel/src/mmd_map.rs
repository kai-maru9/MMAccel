use serde_json::Value;
use std::fs::File;
use std::io::BufReader;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ComboDir {
    Prev,
    Next,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ItemKind {
    Key(u32),
    Button(u32),
    Edit(u32),
    Combo(ComboDir, u32),
    Menu(u32, u32),
    Fold(u32, u32),
    KillFocus,
    FoldAll,
    UnfoldAll,
}

impl ItemKind {
    fn new(a: &[serde_json::Value]) -> Option<Self> {
        let kind = match a[1].as_str()? {
            "key" if a.len() == 3 => Self::Key(u32::from_str_radix(a[2].as_str()?, 16).ok()?),
            "button" if a.len() == 3 => Self::Button(u32::from_str_radix(a[2].as_str()?, 16).ok()?),
            "edit" if a.len() == 3 => Self::Edit(u32::from_str_radix(a[2].as_str()?, 16).ok()?),
            "combo_prev" if a.len() == 3 => Self::Combo(
                ComboDir::Prev,
                u32::from_str_radix(a[2].as_str()?, 16).ok()?,
            ),
            "combo_next" if a.len() == 3 => Self::Combo(
                ComboDir::Next,
                u32::from_str_radix(a[2].as_str()?, 16).ok()?,
            ),
            "menu" if a.len() == 4 => Self::Menu(a[2].as_u64()? as _, a[3].as_u64()? as _),
            "fold" if a.len() == 4 => Self::Fold(
                u32::from_str_radix(a[2].as_str()?, 16).ok()?,
                u32::from_str_radix(a[3].as_str()?, 16).ok()?,
            ),
            "kill_focus" => Self::KillFocus,
            "fold_all" => Self::FoldAll,
            "unfold_all" => Self::UnfoldAll,
            _ => return None,
        };
        Some(kind)
    }

    #[inline]
    pub fn as_key(&self) -> Option<u32> {
        match self {
            Self::Key(v) => Some(*v),
            _ => None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Item {
    pub name: String,
    pub kind: ItemKind,
}

impl Item {
    fn new(a: &[serde_json::Value]) -> Option<Self> {
        if a.len() < 2 {
            return None;
        }
        Some(Self {
            name: a[0].as_str()?.to_string(),
            kind: ItemKind::new(a)?,
        })
    }
}

#[derive(Debug)]
pub struct MmdMap(Vec<(String, Item)>);

impl MmdMap {
    pub fn from_file(path: impl AsRef<std::path::Path>) -> std::io::Result<Self> {
        fn items(m: &mut Vec<(String, Item)>, v: &Value) -> Option<()> {
            match v {
                Value::Object(obj) => {
                    for (key, value) in obj.iter() {
                        if value.is_object() {
                            items(m, value)?;
                        } else if let Some(a) = value.as_array() {
                            m.push((key.clone(), Item::new(a)?));
                        }
                    }
                    Some(())
                }
                _ => None,
            }
        }

        let file = File::open(path)?;
        let data: Value = serde_json::from_reader(BufReader::new(file))?;
        let mut m = Vec::new();
        items(&mut m, &data).ok_or(std::io::ErrorKind::InvalidData)?;
        Ok(Self(m))
    }

    #[inline]
    pub fn get(&self, key: impl AsRef<str>) -> Option<&Item> {
        self.0
            .iter()
            .find(|(k, _)| k == key.as_ref())
            .map(|(_, item)| item)
    }

    #[inline]
    pub fn iter(&self) -> std::slice::Iter<(String, Item)> {
        self.0.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_mmd_map() {
        let m = MmdMap::from_file("src/mmd_map.json").unwrap();
        let item = &m.iter().find(|(key, _)| key == "Undo").unwrap().1;
        assert!(item.name == "元に戻す");
        assert!(matches!(item.kind, ItemKind::Button(0x190)));
        let item = &m.iter().find(|(key, _)| key == "MenuHelpAbout").unwrap().1;
        assert!(item.name == "バージョン情報");
        assert!(matches!(item.kind, ItemKind::Menu(7, 6)));
    }
}
