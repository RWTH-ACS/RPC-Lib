// Copyright 2022 Philipp Fensch
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::net::{AddrParseError, IpAddr, TcpStream};
use std::str::FromStr;
use std::vec::Vec;
use std::{io::prelude::*, net::SocketAddr};

use rpc_lib_derive::{XdrDeserialize, XdrSerialize};

use super::xdr::*;
use std::{fmt, io::*};

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

    fn is_last(&self) -> bool {
        (self.number & Self::LAST_FLAG) == Self::LAST_FLAG // contains
    }
}

#[derive(XdrSerialize, XdrDeserialize, Debug)]
struct RpcCall {
    fragment_header: FragmentHeader,
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
    stream: TcpStream,
}

// Create Client
pub fn clnt_create(ip: IpAddr, program: u32, version: u32) -> Result<RpcClient> {
    let portmap_port = 111;
    let portmap_addr = SocketAddr::new(ip, portmap_port);
    let mut client = RpcClient {
        program: 100000,
        version: 4,
        stream: TcpStream::connect(portmap_addr)?,
    };

    let rpcb = Rpcb {
        program,
        version,
        netid: String::from("tcp"),
        address: UniversalAddr::from(portmap_addr).to_string(),
        owner: String::from("rpclib"),
    };

    // Proc 3: GETADDR
    let universal_address_s: String = rpc_call(&mut client, 3, &rpcb)?;

    // Convert Universal Address to Standard IP-Format
    if universal_address_s.is_empty() {
        return Err(Error::new(
            ErrorKind::Other,
            "clnt_create: Rpc-Server not available",
        ));
    }
    let addr = UniversalAddr::from_str(&universal_address_s).unwrap();

    // Create TcpStream
    let stream = TcpStream::connect(addr.0)?;

    Ok(RpcClient {
        program,
        version,
        stream,
    })
}

pub fn rpc_call<T: XdrDeserialize>(
    client: &mut RpcClient,
    procedure: u32,
    send: impl XdrSerialize,
) -> Result<T> {
    send_rpc_request(client, procedure, send)?;
    receive_rpc_reply(client)
}

fn send_rpc_request(
    client: &mut RpcClient,
    procedure: u32,
    send_data: impl XdrSerialize,
) -> Result<()> {
    const REQUEST_HEADER_LEN: usize = 40;
    let mut buf = Vec::new();
    send_data.serialize(&mut buf)?;
    let length = REQUEST_HEADER_LEN + buf.len();

    // println!("[Rpc-Lib] Request Procedure: {}", procedure);
    let request = RpcRequest {
        header: RpcCall {
            fragment_header: FragmentHeader::new(true, length as u32),
            xid: 123456, // Random but unique number
            msg_type: 0, // Type: Call
        },
        rpc_version: 2,
        program_num: client.program,
        version_num: client.version,
        proc_num: procedure,
        credentials: 0, // No authentification
        verifier: 0,
    };

    // Send Request
    request.serialize(&mut client.stream)?;
    client.stream.write_all(&buf)?;
    Ok(())
}

fn receive_rpc_reply<T: XdrDeserialize>(client: &mut RpcClient) -> Result<T> {
    // Packet-length: If the reply is split into multiple fragments,
    // there will only be the fragment-header
    //
    // FRAGMENT-HEADER | REPLY-HEADER | PAYLOAD
    //        4        |      24      |
    const FRAGMENT_HEADER_LEN: usize = 4;
    const REPLY_HEADER_LEN: usize = 24;
    let mut vec: Vec<u8> = Vec::new();
    // Receive first fragment
    let mut last_fragment =
        receive_reply_packet(client, &mut vec, FRAGMENT_HEADER_LEN + REPLY_HEADER_LEN)?;
    while !last_fragment {
        // Receive following fragments
        last_fragment = receive_reply_packet(client, &mut vec, FRAGMENT_HEADER_LEN)?;
    }
    XdrDeserialize::deserialize(&vec[..])
}

fn receive_reply_packet(
    client: &mut RpcClient,
    buffer: &mut Vec<u8>,
    header_len: usize,
) -> Result<bool> {
    // Receive Header
    let mut header_buf = vec![0; header_len];
    client.stream.read_exact(&mut header_buf)?;
    let (payload_length, last_fragment) = if header_len == 28 {
        let reply_header = RpcReply::deserialize(&header_buf[..])?;
        (
            reply_header.header.fragment_header.len() as usize - header_len + 4,
            reply_header.header.fragment_header.is_last(),
        )
    } else {
        let fragment_header = FragmentHeader::deserialize(&header_buf[..])?;
        (
            fragment_header.len() as usize - header_len + 4,
            fragment_header.is_last(),
        )
    };

    // Receive Reply-Data
    let old_len = buffer.len();
    let new_len = old_len + payload_length;
    buffer.resize(new_len, 0);
    client.stream.read_exact(&mut buffer[old_len..new_len])?;
    Ok(last_fragment)
}
