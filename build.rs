extern crate cc;

use std::process::Command;

fn main() {
    // Compile libtirpc
    Command::new("make")
        .current_dir("submodules/")
        .args(&["libtirpc"])
        .status()
        .expect("Compiling libtirpc failed!");
}
