extern crate bindgen;
extern crate cargo_emit;
extern crate cc;

use std::env;
use std::path::PathBuf;

fn main() {
    cargo_emit::rerun_if_changed!("extern/icmp_dgram.c", "extern/icmp_dgram.h",);

    cc::Build::new().file("extern/icmp_dgram.c").compile("libicmp_dgram.a");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindgen::Builder::default()
        .header("extern/icmp_dgram.h")
        .generate()
        .expect("Unable to generate bindings.") // -> bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Could not write bindings.");
}
