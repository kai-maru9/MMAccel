use crate::*;
use std::io::BufRead;

fn str_to_vk(k: &str) -> Option<u32> {
    match k.trim() {
        "esc" => Some(VK_ESCAPE),
        "tab" => Some(VK_TAB),
        "capslock" => Some(VK_CAPITAL),
        "shift" => Some(VK_SHIFT),
        "ctrl" => Some(VK_CONTROL),
        "alt" => Some(VK_MENU),
        "backspace" => Some(VK_BACK),
        "enter" => Some(VK_RETURN),
        "space" => Some(VK_SPACE),
        "printscreen" => Some(VK_SNAPSHOT),
        "pause" => Some(VK_PAUSE),
        "insert" => Some(VK_INSERT),
        "delete" => Some(VK_DELETE),
        "home" => Some(VK_HOME),
        "end" => Some(VK_END),
        "pageup" => Some(VK_PRIOR),
        "pagedown" => Some(VK_NEXT),
        "up" => Some(VK_UP),
        "down" => Some(VK_DOWN),
        "left" => Some(VK_LEFT),
        "right" => Some(VK_RIGHT),
        "num+" => Some(VK_ADD),
        "num-" => Some(VK_SUBTRACT),
        "num*" => Some(VK_MULTIPLY),
        "num/" => Some(VK_DIVIDE),
        "num." => Some(VK_DECIMAL),
        "-" => Some(VK_OEM_MINUS),
        ";" => Some(VK_OEM_PLUS),
        "," => Some(VK_OEM_COMMA),
        "." => Some(VK_OEM_PERIOD),
        ":" => Some(VK_OEM_1),
        "/" => Some(VK_OEM_2),
        "@" => Some(VK_OEM_3),
        "[" => Some(VK_OEM_4),
        "\\" => Some(VK_OEM_5),
        "]" => Some(VK_OEM_6),
        "^" => Some(VK_OEM_7),
        "_" => Some(VK_OEM_102),
        _ if k.len() == 1 => {
            let c = k.chars().next().unwrap();
            (!c.is_ascii_control()).then(|| c.to_ascii_uppercase() as u32)
        }
        _ if k.starts_with("num") => {
            k.trim_matches(|c| !char::is_numeric(c)).parse().map(|n: u32| VK_NUMPAD0 + n).ok()
        }
        _ if k.starts_with('f') => {
            k.trim_matches(|c| !char::is_numeric(c)).parse().map(|n: u32| VK_F1 + n - 1).ok()
        }
        _ => None,
    }
}

#[derive(Debug)]
pub struct Item {
    pub id: String,
    pub keys: Option<Keys>,
}

#[derive(Debug)]
pub struct OldKeyMap(pub Vec<Item>);

impl OldKeyMap {
    pub fn from_file(path: impl AsRef<std::path::Path>) -> Result<Self, Error> {
        let file = std::fs::File::open(path)?;
        let mut reader = std::io::BufReader::new(file);
        let mut key_map = vec![];
        let mut buffer = String::new();
        loop {
            buffer.clear();
            if reader.read_line(&mut buffer)? == 0 {
                break;
            }
            if buffer.is_empty() {
                continue;
            }
            if buffer.starts_with('#') {
                continue;
            }
            let ss = buffer.split('=').collect::<Vec<_>>();
            if ss.len() != 2 {
                continue;
            }
            let keys = ss[1].trim().to_ascii_lowercase().split('+').map(|s| str_to_vk(s)).collect::<Option<Vec<_>>>();
            if keys.is_none() {
                continue;
            }
            key_map.push(Item {
                id: ss[0].trim().to_string(),
                keys: Some(Keys::from_slice(&keys.unwrap())),
            });
        }
        Ok(Self(key_map))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn load_key_map() {
        let data = OldKeyMap::from_file("key_map.txt").unwrap();
        let prev = data.0.iter().find(|item| item.id == "FramePrev").unwrap();
        let mut keys = Keys::new();
        keys.vk(b'A' as _);
        assert!(prev.keys.as_ref().unwrap() == &keys);
        let next = data.0.iter().find(|item| item.id == "FrameNext").unwrap();
        let mut keys = Keys::new();
        keys.vk(b'D' as _);
        assert!(next.keys.as_ref().unwrap() == &keys);
    }
}