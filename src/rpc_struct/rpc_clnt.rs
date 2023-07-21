// Copyright 2022 Philipp Fensch
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::fmt;
use std::io::{self, BufReader, BufWriter, ErrorKind, Read, Write};
use std::net::{AddrParseError, IpAddr, SocketAddr, TcpStream};
use std::str::FromStr;

use crate::{XdrDeserialize, XdrSerialize};

#[derive(XdrSerialize, XdrDeserialize)]
struct Rpcb {
    program: u32,
    version: u32,
    netid: String,
    address: String,
    owner: String,
}

#[derive(XdrSerialize, XdrDeserialize, Debug)]
struct FragmentHeader {
    number: u32,
}

impl FragmentHeader {
    const LAST_FLAG: u32 = 1 << (u32::BITS - 1);

    fn new(last: bool, len: u32) -> Self {
        assert!(len <= u32::MAX >> 1);
        let mut number = len;
        if last {
            number |= Self::LAST_FLAG; // insert
        }
        Self { number }
    }

    fn len(&self) -> u32 {
        let mut len = self.number;
        len &= !Self::LAST_FLAG; // remove
        len
    }
}

#[derive(XdrSerialize, XdrDeserialize, Debug)]
struct RpcCall {
    xid: u32,
    msg_type: u32, // (Call: 0, Reply: 1)
}

#[derive(XdrSerialize, XdrDeserialize)]
struct RpcRequest {
    header: RpcCall,
    rpc_version: u32,
    program_num: u32,
    version_num: u32,
    proc_num: u32,
    credentials: u64,
    verifier: u64,
}

#[derive(XdrSerialize, XdrDeserialize, Debug)]
struct RpcReply {
    header: RpcCall,
    reply_state: u32,
    verifier: u64,
    accept_state: u32,
    // Serialized Data (Return Value of RPC-Procedure)
}

/// Universal Address
///
/// Defined in [RFC 3530](https://www.rfc-editor.org/rfc/rfc3530)
#[derive(Debug)]
struct UniversalAddr(SocketAddr);

impl From<SocketAddr> for UniversalAddr {
    fn from(socket_addr: SocketAddr) -> Self {
        Self(socket_addr)
    }
}

impl fmt::Display for UniversalAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ip = self.0.ip();
        let [o1, o2] = self.0.port().to_be_bytes();
        write!(f, "{ip}.{o1}.{o2}")
    }
}

impl FromStr for UniversalAddr {
    type Err = AddrParseError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let mut split = s.rsplitn(3, '.');
        let o2 = split.next().unwrap().parse().unwrap();
        let o1 = split.next().unwrap().parse().unwrap();
        let port = u16::from_be_bytes([o1, o2]);
        let ip = split.next().unwrap();
        Ok(Self(SocketAddr::new(ip.parse()?, port)))
    }
}

/// Contains required fields to make RPC-Calls.
///
/// Consists of:
///  - An already connected [`TcpStream`]
///  - Program-Number (as defined in RPCL-File)
///  - Version-Number (as defined in RPCL-File)
#[derive(Debug)]
pub struct RpcClient {
    program: u32,
    version: u32,
    reader: BufReader<TcpStream>,
    writer: BufWriter<TcpStream>,
}

const BUF_SIZE: usize = 256;

// Create Client
pub fn clnt_create(ip: IpAddr, program: u32, version: u32) -> io::Result<RpcClient> {
    let portmap_port = 111;
    let portmap_addr = SocketAddr::new(ip, portmap_port);
    let tcp_stream = TcpStream::connect(portmap_addr)?;
    let mut client = RpcClient {
        program: 100000,
        version: 4,
        reader: BufReader::with_capacity(BUF_SIZE, tcp_stream.try_clone()?),
        writer: BufWriter::with_capacity(BUF_SIZE, tcp_stream),
    };

    let rpcb = Rpcb {
        program,
        version,
        netid: String::from("tcp"),
        address: UniversalAddr::from(portmap_addr).to_string(),
        owner: String::from("rpclib"),
    };

    // Proc 3: GETADDR
    let universal_address_s: String = client.call(3, &rpcb)?;

    // Convert Universal Address to Standard IP-Format
    if universal_address_s.is_empty() {
        return Err(io::Error::new(
            ErrorKind::Other,
            "clnt_create: Rpc-Server not available",
        ));
    }
    let addr = UniversalAddr::from_str(&universal_address_s).unwrap();

    // Create TcpStream
    let tcp_stream = TcpStream::connect(addr.0)?;

    Ok(RpcClient {
        program,
        version,
        reader: BufReader::with_capacity(BUF_SIZE, tcp_stream.try_clone()?),
        writer: BufWriter::with_capacity(BUF_SIZE, tcp_stream),
    })
}

