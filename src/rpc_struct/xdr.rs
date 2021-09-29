pub trait Xdr {
    // Serializes data and converts to network byte order
    fn serialize(&self) -> std::vec::Vec<u8>;
    // Reverse Operation
    fn deserialize(bytes: &Vec<u8>, parse_index: &mut usize) -> Self;
}

use std::convert::{TryFrom, TryInto};

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
        let s = String::from_utf8(bytes[*parse_index..*parse_index+len].to_vec()).unwrap();
        let len_and_padding = ((len + 4) / 4) * 4;
        *parse_index += len_and_padding;
        s
    }
}
