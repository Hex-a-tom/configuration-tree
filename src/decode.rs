use byteorder::ReadBytesExt;
use std::{
    io::{ErrorKind, Read},
    path::Path,
};

use crate::{ConfigurationItem, ConfigurationNode, Endianess};

#[derive(Debug)]
pub enum DecodeError {
    InvalidFileError,
    ReadError,
    InvalidEncodingError,
}

fn read_byte(file: &mut impl Read) -> Result<u8, DecodeError> {
    let mut buf: [u8; 1] = [0];

    let res = file.read_exact(&mut buf);

    if let Ok(_) = res {
        Ok(buf[0])
    } else {
        Err(DecodeError::ReadError)
    }
}

fn decode_folder(file: &mut impl Read) -> Result<ConfigurationNode, DecodeError> {
    let mut node = ConfigurationNode::default();

    loop {
        let key = decode_key(file)?;
        if let Some(k) = key {
            if k.is_folder {
                node.entries
                    .insert(k.name, ConfigurationItem::Node(decode_folder(file)?));
            } else {
                node.entries.insert(k.name, decode_type(file)?);
            }
        } else {
            return Ok(node);
        }
    }
}

pub fn decode_file(file: &mut impl Read) -> Result<ConfigurationNode, DecodeError> {
    let header = decode_header(file);

    decode_folder(file)
}

fn decode_header(file: &mut impl Read) -> Result<(u32, Box<str>), DecodeError> {
    let mut buf: [u8; 4] = [0u8; 4];
    if let Err(_) = file.read_exact(&mut buf) {
        return Err(DecodeError::ReadError);
    }

    if buf != [0x7Fu8, 0x43u8, 0x54u8, 0x00u8] {
        return Err(DecodeError::InvalidFileError);
    }

    let mut buf: [u8; 8] = [0u8; 8];
    if let Err(_) = file.read_exact(&mut buf) {
        return Err(DecodeError::ReadError);
    }

    let version = if let Ok(v) = file.read_u32::<Endianess>() {
        v
    } else {
        return Err(DecodeError::ReadError);
    };

    let root = decode_string(file)?;

    Ok((version, root))
}

struct Key {
    is_folder: bool,
    name: Box<str>,
}

fn decode_key(file: &mut impl Read) -> Result<Option<Key>, DecodeError> {
    let byte_tmp = read_byte(file);

    if let Err(_) = byte_tmp {
        return Ok(None);
    }

    let byte = byte_tmp?;

    let folder: bool = byte >> 7 == 0;

    let len = 0x7Fu8 & byte;

    if len == 0 {
        return Ok(None);
    }

    let mut buf = vec![0u8; len as usize];
    if let Err(_) = file.read_exact(&mut buf) {
        return Err(DecodeError::ReadError);
    }

    Ok(Some(Key {
        is_folder: folder,
        name: String::from_utf8(buf)
            .map_err(|_| DecodeError::InvalidEncodingError)?
            .into_boxed_str(),
    }))
}

fn decode_usize(file: &mut impl Read) -> Result<usize, DecodeError> {
    let mut dat: usize = 0;

    for i in 0..10 {
        let byte = read_byte(file)?;

        dat += ((byte & 0x7Fu8) as usize) << (7 * i);

        if byte >> 7 == 0 {
            break;
        }
    }

    Ok(dat)
}

fn decode_isize(file: &mut impl Read) -> Result<isize, DecodeError> {
    let u = decode_usize(file)?;

    Ok((u as isize >> 1) ^ -(u as isize & 1))
}

fn decode_f32(file: &mut impl Read) -> Result<f32, DecodeError> {
    file.read_f32::<Endianess>()
        .map_err(|_| DecodeError::ReadError)
}

fn decode_f64(file: &mut impl Read) -> Result<f64, DecodeError> {
    file.read_f64::<Endianess>()
        .map_err(|_| DecodeError::ReadError)
}

fn decode_timestamp(file: &mut impl Read) -> Result<u64, DecodeError> {
    todo!()
}

fn decode_color(file: &mut impl Read) -> Result<[u8; 3], DecodeError> {
    Ok([read_byte(file)?, read_byte(file)?, read_byte(file)?])
}

fn decode_type(file: &mut impl Read) -> Result<ConfigurationItem, DecodeError> {
    match read_byte(file)? {
        0x01u8 => Ok(ConfigurationItem::Bool(false)),
        0x02u8 => Ok(ConfigurationItem::Bool(true)),
        0x03u8 => Ok(ConfigurationItem::Byte(read_byte(file)?)),
        0x04u8 => Ok(ConfigurationItem::Usize(decode_usize(file)?)),
        0x05u8 => Ok(ConfigurationItem::Isize(decode_isize(file)?)),
        0x06u8 => Ok(ConfigurationItem::F32(decode_f32(file)?)),
        0x07u8 => Ok(ConfigurationItem::F64(decode_f64(file)?)),
        0x08u8 => unimplemented!(),
        0x09u8 => Ok(ConfigurationItem::Color(decode_color(file)?)),
        0x80u8 => Ok(ConfigurationItem::ByteArray(decode_bytevec(file)?)),
        0x81u8 => Ok(ConfigurationItem::Array(decode_array(file)?)),
        0x82u8 => Ok(ConfigurationItem::String(decode_string(file)?)),
        0x83u8 => Ok(ConfigurationItem::Path(decode_path(file)?)),
        _ => Err(DecodeError::InvalidEncodingError),
    }
}

fn decode_bytevec(file: &mut impl Read) -> Result<Box<[u8]>, DecodeError> {
    let len = decode_usize(file)?;

    let mut buf = vec![0u8; len];

    file.read_exact(&mut buf)
        .map_err(|_| DecodeError::ReadError)?;

    Ok(buf.into())
}

fn decode_array(file: &mut impl Read) -> Result<Box<[ConfigurationItem]>, DecodeError> {
    let len = decode_usize(file)?;

    let mut buf: Vec<ConfigurationItem> = Vec::with_capacity(len);

    for _ in 0..len {
        buf.push(decode_type(file)?)
    }

    Ok(buf.into())
}

fn decode_string(file: &mut impl Read) -> Result<Box<str>, DecodeError> {
    let mut string = String::new();

    let mut buf: [u8; 1] = [0];

    loop {
        if let Ok(_) = file.read_exact(&mut buf) {
            if buf[0] == 0x00u8 {
                break;
            }
            string.push(buf[0] as char)
        } else {
            return Err(DecodeError::ReadError);
        }
    }

    Ok(string.into_boxed_str())
}

fn decode_path(file: &mut impl Read) -> Result<Box<Path>, DecodeError> {
    Ok(Path::new(&*decode_string(file)?).into())
}
