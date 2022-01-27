# Rpc-Lib

Rpc-Lib compiles a definition of RPC-Functions in the SUN-RPC format into Rust-code and provides necessary network-code to run the application. It follows the standard [RFC-5531](https://datatracker.ietf.org/doc/html/rfc5531).

## Setup

* Write the RPC-Definition in the SUN-RPC Format
* Create the server-application. This can be done in C with `rpcgen`
* Make sure the Portmapper-service is installed and running before starting the server-application

## Example

The following example shows how to use Rpc-Lib.

```toml
# Cargo.toml
[package]
name = "rpc-math"
version = "0.1.0"
edition = "2021"

# See more keys and their definitions at https://doc.rust-lang.org/cargo/reference/manifest.html

[dependencies]
rpc-lib = { git = "https://git.rwth-aachen.de/acs/public/virtualization/rpc-lib/rpc-lib"}
```

```c
/* math.x */
program MATH {
    version VERS_1 {
        int ADD(int, int) = 1;
    } = 1;
} = 67908;
```

```rust
// src/main.rs
extern crate rpc_lib;
use rpc_lib::include_rpcl;

#[include_rpcl("math.x")]
struct RPCStruct;

fn main() {
    let mut rpc = RPCStruct::new("127.0.0.1").expect("Server not available");

    let result = rpc.ADD(&1, &2).expect("Rpc call failed");

    assert!(result == 3, "Add failed");
}
```

Build-Instructions:

Build with `cargo build`. Additional documentation can be generated with `cargo doc`.
