use bindings::Windows::Win32::WindowsAndMessaging::*;
use serde::ser::SerializeMap;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;

pub fn vk_to_string(k: u32) -> String {
    const ZERO: u32 = b'0' as _;
    const Z: u32 = b'Z' as _;
    match k {
        k @ ZERO..=Z => (k as u8 as char).to_string(),
        VK_ESCAPE => "Esc".into(),
        VK_TAB => "Tab".into(),
        VK_CAPITAL => "CapsLock".into(),
        VK_SHIFT => "Shift".into(),
        VK_CONTROL => "Ctrl".into(),
        VK_MENU => "Alt".into(),
        VK_BACK => "BackSpace".into(),
        VK_RETURN => "Enter".into(),
        VK_SPACE => "Space".into(),
        VK_SNAPSHOT => "PrintScreen".into(),
        VK_SCROLL => "ScrollLock".into(),
        VK_PAUSE => "Pause".into(),
        VK_INSERT => "Insert".into(),
        VK_DELETE => "Delete".into(),
        VK_HOME => "Home".into(),
        VK_END => "End".into(),
        VK_PRIOR => "PageUp".into(),
        VK_NEXT => "PageDown".into(),
        VK_UP => "Up".into(),
        VK_DOWN => "Down".into(),
        VK_LEFT => "Left".into(),
        VK_RIGHT => "Right".into(),
        VK_NUMLOCK => "NumLock".into(),
        v @ VK_NUMPAD0..=VK_NUMPAD9 => format!("Num{}", v - VK_NUMPAD0),
        VK_ADD => "Num+".into(),
        VK_SUBTRACT => "Num-".into(),
        VK_MULTIPLY => "Num*".into(),
        VK_DIVIDE => "Num/".into(),
        VK_DECIMAL => "Num.".into(),
        // VK_LSHIFT => "LShift".into(),
        // VK_RSHIFT => "RShift".into(),
        // VK_LCONTROL => "LCtrl".into(),
        // VK_RCONTROL => "RCtrl".into(),
        // VK_LMENU => "LAlt".into(),
        // VK_RMENU => "RAlt".into(),
        v @ VK_F1..=VK_F24 => format!("F{}", v - VK_F1 + 1),
        VK_OEM_MINUS => "-".into(),
        VK_OEM_PLUS => ";".into(),
        VK_OEM_COMMA => ",".into(),
        VK_OEM_PERIOD => ".".into(),
        VK_OEM_1 => ":".into(),
        VK_OEM_2 => "/".into(),
        VK_OEM_3 => "@".into(),
        VK_OEM_4 => "[".into(),
        VK_OEM_5 => "\\".into(),
        VK_OEM_6 => "]".into(),
        VK_OEM_7 => "^".into(),
        VK_OEM_102 => "_".into(),
        _ => format!("({})", k),
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug, serde::Serialize, serde::Deserialize)]
pub struct Keys(Vec<u32>);

impl Keys {
    #[inline]
    pub fn new() -> Self {
        Self(vec![])
    }

    #[inline]
    pub fn from_slice(v: &[u32]) -> Self {
        let mut v = v.to_vec();
        v.sort_unstable();
        Self(v)
    }

    #[inline]
    pub fn with_capacity(n: usize) -> Self {
        Self(Vec::with_capacity(n))
    }

    #[inline]
    pub fn clear(&mut self) {
        self.0.clear();
    }

    #[inline]
    pub fn keyboard_state(&mut self, v: &[u8]) {
        #[inline]
        fn is_lr_key(k: u32) -> bool {
            k == VK_LSHIFT || k == VK_RSHIFT || k == VK_LCONTROL || k == VK_RCONTROL || k == VK_LMENU || k == VK_RMENU
        }

        self.0.clear();
        for (i, k) in v.iter().enumerate() {
            if i <= 0xe0 && (k & 0x80) != 0 && !is_lr_key(i as _) {
                self.0.push(i as u32);
            }
        }
        self.0.sort_unstable();
    }

    #[inline]
    pub fn vk(&mut self, vk: u32) {
        self.0.clear();
        self.0.push(vk);
    }

    pub fn to_strings(&self) -> Vec<String> {
        let mut v = vec![];
        for &k in self.0.iter() {
            v.push(vk_to_string(k));
        }
        v
    }
}

impl Default for Keys {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct KeyMap(HashMap<String, Keys>);

impl KeyMap {
    pub fn from_file(path: impl AsRef<std::path::Path>) -> std::io::Result<Self> {
        let file = File::open(path)?;
        Ok(serde_json::from_reader(BufReader::new(file))?)
    }

    #[inline]
    pub fn insert(&mut self, k: impl AsRef<str>, v: Keys) {
        self.0.insert(k.as_ref().into(), v);
    }

    #[inline]
    pub fn get(&self, k: impl AsRef<str>) -> Option<&Keys> {
        self.0.get(k.as_ref())
    }

    #[inline]
    pub fn get_mut(&mut self, k: impl AsRef<str>) -> Option<&mut Keys> {
        self.0.get_mut(k.as_ref())
    }

    #[inline]
    pub fn iter(&self) -> std::collections::hash_map::Iter<String, Keys> {
        self.0.iter()
    }
}

impl std::iter::IntoIterator for KeyMap {
    type Item = (String, Keys);
    type IntoIter = std::collections::hash_map::IntoIter<String, Keys>;

