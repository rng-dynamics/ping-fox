extern crate bindgen;
extern crate cc;

use std::env;
use std::path::PathBuf;

fn main() {
    cc::Build::new()
        .file("c_src/dgram_socket_api.c")
        .compile("libdgram_socket_api.a");
    let bindings = bindgen::Builder::default()
        .header("c_src/dgram_socket_api.h")
        .generate()
        .expect("Unable to generate bindings.");
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Could not write bindings.");
}
