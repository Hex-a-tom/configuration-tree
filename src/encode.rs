use std::os::unix::prelude::OsStrExt;

use crate::{ConfigurationItem, ConfigurationNode};

pub fn encode_configuration(root: &ConfigurationNode) -> Vec<u8> {
    let mut vec = vec![
        0x7Fu8, 0x43u8, 0x54u8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];

    encode_node(&mut vec, root);

    vec
}

fn encode_node(vec: &mut Vec<u8>, node: &ConfigurationNode) {
    for i in node.entries.iter() {
        match i.1 {
            ConfigurationItem::Node(n) => {
                encode_key(vec, true, i.0);
                encode_node(vec, &n);
            }
            _ => {
                encode_key(vec, false, i.0);
                encode_type(vec, i.1);
            }
        }
    }
    vec.push(0x00u8)
}

fn encode_key(vec: &mut Vec<u8>, dir: bool, name: &str) {
    debug_assert!(name.len() <= 128, "Name was too long!");

    let dir_val = if dir { 0u8 } else { 0x80u8 };

    let head = dir_val + name.len() as u8;

    vec.push(head);
    vec.extend(name.as_bytes());
}

fn encode_usize(vec: &mut Vec<u8>, val: usize) {
    for i in 0..10 {
        let n = val >> (7 * i);

        if n < 0x80usize {
            vec.push(n as u8);
            return;
        } else {
            vec.push((n & 0x7Fusize) as u8 + 0x80u8)
        }
    }
}

fn encode_isize(vec: &mut Vec<u8>, val: isize) {
    encode_usize(vec, ((val << 1) ^ (val >> isize::BITS - 1)) as usize)
}

fn encode_type(vec: &mut Vec<u8>, entry: &ConfigurationItem) {
    match entry {
        ConfigurationItem::Bool(b) => {
            if *b {
                vec.push(0x02u8)
            } else {
                vec.push(0x01u8)
            }
        }
        ConfigurationItem::Byte(b) => vec.extend([0x03u8, *b]),
        ConfigurationItem::Usize(u) => {
            vec.push(0x04u8);
            encode_usize(vec, *u)
        }
        ConfigurationItem::Isize(i) => {
            vec.push(0x05u8);
            encode_isize(vec, *i)
        }
        ConfigurationItem::F32(f) => {
            vec.push(0x06u8);
            vec.extend(f.to_le_bytes());
        }
        ConfigurationItem::F64(f) => {
            vec.push(0x07u8);
            vec.extend(f.to_le_bytes());
        }
        ConfigurationItem::Timestamp => unimplemented!(),
        ConfigurationItem::Color(c) => {
            vec.push(0x09u8);
            vec.extend(c);
        }
        ConfigurationItem::ByteArray(a) => {
            vec.push(0x80u8);
            encode_usize(vec, a.len());
            vec.extend(a.iter());
        }
        ConfigurationItem::Array(a) => unimplemented!(),
        ConfigurationItem::String(s) => {
            vec.push(0x82u8);
            vec.extend(s.bytes());
            vec.push(0x00u8);
        }
        ConfigurationItem::Path(p) => {
            vec.push(0x83u8);
            vec.extend(p.as_os_str().as_bytes());
            vec.push(0x00u8);
        }
        _ => unimplemented!(),
    }
}
