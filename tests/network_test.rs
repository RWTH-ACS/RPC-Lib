extern crate rpc_lib;
use rpc_lib::include_rpcl;

#[include_rpcl("tests/math.x")]
struct RPCConnection;


#[test]
fn addition() {
    let rpc_connection = RPCConnection::new("127.0.0.1");

    let value = rpc_connection.ADD(2, 3);
    assert!(value == 5, "Addition failed");
}
