use crate::*;
use std::io::BufRead;

fn str_to_vk(k: &str) -> Option<u32> {
    match k.trim() {
        "esc" => Some(VK_ESCAPE.0 as u32),
        "tab" => Some(VK_TAB.0 as u32),
        "capslock" => Some(VK_CAPITAL.0 as u32),
        "shift" => Some(VK_SHIFT.0 as u32),
        "ctrl" => Some(VK_CONTROL.0 as u32),
        "alt" => Some(VK_MENU.0 as u32),
        "backspace" => Some(VK_BACK.0 as u32),
        "enter" => Some(VK_RETURN.0 as u32),
        "space" => Some(VK_SPACE.0 as u32),
        "printscreen" => Some(VK_SNAPSHOT.0 as u32),
        "pause" => Some(VK_PAUSE.0 as u32),
        "insert" => Some(VK_INSERT.0 as u32),
        "delete" => Some(VK_DELETE.0 as u32),
        "home" => Some(VK_HOME.0 as u32),
        "end" => Some(VK_END.0 as u32),
        "pageup" => Some(VK_PRIOR.0 as u32),
        "pagedown" => Some(VK_NEXT.0 as u32),
        "up" => Some(VK_UP.0 as u32),
        "down" => Some(VK_DOWN.0 as u32),
        "left" => Some(VK_LEFT.0 as u32),
        "right" => Some(VK_RIGHT.0 as u32),
        "num+" => Some(VK_ADD.0 as u32),
        "num-" => Some(VK_SUBTRACT.0 as u32),
        "num*" => Some(VK_MULTIPLY.0 as u32),
        "num/" => Some(VK_DIVIDE.0 as u32),
        "num." => Some(VK_DECIMAL.0 as u32),
        "-" => Some(VK_OEM_MINUS.0 as u32),
        ";" => Some(VK_OEM_PLUS.0 as u32),
        "," => Some(VK_OEM_COMMA.0 as u32),
        "." => Some(VK_OEM_PERIOD.0 as u32),
        ":" => Some(VK_OEM_1.0 as u32),
        "/" => Some(VK_OEM_2.0 as u32),
        "@" => Some(VK_OEM_3.0 as u32),
        "[" => Some(VK_OEM_4.0 as u32),
        "\\" => Some(VK_OEM_5.0 as u32),
        "]" => Some(VK_OEM_6.0 as u32),
        "^" => Some(VK_OEM_7.0 as u32),
        "_" => Some(VK_OEM_102.0 as u32),
        _ if k.len() == 1 => {
            let c = k.chars().next().unwrap();
            (!c.is_ascii_control()).then(|| c.to_ascii_uppercase() as u32)
        }
        _ if k.starts_with("num") => k
            .trim_matches(|c| !char::is_numeric(c))
            .parse()
            .map(|n: u32| VK_NUMPAD0.0 as u32 + n)
            .ok(),
        _ if k.starts_with('f') => k
            .trim_matches(|c| !char::is_numeric(c))
            .parse()
            .map(|n: u32| VK_F1.0 as u32 + n - 1)
            .ok(),
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
            let keys = ss[1]
                .trim()
                .to_ascii_lowercase()
                .split('+')
                .map(|s| str_to_vk(s))
                .collect::<Option<Vec<_>>>();
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
