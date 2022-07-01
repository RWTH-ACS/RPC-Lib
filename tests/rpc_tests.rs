extern crate rpc_lib;
use rpc_lib::include_rpcl;

#[include_rpcl("tests/test.x")]
struct RPCConnection;

macro_rules! create_con {
    () => {
        RPCConnection::new("127.0.0.1").expect("Failed to connect to Rpc-Server")
    };
}

#[test]
fn addition() {
    let mut rpc_connection = create_con!();
    let value = rpc_connection.ADD(&2, &3).expect("Rpc-Call failed");
    assert!(value == 5, "Add Test failed");
}

#[test]
fn struct_param() {
    let mut rpc_connection = create_con!();
    let my_struct = MyStruct { x: 2, y: -5 };
    let value = rpc_connection
        .STRUCT_MUL_FIELDS(&my_struct)
        .expect("Rpc-Call failed");
    assert!(value == -10, "Struct Test failed");
}

#[test]
fn struct_return() {
    let mut rpc_connection = create_con!();
    let value = rpc_connection
        .STRUCT_COMBINE(&5, &-20)
        .expect("Rpc-Call failed");
    assert!(value.x == 5 && value.y == -20, "Struct Test failed");
}

#[test]
fn union_return() {
    let mut rpc_connection = create_con!();
    let value = rpc_connection.UNION_TEST(&20).expect("Rpc-Call failed");
    assert!(
        match value {
            ResultUnion::Case0 { int_res: _ } => panic!("Wrong Type"),
            ResultUnion::Case20 { float_res } => {
                float_res
            }
            ResultUnion::CaseDefault => panic!("Wrong Type"),
        } == 1.0f32,
        "Union Test failed"
    );

    let value = rpc_connection.UNION_TEST(&0).expect("Rpc-Call failed");
    assert!(
        match value {
            ResultUnion::Case0 { int_res } => {
                int_res
            }
            ResultUnion::Case20 { float_res: _ } => panic!("Wrong Type"),
            ResultUnion::CaseDefault => panic!("Wrong Type"),
        } == 1i32,
        "Union Test failed"
    );
}

#[test]
fn union_param() {
    let mut rpc_con = create_con!();
    let param = ResultUnion::Case20 { float_res: 8.0f32 };
    let value = rpc_con.UNION_PARAM(&param).expect("Rpc-Call failed");
    assert!(value == 8, "Union Test failed");
}
