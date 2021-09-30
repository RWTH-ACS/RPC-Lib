extern crate quote;
extern crate rpc_lib_impl;

mod rpc_struct;

// include_rpcl
pub use rpc_lib_impl::include_rpcl;

// Common functions
pub use rpc_struct::clnt_create;
pub use rpc_struct::rpc_call;
pub use rpc_struct::RpcClient;
pub use rpc_struct::RpcReply;

pub use rpc_struct::xdr::*;
