use byteorder::LittleEndian;
use std::{collections::HashMap, ops::Index, path::Path};

mod decode;
mod encode;

pub type Endianess = LittleEndian;

#[derive(Debug, Clone)]
pub enum ConfigurationItem {
    None,
    Bool(bool),
    Byte(u8),
    Usize(usize),
    Isize(isize),
    F32(f32),
    F64(f64),
    Timestamp,
    Color([u8; 0x3]),
    ByteArray(Box<[u8]>),
    Array(Box<[ConfigurationItem]>),
    String(Box<str>),
    Path(Box<Path>),
    Node(ConfigurationNode),
}

#[derive(Debug, Default, Clone)]
pub struct ConfigurationNode {
    entries: HashMap<Box<str>, ConfigurationItem>,
}

impl ConfigurationNode {
    /// Notice: other takes priority
    pub fn merge(&mut self, other: &Self) {
        for item in other.entries.iter() {
            if self.entries.contains_key(item.0) {
                if let ConfigurationItem::Node(ref mut s) = self.entries.get_mut(item.0).unwrap() {
                    if let ConfigurationItem::Node(n) = item.1 {
                        s.merge(n);
                    }
                    continue;
                }
            }
            _ = self.entries.insert(item.0.clone(), item.1.clone());
        }
    }
}

impl Index<&str> for ConfigurationNode {
    type Output = ConfigurationItem;

    fn index(&self, index: &str) -> &Self::Output {
        &self.entries[index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use decode::decode_file;
    use std::{fs::File, io::BufReader};

    use crate::encode::encode_configuration;

    #[test]
    fn test_decode_encode() {
        let dat = decode_file(&mut BufReader::new(File::open("test.ct").unwrap()));

        println!("{dat:#?}");

        let enc = encode_configuration(&dat.unwrap());

        println!("{enc:02X?}");

        let dat = decode_file(&mut enc.as_slice());

        println!("{dat:?}");
    }

    #[test]
    fn test_merge() {
        let mut initial = ConfigurationNode {
            entries: HashMap::from([
                (
                    "A".to_owned().into_boxed_str(),
                    ConfigurationItem::Bool(true),
                ),
                (
                    "C".to_owned().into_boxed_str(),
                    ConfigurationItem::Node(ConfigurationNode {
                        entries: HashMap::from([(
                            "C".to_owned().into_boxed_str(),
                            ConfigurationItem::Bool(true),
                        )]),
                    }),
                ),
                (
                    "D".to_owned().into_boxed_str(),
                    ConfigurationItem::Bool(false),
                ),
            ]),
        };

        let over = ConfigurationNode {
            entries: HashMap::from([
                (
                    "A".to_owned().into_boxed_str(),
                    ConfigurationItem::Node(ConfigurationNode {
                        entries: HashMap::from([(
                            "B".to_owned().into_boxed_str(),
                            ConfigurationItem::Bool(true),
                        )]),
                    }),
                ),
                (
                    "C".to_owned().into_boxed_str(),
                    ConfigurationItem::Bool(true),
                ),
                (
                    "D".to_owned().into_boxed_str(),
                    ConfigurationItem::Bool(true),
                ),
            ]),
        };

        initial.merge(&over);

        println!("{:#?}", initial);
    }
}
