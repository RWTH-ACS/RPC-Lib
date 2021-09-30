// This File contains Serialization for Basic Datatypes:
//  * Signed Integers:   i32
//  * Unsigned Integers: u32, u64
//  * Strings:           String
//  * Floats:

use std::convert::{TryFrom, TryInto};
use std::vec::Vec;

pub trait Xdr {
    // Serializes data and converts to network byte order
    fn serialize(&self) -> std::vec::Vec<u8>;
    // Reverse Operation
    fn deserialize(bytes: &Vec<u8>, parse_index: &mut usize) -> Self;
}

// Constant Length Array
impl<T: Xdr, const LEN: usize> Xdr for [T; LEN] {
    fn serialize(&self) -> std::vec::Vec<u8> {
        let mut vec = (self.len() as u32).serialize();
        for item in self {
            vec.extend(item.serialize());
        }
        vec
    }

    fn deserialize(bytes: &Vec<u8>, parse_index: &mut usize) -> [T; LEN] {
        let mut array: [T; LEN] = unsafe { std::mem::MaybeUninit::uninit().assume_init() };

        for i in 0..LEN {
            array[i] = T::deserialize(bytes, parse_index);
        }
        array
    }
}

// Varlen Array
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

    fn deserialize(bytes: &Vec<u8>, parse_index: &mut usize) -> Vec<T> {
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

impl Xdr for i8 {
    fn serialize(&self) -> std::vec::Vec<u8> {
        self.to_be_bytes().to_vec()
    }

    fn deserialize(bytes: &Vec<u8>, parse_index: &mut usize) -> i8 {
        let x = <&[u8; 1]>::try_from(&bytes[*parse_index..*parse_index + 1]).unwrap();
        *parse_index += 1;
        i8::from_be_bytes(*x)
    }
}

impl Xdr for i32 {
    fn serialize(&self) -> std::vec::Vec<u8> {
        self.to_be_bytes().to_vec()
    }

    fn deserialize(bytes: &Vec<u8>, parse_index: &mut usize) -> i32 {
        let x = <&[u8; 4]>::try_from(&bytes[*parse_index..*parse_index + 4]).unwrap();
        *parse_index += 4;
        i32::from_be_bytes(*x)
    }
}

impl Xdr for u32 {
    fn serialize(&self) -> std::vec::Vec<u8> {
        self.to_be_bytes().to_vec()
    }

    fn deserialize(bytes: &Vec<u8>, parse_index: &mut usize) -> u32 {
        let x = <&[u8; 4]>::try_from(&bytes[*parse_index..*parse_index + 4]).unwrap();
        *parse_index += 4;
        u32::from_be_bytes(*x)
    }
}

impl Xdr for u64 {
    fn serialize(&self) -> std::vec::Vec<u8> {
        self.to_be_bytes().to_vec()
    }

    fn deserialize(bytes: &Vec<u8>, parse_index: &mut usize) -> u64 {
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

    fn deserialize(bytes: &Vec<u8>, parse_index: &mut usize) -> String {
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

    fn deserialize(bytes: &Vec<u8>, parse_index: &mut usize) -> f32 {
        let x = <&[u8; 4]>::try_from(&bytes[*parse_index..*parse_index + 4]).unwrap();
        *parse_index += 4;
        f32::from_be_bytes(*x)
    }
}

impl Xdr for f64 {
    fn serialize(&self) -> std::vec::Vec<u8> {
        self.to_be_bytes().to_vec()
    }

    fn deserialize(bytes: &Vec<u8>, parse_index: &mut usize) -> f64 {
        let x = <&[u8; 8]>::try_from(&bytes[*parse_index..*parse_index + 8]).unwrap();
        *parse_index += 8;
        f64::from_be_bytes(*x)
    }
}