impl RpcClient {
    pub fn peer_addr(&self) -> io::Result<SocketAddr> {
        self.reader.get_ref().peer_addr()
    }

    pub fn call<T: XdrDeserialize>(
        &mut self,
        procedure: u32,
        args: impl XdrSerialize,
    ) -> io::Result<T> {
        self.send_request(procedure, args)?;
        self.recv()
    }

    /// Makes a RPC call. Doesn't processes the response but writes it into `resp`.
    pub fn call_with_raw_union_response<'a>(
        &mut self,
        procedure: u32,
        args: impl XdrSerialize,
        resp: &'a mut RawResponseUnion<'a, i32>,
    ) -> io::Result<()> {
        self.send_request(procedure, args)?;
        self.recv_raw_union(resp)
    }

    fn send_request(&mut self, procedure: u32, args: impl XdrSerialize) -> io::Result<()> {
        let request = RpcRequest {
            header: RpcCall {
                xid: 123456, // Random but unique number
                msg_type: 0, // Type: Call
            },
            rpc_version: 2,
            program_num: self.program,
            version_num: self.version,
            proc_num: procedure,
            credentials: 0, // No authentification
            verifier: 0,
        };

        let length = request.len() + args.len();
        let fragment_header = FragmentHeader::new(true, length.try_into().unwrap());

        fragment_header.serialize(&mut self.writer)?;
        request.serialize(&mut self.writer)?;
        args.serialize(&mut self.writer)?;
        self.writer.flush()?;

        Ok(())
    }

    fn recv<T: XdrDeserialize>(&mut self) -> io::Result<T> {
        let mut reader = FragmentReader::new(&mut self.reader);
        let _rpc_reply = RpcReply::deserialize(&mut reader)?;
        XdrDeserialize::deserialize(&mut reader)
    }

    fn recv_raw_union<'a>(&mut self, target: &'a mut RawResponseUnion<'a, i32>) -> io::Result<()> {
        // TODO: This is very crude and needs improvements
        let mut reader = FragmentReader::new(&mut self.reader);
        let _rpc_reply = RpcReply::deserialize(&mut reader)?;
        let discriminant = i32::deserialize(&mut reader)?;
        *target.discriminant = discriminant;
        let data_len_internal = i32::deserialize(&mut reader)?;
        reader.read_exact(target.data)?;
        assert_eq!(data_len_internal as usize, target.data.len());
        Ok(())
    }
}

#[derive(Debug)]
/// Raw data from a RPC response. Used for zero-copy responses.
pub struct RawResponseUnion<'a, DISCRIMINANT> {
    pub discriminant: &'a mut DISCRIMINANT,
    pub data_length: &'a mut usize,
    pub data: &'a mut [u8],
}

struct FragmentReader<R> {
    inner: R,
    nleft: u32,
}

impl<R: Read> FragmentReader<R> {
    fn new(inner: R) -> Self {
        Self { inner, nleft: 0 }
    }
}

impl<R: Read> Read for FragmentReader<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.nleft == 0 {
            let fragment_header = FragmentHeader::deserialize(&mut self.inner)?;
            self.nleft = fragment_header.len();
        }

        let nread = self.inner.by_ref().take(self.nleft.into()).read(buf)?;
        self.nleft -= nread as u32;
        Ok(nread)
    }

    fn read_exact(&mut self, mut buf: &mut [u8]) -> io::Result<()> {
        while buf.len() > self.nleft as usize {
            self.inner.read_exact(&mut buf[..self.nleft as usize])?;
            buf = &mut buf[self.nleft as usize..];
            let fragment_header = FragmentHeader::deserialize(&mut self.inner)?;
            self.nleft = fragment_header.len();
        }

        self.nleft -= buf.len() as u32;
        self.inner.read_exact(buf)
    }
}
