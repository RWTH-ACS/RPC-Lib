// Copyright 2022 Philipp Fensch
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::convert::{TryFrom, TryInto};
use std::io::{self, Write};
use std::vec::Vec;

/// Types with the `Xdr`-Trait can be serialised and deserialised as described in [`RFC 4506`]
///
/// [`RFC 4506`]: <https://datatracker.ietf.org/doc/html/rfc4506>
pub trait Xdr {
    // Serializes data and converts to network byte order
    fn serialize(&self, writer: impl Write) -> io::Result<()>;
    // Reverse Operation
    fn deserialize(bytes: &[u8], parse_index: &mut usize) -> Self;
}

fn padding(len: usize) -> usize {
    (4 - len % 4) % 4
}

/// Implementation for fixed-size arrays
impl<T: Xdr, const LEN: usize> Xdr for [T; LEN] {
    fn serialize(&self, mut writer: impl Write) -> io::Result<()> {
        (self.len() as u32).serialize(&mut writer)?;
        for item in self {
            item.serialize(&mut writer)?;
        }
        Ok(())
    }

    fn deserialize(bytes: &[u8], parse_index: &mut usize) -> [T; LEN] {
        let mut vec = Vec::with_capacity(LEN);
        for _ in 0..LEN {
            vec.push(T::deserialize(bytes, parse_index));
        }
        match vec.try_into() {
            Ok(array) => array,
            Err(_) => unreachable!(),
        }
    }
}

/// Implementation for Variable-Length arrays
impl<T: std::clone::Clone> Xdr for Vec<T> {
    fn serialize(&self, mut writer: impl Write) -> io::Result<()> {
        // Length of data in bytes
        let data_len = (self.len() * std::mem::size_of::<T>()) as u32;
        data_len.serialize(&mut writer)?;

        // Data in Vector
        let slice = unsafe { std::mem::transmute::<&[T], &[u8]>(&self) };
        writer.write_all(slice)?;

        // Alignment on 4 bytes
        if data_len % 4 != 0 {
            let padding = ((data_len / 4) * 4 + 4) - data_len;
            for _i in 0..padding {
                writer.write_all(&[0])?;
            }
        }
        Ok(())
    }

    fn deserialize(bytes: &[u8], parse_index: &mut usize) -> Vec<T> {
        // Length
        let len: usize = u32::deserialize(bytes, parse_index).try_into().unwrap();

        // Data
        let slice =
            unsafe { std::mem::transmute::<&[u8], &[T]>(&bytes[*parse_index..*parse_index + len]) };
        let vec = slice.to_vec();
        *parse_index += len;

        // Alignment on 4 bytes
        if len % 4 != 0 {
            let padding = ((len / 4) * 4 + 4) - len;
            *parse_index += padding;
        }

        vec
    }
}

impl Xdr for i32 {
    fn serialize(&self, mut writer: impl Write) -> io::Result<()> {
        writer.write_all(&self.to_be_bytes())
    }

    fn deserialize(bytes: &[u8], parse_index: &mut usize) -> i32 {
        let x = <&[u8; 4]>::try_from(&bytes[*parse_index..*parse_index + 4]).unwrap();
        *parse_index += 4;
        i32::from_be_bytes(*x)
    }
}

impl Xdr for u32 {
    fn serialize(&self, mut writer: impl Write) -> io::Result<()> {
        writer.write_all(&self.to_be_bytes())
    }

    fn deserialize(bytes: &[u8], parse_index: &mut usize) -> u32 {
        let x = <&[u8; 4]>::try_from(&bytes[*parse_index..*parse_index + 4]).unwrap();
        *parse_index += 4;
        u32::from_be_bytes(*x)
    }
}

impl Xdr for i64 {
    fn serialize(&self, mut writer: impl Write) -> io::Result<()> {
        writer.write_all(&self.to_be_bytes())
    }

    fn deserialize(bytes: &[u8], parse_index: &mut usize) -> i64 {
        let x = <&[u8; 8]>::try_from(&bytes[*parse_index..*parse_index + 8]).unwrap();
        *parse_index += 8;
        i64::from_be_bytes(*x)
    }
}

impl Xdr for u64 {
    fn serialize(&self, mut writer: impl Write) -> io::Result<()> {
        writer.write_all(&self.to_be_bytes())
    }

    fn deserialize(bytes: &[u8], parse_index: &mut usize) -> u64 {
        let x = <&[u8; 8]>::try_from(&bytes[*parse_index..*parse_index + 8]).unwrap();
        *parse_index += 8;
        u64::from_be_bytes(*x)
    }
}

impl Xdr for String {
    fn serialize(&self, mut writer: impl Write) -> io::Result<()> {
        (self.len() as u32).serialize(&mut writer)?;
        writer.write_all(self.as_bytes())?;
        // Alignment on 4 bytes
        for _i in 0..padding(self.len()) {
            writer.write_all(&[0])?;
        }
        Ok(())
    }

    fn deserialize(bytes: &[u8], parse_index: &mut usize) -> String {
        let len: usize = u32::deserialize(bytes, parse_index).try_into().unwrap();
        let s = String::from_utf8(bytes[*parse_index..*parse_index + len].to_vec()).unwrap();
        *parse_index += len + padding(len);
        s
    }
}

impl Xdr for f32 {
    fn serialize(&self, mut writer: impl Write) -> io::Result<()> {
        writer.write_all(&self.to_be_bytes())
    }

    fn deserialize(bytes: &[u8], parse_index: &mut usize) -> f32 {
        let x = <&[u8; 4]>::try_from(&bytes[*parse_index..*parse_index + 4]).unwrap();
        *parse_index += 4;
        f32::from_be_bytes(*x)
    }
}

impl Xdr for f64 {
    fn serialize(&self, mut writer: impl Write) -> io::Result<()> {
        writer.write_all(&self.to_be_bytes())
    }

    fn deserialize(bytes: &[u8], parse_index: &mut usize) -> f64 {
        let x = <&[u8; 8]>::try_from(&bytes[*parse_index..*parse_index + 8]).unwrap();
        *parse_index += 8;
        f64::from_be_bytes(*x)
    }
}
