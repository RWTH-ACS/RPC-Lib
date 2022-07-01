// Copyright 2022 Philipp Fensch
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::convert::{TryFrom, TryInto};
use std::vec::Vec;

/// Types with the `Xdr`-Trait can be serialised and deserialised as described in [`RFC 4506`]
///
/// [`RFC 4506`]: <https://datatracker.ietf.org/doc/html/rfc4506>
pub trait Xdr {
    // Serializes data and converts to network byte order
    fn serialize(&self) -> std::vec::Vec<u8>;
    // Reverse Operation
    fn deserialize(bytes: &[u8], parse_index: &mut usize) -> Self;
}

/// Implementation for fixed-size arrays
impl<T: Xdr, const LEN: usize> Xdr for [T; LEN] {
    fn serialize(&self) -> std::vec::Vec<u8> {
        let mut vec = (self.len() as u32).serialize();
        for item in self {
            vec.extend(item.serialize());
        }
        vec
    }

    fn deserialize(bytes: &[u8], parse_index: &mut usize) -> [T; LEN] {
        let mut array: [T; LEN] = unsafe { std::mem::MaybeUninit::uninit().assume_init() };

        for i in 0..LEN {
            array[i] = T::deserialize(bytes, parse_index);
        }
        array
    }
}

/// Implementation for Variable-Length arrays
impl<T: std::clone::Clone> Xdr for Vec<T> {
    fn serialize(&self) -> std::vec::Vec<u8> {
        // Length of data in bytes
        let data_len = (self.len() * std::mem::size_of::<T>()) as u32;
        let mut vec = data_len.serialize();

        // Data in Vector
        let slice = unsafe { std::mem::transmute::<&[T], &[u8]>(&self) };
        vec.extend(slice);
        
        // Alignment on 4 bytes
        if data_len % 4 != 0 {
            let padding = ((data_len / 4) * 4 + 4) - data_len;
            for _i in 0..padding {
                vec.push(0);
            }
        }
        vec
    }

    fn deserialize(bytes: &[u8], parse_index: &mut usize) -> Vec<T> {
        // Length
        let len: usize = u32::deserialize(bytes, parse_index).try_into().unwrap();

        // Data
        let slice = unsafe { std::mem::transmute::<&[u8], &[T]>(&bytes[*parse_index..*parse_index + len]) };
        let vec = slice.to_vec();

        // Alignment on 4 bytes
        if len % 4 != 0 {
            let padding = ((len / 4) * 4 + 4) - len;
            *parse_index += padding;
        }
        
        vec
    }
}

impl Xdr for i32 {
    fn serialize(&self) -> std::vec::Vec<u8> {
        self.to_be_bytes().to_vec()
    }

    fn deserialize(bytes: &[u8], parse_index: &mut usize) -> i32 {
        let x = <&[u8; 4]>::try_from(&bytes[*parse_index..*parse_index + 4]).unwrap();
        *parse_index += 4;
        i32::from_be_bytes(*x)
    }
}

impl Xdr for u32 {
    fn serialize(&self) -> std::vec::Vec<u8> {
        self.to_be_bytes().to_vec()
    }

    fn deserialize(bytes: &[u8], parse_index: &mut usize) -> u32 {
        let x = <&[u8; 4]>::try_from(&bytes[*parse_index..*parse_index + 4]).unwrap();
        *parse_index += 4;
        u32::from_be_bytes(*x)
    }
}

impl Xdr for i64 {
    fn serialize(&self) -> std::vec::Vec<u8> {
        self.to_be_bytes().to_vec()
    }

    fn deserialize(bytes: &[u8], parse_index: &mut usize) -> i64 {
        let x = <&[u8; 8]>::try_from(&bytes[*parse_index..*parse_index + 8]).unwrap();
        *parse_index += 8;
        i64::from_be_bytes(*x)
    }
}

impl Xdr for u64 {
    fn serialize(&self) -> std::vec::Vec<u8> {
        self.to_be_bytes().to_vec()
    }

    fn deserialize(bytes: &[u8], parse_index: &mut usize) -> u64 {
        let x = <&[u8; 8]>::try_from(&bytes[*parse_index..*parse_index + 8]).unwrap();
        *parse_index += 8;
        u64::from_be_bytes(*x)
    }
}

impl Xdr for String {
    fn serialize(&self) -> std::vec::Vec<u8> {
        let mut vec = (self.len() as u32).serialize();
        vec.extend(self.as_bytes());
        // Alignment on 4 bytes
        let padding = ((self.len() / 4) * 4 + 4) - self.len();
        for _i in 0..padding {
            vec.push(0);
        }
        vec
    }

    fn deserialize(bytes: &[u8], parse_index: &mut usize) -> String {
        let len: usize = u32::deserialize(bytes, parse_index).try_into().unwrap();
        let s = String::from_utf8(bytes[*parse_index..*parse_index + len].to_vec()).unwrap();
        let len_and_padding = ((len + 4) / 4) * 4;
        *parse_index += len_and_padding;
        s
    }
}

impl Xdr for f32 {
    fn serialize(&self) -> std::vec::Vec<u8> {
        self.to_be_bytes().to_vec()
    }

    fn deserialize(bytes: &[u8], parse_index: &mut usize) -> f32 {
        let x = <&[u8; 4]>::try_from(&bytes[*parse_index..*parse_index + 4]).unwrap();
        *parse_index += 4;
        f32::from_be_bytes(*x)
    }
}

impl Xdr for f64 {
    fn serialize(&self) -> std::vec::Vec<u8> {
        self.to_be_bytes().to_vec()
    }

    fn deserialize(bytes: &[u8], parse_index: &mut usize) -> f64 {
        let x = <&[u8; 8]>::try_from(&bytes[*parse_index..*parse_index + 8]).unwrap();
        *parse_index += 8;
        f64::from_be_bytes(*x)
    }
}
