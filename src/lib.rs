// Copyright 2022 Philipp Fensch
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! # Rpc-lib
//!
//! Rpc-lib is a crate to compile an RPC-Definition in the ONC-RPC format ([`RFC 5531`]) into Rust-code. It comes
//! with the necessary network-code to execute the RPC-Calls and to serialize and deserialize the
//! data.
//!
//! [`RFC 5531`]: https://datatracker.ietf.org/doc/html/rfc5531
//!
//! # Example
//!
//! Creates a connection to 127.0.0.1, makes an Rpc-Call and prints the result.
//! ```rust
//! use rpc_lib::include_rpcl;
//!
//! #[include_rpcl("my_rpcl_file.x")]
//! struct RPCStruct;
//!
//! let mut rpc = RPCStruct::new("127.0.0.1").expect("Can't connect to server");
//! let result = rpc.MY_RPC_PROCEDURE(&1, &2).expect("Rpc call failed");
//! println!("MY_RPC_PROCEDURE returned: {}", result);
//! ```
#![forbid(unsafe_code)]
#![warn(rust_2018_idioms)]

mod rpc_struct;

/// Reads file and generates Rustcode according to contents
///
/// # Examples
/// Reads `my_file.x` and adds associated functions to `MyStruct` according to procedure-definitions in
/// `my_file.x`
/// ```
/// #[include_rpcl("my_file.x")]
/// struct MyStruct;
/// ```
pub use rpc_lib_derive::include_rpcl;

pub use crate::rpc_struct::clnt_create;
pub use crate::rpc_struct::RpcClient;

pub use crate::rpc_struct::xdr::*;
