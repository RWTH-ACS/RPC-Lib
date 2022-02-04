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
    fn serialize(&self) -> Vec<u8> {
        let mut vec = Vec::new();
        if self.last_fragment {
            vec.extend((self.length + (1 << 31)).serialize());
        } else {
            vec.extend(self.length.serialize());
        }
        vec
    }

    fn deserialize(bytes: &Vec<u8>, parse_index: &mut usize) -> FragmentHeader {
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
    fn serialize(&self) -> Vec<u8> {
        let mut vec = Vec::new();
        vec.extend(self.fragment_header.serialize());
        vec.extend(self.xid.serialize());
        vec.extend(self.msg_type.serialize());
        vec
    }

    fn deserialize(bytes: &Vec<u8>, parse_index: &mut usize) -> RpcCall {
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
    fn serialize(&self) -> Vec<u8> {
        let mut vec = Vec::new();
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
pub struct RpcReply {
    header: RpcCall,
    reply_state: u32,
    verifier: u64,
    accept_state: u32,
    // Serialized Data (Return Value of RPC-Procedure)
}

impl Xdr for RpcReply {
    fn serialize(&self) -> Vec<u8> {
        let mut vec = Vec::new();
        vec.extend(self.header.serialize());
        vec.extend(self.reply_state.serialize());
        vec.extend(self.verifier.serialize());
        vec.extend(self.accept_state.serialize());
        vec
    }

    fn deserialize(bytes: &Vec<u8>, parse_index: &mut usize) -> RpcReply {
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

#[derive(Debug)]
pub struct RpcClient {
    program: u32,
    version: u32,
    stream: TcpStream,
}

impl UniversalAddress {
    // Format: xxx.xxx.xxx.xxx.xxx.xxx
    fn from_string(s: &String) -> UniversalAddress {
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
            self.ip[0].to_string(),
            self.ip[1].to_string(),
            self.ip[2].to_string(),
            self.ip[3].to_string(),
            self.port.to_string(),
        );
        string_repr
    }
}

// Create Client
pub fn clnt_create(address: &str, program: u32, version: u32) -> RpcClient {
    let univ_addr_portmap = UniversalAddress::from_string(&(String::from(address) + ".0.111")); // Port of Portmap Service in universal address format
    let mut client = RpcClient {
        program: 100000,
        version: 4,
        stream: TcpStream::connect(univ_addr_portmap.to_string()).expect("rpc_call: Failed to connect"),
    };

    let rpcb = Rpcb {
        program: program,
        version: version,
        netid: String::from("tcp"),
        address: String::from(address) + ".0.111", // Port of Portmap Service in universal address format
        owner: String::from("rpclib"),
    };

    // Proc 3: GETADDR
    let vec = rpc_call(&mut client, 3, &rpcb.serialize());

    // Parse Universal Address & Convert to Standard IP-Format
    let mut parse_index = 0;
    let universal_address_s = String::deserialize(&vec, &mut parse_index);
    let ip = UniversalAddress::from_string(&universal_address_s);

    // Create TcpStream
    let stream = TcpStream::connect(ip.to_string()).expect("rpc_call: Failed to connect");

    RpcClient {
        program: program,
        version: version,
        stream: stream,
    }
}

// Rpc-Call
pub fn rpc_call(client: &mut RpcClient, procedure: u32, send: &Vec<u8>) -> Vec<u8> {
    const REQUEST_HEADER_LEN: usize = 40;
    let length = REQUEST_HEADER_LEN + send.len();

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

    // Connect

    // Send Request
    let request_header = request.serialize();
    client.stream.write(&request_header).expect("rpc_call: Failed to send data");
    client.stream.write(&*send).expect("rpc_call: Failed to send data");

    // Receive Header
    let mut header_buf: [u8; 28] = [0; 28];
    let rec = client.stream.read(&mut header_buf).expect("rpc_call: Failed to receive data");
    assert!(rec == 28, "rpc_call: Failed to receive data");
    let mut index = 0usize;
    let reply_header = RpcReply::deserialize(&header_buf.to_vec(), &mut index);
    let reply_length = reply_header.header.fragment_header.length as usize;
    // println!("  Reply Length (Header-Field): {} Actual Header length: {} Actually read {} bytes", reply_length, index, rec);

    // Receive Reply-Data
    let mut vec = Vec::with_capacity(reply_length - 24);
    unsafe{ vec.set_len(reply_length - 24); }
    let rec = client.stream.read(vec.as_mut_slice()).expect("rpc_call: Failed to receive data");
    assert!(rec == reply_length - 24, "rpc_call: Failed to receive data");
    // println!("  Read {} bytes Reply-Data", rec);
    vec
}
