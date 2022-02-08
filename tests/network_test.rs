extern crate rpc_lib;
use rpc_lib::include_rpcl;

#[include_rpcl("tests/math.x")]
struct RPCConnection;


#[test]
fn addition() {
    let mut rpc_connection = RPCConnection::new("127.0.0.1").expect("Failed to connect to Rpc-Server");

    let value = rpc_connection.ADD(&2, &3).expect("Rpc-Call failed");
    assert!(value == 5, "Addition failed");
}
