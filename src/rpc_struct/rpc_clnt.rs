use std::io::prelude::*;
use std::net::TcpStream;
use std::vec::Vec;

use super::xdr::*;
use std::convert::TryFrom;

struct Rpcb {
    program: u32,
    version: u32,
    netid: String,
    address: String,
    owner: String,
}

impl Xdr for Rpcb {
    fn serialize(&self) -> Vec<u8> {
        let mut vec = Vec::new();
        vec.extend(self.program.serialize());
        vec.extend(self.version.serialize());
        vec.extend(self.netid.serialize());
        vec.extend(self.address.serialize());
        vec.extend(self.owner.serialize());
        vec
    }

    fn deserialize(bytes: &Vec<u8>, parse_index: &mut usize) -> Rpcb {
        Rpcb {
            program: u32::deserialize(bytes, parse_index),
            version: u32::deserialize(bytes, parse_index),
            netid:   String::deserialize(bytes, parse_index),
            address: String::deserialize(bytes, parse_index),
            owner:   String::deserialize(bytes, parse_index),
        }
    }
}

#[derive(Debug)]
struct FragmentHeader {
    last_fragment: bool,
    length:        u32,
}

impl Xdr for FragmentHeader {
    fn serialize(&self) -> Vec<u8> {
        let mut vec = Vec::new();
        if self.last_fragment {
            vec.extend((self.length + (1 << 31)).serialize());
        }
        else {
            vec.extend(self.length.serialize());
        }
        vec
    }

    fn deserialize(bytes: &Vec<u8>, parse_index: &mut usize) -> FragmentHeader {
        let x = <&[u8; 4]>::try_from(&bytes[*parse_index..*parse_index + 4]).unwrap();
        *parse_index += 4;
        let num = u32::from_be_bytes(*x);
        FragmentHeader {
            last_fragment: (1 << 31) & num != 0,
            length:        num - (1 << 31),
        }
    }
}

#[derive(Debug)]
struct RpcCall {
    fragment_header: FragmentHeader,
    xid:             u32,
    msg_type:        u32, // (Call: 0, Replay: 1)
}

impl Xdr for RpcCall {
    fn serialize(&self) -> Vec<u8> {
        let mut vec = Vec::new();
        vec.extend(self.fragment_header.serialize());   // TODO
        vec.extend(self.xid.serialize());
        vec.extend(self.msg_type.serialize());
        vec
    }

    fn deserialize(bytes: &Vec<u8>, parse_index: &mut usize) -> RpcCall {
        RpcCall {
            fragment_header: FragmentHeader::deserialize(bytes, parse_index),
            xid:             u32::deserialize(bytes, parse_index),
            msg_type:        u32::deserialize(bytes, parse_index),
        }
    }
}

struct RpcRequest {
    header:      RpcCall,
    rpc_version: u32,
    program_num: u32,
    version_num: u32,
    proc_num:    u32,
    credentials: u64,
    verifier:    u64,
}

impl Xdr for RpcRequest {
    fn serialize(&self) -> std::vec::Vec<u8> {
        let mut vec = std::vec::Vec::new();
        vec.extend(self.header.serialize());
        vec.extend(self.rpc_version.serialize());
        vec.extend(self.program_num.serialize());
        vec.extend(self.version_num.serialize());
        vec.extend(self.proc_num.serialize());
        vec.extend(self.credentials.serialize());
        vec.extend(self.verifier.serialize());
        vec
    }

    fn deserialize(bytes: &Vec<u8>, parse_index: &mut usize) -> RpcRequest {
        RpcRequest {
            header:      RpcCall::deserialize(bytes, parse_index),
            rpc_version: u32::deserialize(bytes, parse_index),
            program_num: u32::deserialize(bytes, parse_index),
            version_num: u32::deserialize(bytes, parse_index),
            proc_num:    u32::deserialize(bytes, parse_index),
            credentials: u64::deserialize(bytes, parse_index),
            verifier:    u64::deserialize(bytes, parse_index),
        }
    }
}

#[derive(Debug)]
struct RpcReply {
    header:       RpcCall,
    reply_state:  u32,
    verifier:     u64,
    accept_state: u32,
    // Serialized Data (Return Value of RPC-Procedure)
}

impl Xdr for RpcReply {
    fn serialize(&self) -> std::vec::Vec<u8> {
        let mut vec = std::vec::Vec::new();
        vec.extend(self.header.serialize());
        vec.extend(self.reply_state.serialize());
        vec.extend(self.verifier.serialize());
        vec.extend(self.accept_state.serialize());
        vec
    }

    fn deserialize(bytes: &Vec<u8>, parse_index: &mut usize) -> RpcReply {
        RpcReply {
            header:       RpcCall::deserialize(bytes, parse_index),
            reply_state:  u32::deserialize(bytes, parse_index),
            verifier:     u64::deserialize(bytes, parse_index),
            accept_state: u32::deserialize(bytes, parse_index),
        }
    }
}

#[derive(Debug)]
struct UniversalAddress {
    ip: [u8; 4],
    port: u16,
}

#[derive(Debug)]
pub struct RpcClient {
    program:   u32,
    version:   u32,
    univ_addr: UniversalAddress,
}

impl UniversalAddress {
    fn from_string(s: &String) -> UniversalAddress {
        let splitted = s.split('.').collect::<Vec<&str>>();
        let mut ret = UniversalAddress{ ip: [0, 0, 0, 0], port: 0};
        for i in 0..4 {
            ret.ip[i] = splitted[i].parse::<u8>().unwrap();
        }
        ret.port = splitted[4].parse::<u16>().unwrap() * 256;
        ret.port += splitted[5].parse::<u16>().unwrap();
        ret
    }

    fn to_string(&self) -> String {
        String::new()
    }
}

// Create Client
pub fn clnt_create(address: &str, program: u32, version: u32) -> RpcClient {

    let request = RpcRequest {
        header: RpcCall {
            fragment_header: FragmentHeader {
                last_fragment: true,    // Only one Fragment
                length:        96,
            },
            xid:               123456,  // Random but unique number
            msg_type:          0,       // Type: Call
        },
        rpc_version:           2,
        program_num:           100000,  // Portmap
        version_num:           4,
        proc_num:              3,       // GETADDR
        credentials:           0,       // No authentification
        verifier:              0,
    };

    let rpcb = Rpcb {
        program: program,
        version: version,
        netid: String::from("tcp"),
        address: String::from(address) + ".0.111", // Port of Portmap Service in universal address format
        owner: String::from("rpclib"),
    };

    let mut stream = TcpStream::connect(&(address.to_owned() + ":111")).expect("Failed to connect");

    // Send Request to Portmapper
    let bytes1 = request.serialize();
    let bytes2 = rpcb.serialize();
    stream.write(&bytes1).expect("Failed to query Port");
    stream.write(&bytes2).expect("Failed to query Port");

    // Receive Reply from Portmapper
    let mut buf: [u8; 256] = [0; 256];
    let rec = stream.read(&mut buf).expect("Failed to query Port");
    let mut vec = Vec::new();
    vec.extend_from_slice(&buf[0..rec]);

    // Parse Result
    let mut parse_index = 0;
    // Response Header
    let _response = RpcReply::deserialize(&vec, &mut parse_index);
    // Universal Address
    let universal_address_s = String::deserialize(&vec, &mut parse_index);
    let addr = UniversalAddress::from_string(&universal_address_s);

    RpcClient {
        program:   program,
        version:   version,
        univ_addr: addr,
    }
}
