mod rpc_clnt;
pub mod xdr;

pub use self::rpc_clnt::{clnt_create, rpc_call, RpcClient, RpcReply};
