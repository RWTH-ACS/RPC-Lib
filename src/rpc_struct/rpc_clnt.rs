// Copyright 2022 Philipp Fensch
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::io::{self, prelude::*};
use std::net::TcpStream;
use std::vec::Vec;

use super::xdr::*;
use std::convert::TryFrom;
use std::io::*;

struct Rpcb {
    program: u32,
    version: u32,
    netid: String,
    address: String,
    owner: String,
}

impl Xdr for Rpcb {
    fn serialize(&self, mut writer: impl Write) -> io::Result<()> {
        self.program.serialize(&mut writer)?;
        self.version.serialize(&mut writer)?;
        self.netid.serialize(&mut writer)?;
        self.address.serialize(&mut writer)?;
        self.owner.serialize(&mut writer)?;
        Ok(())
    }

    fn deserialize(bytes: &[u8], parse_index: &mut usize) -> Rpcb {
        Rpcb {
            program: u32::deserialize(bytes, parse_index),
            version: u32::deserialize(bytes, parse_index),
            netid: String::deserialize(bytes, parse_index),
            address: String::deserialize(bytes, parse_index),
            owner: String::deserialize(bytes, parse_index),
        }
    }
}

#[derive(Debug)]
struct FragmentHeader {
    last_fragment: bool,
    length: u32,
}

impl Xdr for FragmentHeader {
    fn serialize(&self, writer: impl Write) -> io::Result<()> {
        if self.last_fragment {
            (self.length + (1 << 31)).serialize(writer)?;
        } else {
            self.length.serialize(writer)?;
        }
        Ok(())
    }

    fn deserialize(bytes: &[u8], parse_index: &mut usize) -> FragmentHeader {
        let x = <&[u8; 4]>::try_from(&bytes[*parse_index..*parse_index + 4]).unwrap();
        *parse_index += 4;
        let num = u32::from_be_bytes(*x);
        // First Bit: Last Fragment Flag, 31 following bits: Length
        FragmentHeader {
            last_fragment: num & 0x80000000u32 != 0,
            length: num & 0x7FFFFFFFu32,
        }
    }
}

#[derive(Debug)]
struct RpcCall {
    fragment_header: FragmentHeader,
    xid: u32,
    msg_type: u32, // (Call: 0, Reply: 1)
}

impl Xdr for RpcCall {
    fn serialize(&self, mut writer: impl Write) -> io::Result<()> {
        self.fragment_header.serialize(&mut writer)?;
        self.xid.serialize(&mut writer)?;
        self.msg_type.serialize(&mut writer)?;
        Ok(())
    }

    fn deserialize(bytes: &[u8], parse_index: &mut usize) -> RpcCall {
        RpcCall {
            fragment_header: FragmentHeader::deserialize(bytes, parse_index),
            xid: u32::deserialize(bytes, parse_index),
            msg_type: u32::deserialize(bytes, parse_index),
        }
    }
}

struct RpcRequest {
    header: RpcCall,
    rpc_version: u32,
    program_num: u32,
    version_num: u32,
    proc_num: u32,
    credentials: u64,
    verifier: u64,
}

impl Xdr for RpcRequest {
    fn serialize(&self, mut writer: impl Write) -> io::Result<()> {
        self.header.serialize(&mut writer)?;
        self.rpc_version.serialize(&mut writer)?;
        self.program_num.serialize(&mut writer)?;
        self.version_num.serialize(&mut writer)?;
        self.proc_num.serialize(&mut writer)?;
        self.credentials.serialize(&mut writer)?;
        self.verifier.serialize(&mut writer)?;
        Ok(())
    }

    fn deserialize(bytes: &[u8], parse_index: &mut usize) -> RpcRequest {
        RpcRequest {
            header: RpcCall::deserialize(bytes, parse_index),
            rpc_version: u32::deserialize(bytes, parse_index),
            program_num: u32::deserialize(bytes, parse_index),
            version_num: u32::deserialize(bytes, parse_index),
            proc_num: u32::deserialize(bytes, parse_index),
            credentials: u64::deserialize(bytes, parse_index),
            verifier: u64::deserialize(bytes, parse_index),
        }
    }
}

#[derive(Debug)]
struct RpcReply {
    header: RpcCall,
    reply_state: u32,
    verifier: u64,
    accept_state: u32,
    // Serialized Data (Return Value of RPC-Procedure)
}

impl Xdr for RpcReply {
    fn serialize(&self, mut writer: impl Write) -> io::Result<()> {
        self.header.serialize(&mut writer)?;
        self.reply_state.serialize(&mut writer)?;
        self.verifier.serialize(&mut writer)?;
        self.accept_state.serialize(&mut writer)?;
        Ok(())
    }

