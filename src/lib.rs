extern crate rpc_lib_impl;
extern crate quote;

mod rpc_struct;

// include_rpcl
pub use rpc_lib_impl::include_rpcl;

// Common functions
pub use rpc_struct::clnt_create;
pub use rpc_struct::RpcClient;