    #[inline]
    fn into_iter(self) -> std::collections::hash_map::IntoIter<String, Keys> {
        self.0.into_iter()
    }
}

impl serde::Serialize for KeyMap {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for (k, v) in self.0.iter() {
            map.serialize_entry(k, v)?;
        }
        map.end()
    }
}

impl<'de> serde::Deserialize<'de> for KeyMap {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> serde::de::Visitor<'de> for Visitor {
            type Value = KeyMap;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(formatter, "KeyMap")
            }

            fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
            where
                M: serde::de::MapAccess<'de>,
            {
                let mut map = HashMap::with_capacity(access.size_hint().unwrap_or(0));
                while let Some((key, value)) = access.next_entry()? {
                    map.insert(key, value);
                }
                Ok(KeyMap(map))
            }
        }

        deserializer.deserialize_map(Visitor)
    }
}

impl Default for KeyMap {
    fn default() -> Self {
        let mut m = Self(HashMap::new());
        m.insert("Undo", Keys::from_slice(&[VK_CONTROL, b'Z' as _]));
        m.insert("Redo", Keys::from_slice(&[VK_CONTROL, b'X' as _]));
        m.insert("BoneSelect", Keys::from_slice(&[b'C' as _]));
        m.insert("BoneRotate", Keys::from_slice(&[b'X' as _]));
        m.insert("BoneMove", Keys::from_slice(&[b'Z' as _]));
        m.insert("BoneAllSelect", Keys::from_slice(&[b'A' as _]));
        m.insert("BoneUnregisterSelect", Keys::from_slice(&[b'S' as _]));
        m.insert("MenuViewHalfTransparency", Keys::from_slice(&[b'V' as _]));
        m.insert("FramePrev", Keys::from_slice(&[VK_LEFT]));
        m.insert("FrameNext", Keys::from_slice(&[VK_RIGHT]));
        m.insert("MainChangeEditor", Keys::from_slice(&[VK_TAB]));
        m.insert("FrameRegister", Keys::from_slice(&[VK_RETURN]));
        m.insert("FrameKeyPrev", Keys::from_slice(&[VK_CONTROL, VK_LEFT]));
        m.insert("FrameKeyNext", Keys::from_slice(&[VK_CONTROL, VK_RIGHT]));
        m.insert("BonePrev", Keys::from_slice(&[VK_UP]));
        m.insert("BoneNext", Keys::from_slice(&[VK_DOWN]));
        m.insert("KeyCopy", Keys::from_slice(&[VK_CONTROL, b'C' as _]));
        m.insert("KeyPaste", Keys::from_slice(&[VK_CONTROL, b'V' as _]));
        m.insert("MenuBackgroundBlack", Keys::from_slice(&[b'B' as _]));
        m.insert("MenuEditCenterBias", Keys::from_slice(&[b'D' as _]));
        m.insert("ChangeSpace", Keys::from_slice(&[b'L' as _]));
        m.insert("MenuEditAnotherFramePaste", Keys::from_slice(&[b'F' as _]));
        m.insert("MenuEditInsertEmptyFrame", Keys::from_slice(&[b'I' as _]));
        m.insert("MenuEditDeleteVerticalFrames", Keys::from_slice(&[b'K' as _]));
        m.insert(
            "MenuEditInsertEmptyFrameMorphOrLighting",
            Keys::from_slice(&[b'U' as _]),
        );
        m.insert(
            "MenuEditDeleteVerticalFramesMorphOrLighting",
            Keys::from_slice(&[b'J' as _]),
        );
        m.insert("MenuEditCorrectBone", Keys::from_slice(&[b'R' as _]));
        m.insert("Play", Keys::from_slice(&[b'P' as _]));
        m.insert("ViewBottom", Keys::from_slice(&[b'0' as _]));
        m.insert("ViewFront", Keys::from_slice(&[b'2' as _]));
        m.insert("ViewLeft", Keys::from_slice(&[b'4' as _]));
        m.insert("ViewTop", Keys::from_slice(&[b'5' as _]));
        m.insert("ViewRight", Keys::from_slice(&[b'6' as _]));
        m.insert("ViewBack", Keys::from_slice(&[b'8' as _]));
        m.insert("MenuFileSave", Keys::from_slice(&[VK_CONTROL, b'S' as _]));
        m.insert("InterpolationAuto", Keys::from_slice(&[VK_OEM_6]));
        m
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vk_to_string_test() {
        assert!(vk_to_string(VK_LEFT) == "Left");
        assert!(vk_to_string(b'A' as _) == "A");
        assert!(vk_to_string(VK_NUMPAD0) == "Num0");
        assert!(vk_to_string(VK_F5) == "F5");
        assert!(vk_to_string(0xdf) == "(223)");
    }

    #[test]
    fn key_map_test() {
        let mut key_map = KeyMap(HashMap::new());
        key_map.insert("Undo", Keys(vec![VK_CONTROL, b'Z' as _]));
        key_map.insert("Redo", Keys(vec![VK_CONTROL, VK_SHIFT, b'Z' as _]));
        let ret: KeyMap = serde_json::from_str(&serde_json::to_string(&key_map).unwrap()).unwrap();
        assert!(ret.get("Undo").unwrap() == &Keys(vec![VK_CONTROL, b'Z' as _]));
        assert!(ret.get("Redo").unwrap() == &Keys(vec![VK_CONTROL, VK_SHIFT, b'Z' as _]));
        assert!(ret.get("Undo").unwrap() != &Keys(vec![VK_SHIFT, b'Z' as _]));
    }
}
