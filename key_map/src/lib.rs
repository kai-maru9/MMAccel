use serde::ser::SerializeMap;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use windows::Win32::UI::Input::KeyboardAndMouse::*;

pub fn vk_to_string(k: u32) -> String {
    const ZERO: u16 = b'0' as _;
    const Z: u16 = b'Z' as _;
    const NUMPAD0: u16 = VK_NUMPAD0.0 as _;
    const NUMPAD9: u16 = VK_NUMPAD9.0 as _;
    const F1: u16 = VK_F1.0 as _;
    const F24: u16 = VK_F24.0 as _;
    match VIRTUAL_KEY(k as _) {
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
        // VK_SCROLL => "ScrollLock".into(),
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
        // VK_NUMLOCK => "NumLock".into(),
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
        l => match l.0 {
            v @ ZERO..=Z => (v as u8 as char).to_string(),
            v @ NUMPAD0..=NUMPAD9 => format!("Num{}", v - VK_NUMPAD0.0),
            v @ F1..=F24 => format!("F{}", v - VK_F1.0 + 1),
            _ => format!("({})", l.0),
        },
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
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    #[inline]
    pub fn clear(&mut self) {
        self.0.clear();
    }

    #[inline]
    pub fn keyboard_state(&mut self, v: &[u8]) {
        #[inline]
        fn is_lr_key(k: u32) -> bool {
            k == VK_LSHIFT.0 as u32
                || k == VK_RSHIFT.0 as u32
                || k == VK_LCONTROL.0 as u32
                || k == VK_RCONTROL.0 as u32
                || k == VK_LMENU.0 as u32
                || k == VK_RMENU.0 as u32
        }

        self.0.clear();
        for (i, k) in v.iter().enumerate() {
            if (0x07..0xe0).contains(&i) && (k & 0x80) != 0 && !is_lr_key(i as _) {
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

    #[inline]
    pub fn is_included(&self, other: &Keys) -> bool {
        let mut index = 0;
        for x in other.0.iter() {
            if *x == (self.0)[index] {
                index += 1;
                if index == self.0.len() {
                    return true;
                }
            }
        }
        false
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
    pub fn new() -> Self {
        Self(HashMap::new())
    }

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
        m.insert("Undo", Keys::from_slice(&[VK_CONTROL.0 as _, b'Z' as _]));
        m.insert("Redo", Keys::from_slice(&[VK_CONTROL.0 as _, b'X' as _]));
        m.insert("BoneSelect", Keys::from_slice(&[b'C' as _]));
        m.insert("BoneRotate", Keys::from_slice(&[b'X' as _]));
        m.insert("BoneMove", Keys::from_slice(&[b'Z' as _]));
        m.insert("BoneAllSelect", Keys::from_slice(&[b'A' as _]));
        m.insert("BoneUnregisterSelect", Keys::from_slice(&[b'S' as _]));
        m.insert("MenuViewHalfTransparency", Keys::from_slice(&[b'V' as _]));
        m.insert("FramePrev", Keys::from_slice(&[VK_LEFT.0 as _]));
        m.insert("FrameNext", Keys::from_slice(&[VK_RIGHT.0 as _]));
        m.insert("MainChangeEditor", Keys::from_slice(&[VK_TAB.0 as _]));
        m.insert("FrameRegister", Keys::from_slice(&[VK_RETURN.0 as _]));
        m.insert("FrameKeyPrev", Keys::from_slice(&[VK_CONTROL.0 as _, VK_LEFT.0 as _]));
        m.insert("FrameKeyNext", Keys::from_slice(&[VK_CONTROL.0 as _, VK_RIGHT.0 as _]));
        m.insert("BonePrev", Keys::from_slice(&[VK_UP.0 as _]));
        m.insert("BoneNext", Keys::from_slice(&[VK_DOWN.0 as _]));
        m.insert("KeyCopy", Keys::from_slice(&[VK_CONTROL.0 as _, b'C' as _]));
        m.insert("KeyPaste", Keys::from_slice(&[VK_CONTROL.0 as _, b'V' as _]));
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
        m.insert("MenuFileSave", Keys::from_slice(&[VK_CONTROL.0 as _, b'S' as _]));
        m.insert("InterpolationAuto", Keys::from_slice(&[VK_OEM_6.0 as _]));
        m
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vk_to_string_test() {
        assert!(vk_to_string(VK_LEFT.0 as _) == "Left");
        assert!(vk_to_string(b'A' as _) == "A");
        assert!(vk_to_string(VK_NUMPAD0.0 as _) == "Num0");
        assert!(vk_to_string(VK_F5.0 as _) == "F5");
        assert!(vk_to_string(0xdf) == "(223)");
    }

    #[test]
    fn key_map_test() {
        let mut key_map = KeyMap(HashMap::new());
        key_map.insert("Undo", Keys(vec![VK_CONTROL.0 as _, b'Z' as _]));
        key_map.insert("Redo", Keys(vec![VK_CONTROL.0 as _, VK_SHIFT.0 as _, b'Z' as _]));
        let ret: KeyMap = serde_json::from_str(&serde_json::to_string(&key_map).unwrap()).unwrap();
        assert!(ret.get("Undo").unwrap() == &Keys(vec![VK_CONTROL.0 as _, b'Z' as _]));
        assert!(ret.get("Redo").unwrap() == &Keys(vec![VK_CONTROL.0 as _, VK_SHIFT.0 as _, b'Z' as _]));
        assert!(ret.get("Undo").unwrap() != &Keys(vec![VK_SHIFT.0 as _, b'Z' as _]));
    }
}
