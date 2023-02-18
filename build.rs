extern crate bindgen;
extern crate cargo_emit;
extern crate cc;

use std::env;
use std::path::PathBuf;

fn main() {
    cargo_emit::rerun_if_changed!("c_src/dgram_socket_api.c", "c_src/dgram_socket_api.h",);

    cc::Build::new()
        .file("c_src/dgram_socket_api.c")
        .compile("libdgram_socket_api.a");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindgen::Builder::default()
        .header("c_src/dgram_socket_api.h")
        .generate()
        .expect("Unable to generate bindings.") // -> bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Could not write bindings.");
}