    fn deserialize(bytes: &[u8], parse_index: &mut usize) -> RpcReply {
        RpcReply {
            header: RpcCall::deserialize(bytes, parse_index),
            reply_state: u32::deserialize(bytes, parse_index),
            verifier: u64::deserialize(bytes, parse_index),
            accept_state: u32::deserialize(bytes, parse_index),
        }
    }
}

#[derive(Debug)]
struct UniversalAddress {
    ip: [u8; 4],
    port: u16,
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

impl UniversalAddress {
    // Format: xxx.xxx.xxx.xxx.xxx.xxx
    fn from_string(s: &str) -> UniversalAddress {
        let splitted = s.split('.').collect::<Vec<&str>>();
        let mut ret = UniversalAddress {
            ip: [0, 0, 0, 0],
            port: 0,
        };
        for i in 0..4 {
            ret.ip[i] = splitted[i].parse::<u8>().unwrap();
        }
        ret.port = splitted[4].parse::<u16>().unwrap() * 256;
        ret.port += splitted[5].parse::<u16>().unwrap();
        ret
    }

    // Format: xxx.xxx.xxx.xxx:xxxxx
    fn to_string(&self) -> String {
        let string_repr = std::format!(
            "{}.{}.{}.{}:{}",
            self.ip[0],
            self.ip[1],
            self.ip[2],
            self.ip[3],
            self.port,
        );
        string_repr
    }
}

// Create Client
pub fn clnt_create(address: &str, program: u32, version: u32) -> Result<RpcClient> {
    let univ_addr_portmap = UniversalAddress::from_string(&(String::from(address) + ".0.111")); // Port of Portmap Service in universal address format
    let mut client = RpcClient {
        program: 100000,
        version: 4,
        stream: TcpStream::connect(univ_addr_portmap.to_string())?,
    };

    let rpcb = Rpcb {
        program,
        version,
        netid: String::from("tcp"),
        address: String::from(address) + ".0.111", // Port of Portmap Service in universal address format
        owner: String::from("rpclib"),
    };

    // Proc 3: GETADDR
    let send = {
        let mut send = Vec::new();
        rpcb.serialize(&mut send)?;
        send
    };
    let vec = rpc_call(&mut client, 3, &send)?;

    // Parse Universal Address & Convert to Standard IP-Format
    let mut parse_index = 0;
    let universal_address_s = String::deserialize(&vec, &mut parse_index);
    if universal_address_s.len() == 0 {
        return Err(Error::new(
            ErrorKind::Other,
            "clnt_create: Rpc-Server not available",
        ));
    }
    let ip = UniversalAddress::from_string(&universal_address_s);

    // Create TcpStream
    let stream = TcpStream::connect(ip.to_string())?;

    Ok(RpcClient {
        program,
        version,
        stream,
    })
}

pub fn rpc_call(client: &mut RpcClient, procedure: u32, send: &[u8]) -> Result<Vec<u8>> {
    send_rpc_request(client, procedure, send)?;
    receive_rpc_reply(client)
}

fn send_rpc_request(client: &mut RpcClient, procedure: u32, send_data: &[u8]) -> Result<()> {
    const REQUEST_HEADER_LEN: usize = 40;
    let length = REQUEST_HEADER_LEN + send_data.len();

    // println!("[Rpc-Lib] Request Procedure: {}", procedure);
    let request = RpcRequest {
        header: RpcCall {
            fragment_header: FragmentHeader {
                last_fragment: true, // Only one Fragment
                length: length as u32,
            },
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
    client.stream.write_all(&*send_data)?;
    Ok(())
}

fn receive_rpc_reply(client: &mut RpcClient) -> Result<Vec<u8>> {
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
    Ok(vec)
}

fn receive_reply_packet(
    client: &mut RpcClient,
    buffer: &mut Vec<u8>,
    header_len: usize,
) -> Result<bool> {
    // Receive Header
    let mut header_buf = Vec::with_capacity(header_len);
    header_buf.resize(header_len, 0);
    client.stream.read_exact(&mut header_buf)?;
    let mut index: usize = 0;
    let (payload_length, last_fragment) = if header_len == 28 {
        let reply_header = RpcReply::deserialize(&header_buf, &mut index);
        (
            reply_header.header.fragment_header.length as usize - header_len + 4,
            reply_header.header.fragment_header.last_fragment,
        )
    } else {
        let fragment_header = FragmentHeader::deserialize(&header_buf, &mut index);
        (
            fragment_header.length as usize - header_len + 4,
            fragment_header.last_fragment,
        )
    };

    // Receive Reply-Data
    let old_len = buffer.len();
    let new_len = old_len + payload_length;
    buffer.resize(new_len, 0);
    client.stream.read_exact(&mut buffer[old_len..new_len])?;
    Ok(last_fragment)
}
