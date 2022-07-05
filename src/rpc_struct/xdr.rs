// Copyright 2022 Philipp Fensch
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::convert::TryInto;
use std::io::{self, Read, Write};
use std::mem;
use std::vec::Vec;

/// A data structure that can be serialized into the XDR format as described in [`RFC 4506`].
///
/// [`RFC 4506`]: <https://datatracker.ietf.org/doc/html/rfc4506>
pub trait XdrSerialize {
    /// Serialize this value into the given writer.
    fn serialize(&self, writer: impl Write) -> io::Result<()>;
}

/// A data structure that can be deserialized from the XDR format as described in [`RFC 4506`].
///
/// [`RFC 4506`]: <https://datatracker.ietf.org/doc/html/rfc4506>
pub trait XdrDeserialize: Sized {
    /// Deserialize this value from the given reader.
    fn deserialize(reader: impl Read) -> io::Result<Self>;
}

fn padding(len: usize) -> usize {
    (4 - len % 4) % 4
}

impl<T: XdrSerialize + ?Sized> XdrSerialize for &T {
    #[inline]
    fn serialize(&self, writer: impl Write) -> io::Result<()> {
        (**self).serialize(writer)
    }
}

/// Fixed-Length Opaque Data
impl<const LEN: usize> XdrSerialize for [u8; LEN] {
    fn serialize(&self, mut writer: impl Write) -> io::Result<()> {
        writer.write_all(self)?;
        writer.write_all(&[0u8; 3][..padding(LEN)])
    }
}

impl<const LEN: usize> XdrDeserialize for [u8; LEN] {
    fn deserialize(mut reader: impl Read) -> io::Result<Self> {
        let mut this = [0; LEN];
        reader.read_exact(&mut this)?;
        Ok(this)
    }
}

/// Variable-Length Opaque Data
impl XdrSerialize for Vec<u8> {
    fn serialize(&self, mut writer: impl Write) -> io::Result<()> {
        (self.len() as u32).serialize(&mut writer)?;
        writer.write_all(self)?;
        writer.write_all(&[0u8; 3][..padding(self.len())])
    }
}

impl XdrDeserialize for Vec<u8> {
    fn deserialize(mut reader: impl Read) -> io::Result<Self> {
        let len = u32::deserialize(&mut reader)? as usize;
        let mut this = vec![0; len];
        reader.read_exact(&mut this)?;
        Ok(this)
    }
}

/// Fixed-Length Array
impl<T: XdrSerialize, const LEN: usize> XdrSerialize for [T; LEN] {
    fn serialize(&self, mut writer: impl Write) -> io::Result<()> {
        for item in self {
            item.serialize(&mut writer)?;
        }
        Ok(())
    }
}

impl<T: XdrDeserialize, const LEN: usize> XdrDeserialize for [T; LEN] {
    fn deserialize(mut reader: impl Read) -> io::Result<Self> {
        let mut vec = Vec::with_capacity(LEN);
        for _ in 0..LEN {
            vec.push(T::deserialize(&mut reader)?);
        }
        vec.try_into().map_err(|_| unreachable!())
    }
}

/// Variable-Length Array
impl<T: XdrSerialize> XdrSerialize for Vec<T> {
    fn serialize(&self, mut writer: impl Write) -> io::Result<()> {
        (self.len() as u32).serialize(&mut writer)?;
        for item in self {
            item.serialize(&mut writer)?;
        }
        Ok(())
    }
}

impl<T: XdrDeserialize> XdrDeserialize for Vec<T> {
    fn deserialize(mut reader: impl Read) -> io::Result<Self> {
        let len = u32::deserialize(&mut reader)? as usize;
        let mut this = Vec::with_capacity(len);
        for _ in 0..len {
            this.push(T::deserialize(&mut reader)?);
        }
        Ok(this)
    }
}

impl XdrSerialize for String {
    fn serialize(&self, mut writer: impl Write) -> io::Result<()> {
        assert!(self.is_ascii());
        (self.len() as u32).serialize(&mut writer)?;
        writer.write_all(self.as_bytes())?;
        writer.write_all(&[0u8; 3][..padding(self.len())])
    }
}

impl XdrDeserialize for String {
    fn deserialize(reader: impl Read) -> io::Result<Self> {
        let vec = Vec::<u8>::deserialize(reader)?;
        Ok(Self::from_utf8(vec).unwrap())
    }
}

macro_rules! impl_xdr_be_bytes {
    ($Ty:ty) => {
        impl XdrSerialize for $Ty {
            fn serialize(&self, mut writer: impl Write) -> io::Result<()> {
                writer.write_all(&self.to_be_bytes())
            }
        }

        impl XdrDeserialize for $Ty {
            fn deserialize(mut reader: impl Read) -> io::Result<Self> {
                let mut buf = [0; mem::size_of::<Self>()];
                reader.read_exact(&mut buf)?;
                Ok(Self::from_be_bytes(buf))
            }
        }
    };
}

impl_xdr_be_bytes!(u32);
impl_xdr_be_bytes!(u64);
impl_xdr_be_bytes!(i32);
impl_xdr_be_bytes!(i64);
impl_xdr_be_bytes!(f32);
impl_xdr_be_bytes!(f64);
