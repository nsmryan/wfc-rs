use std::env;
use std::path::PathBuf;

use cc;
use bindgen;

fn main() {
    let mut build = cc::Build::new();
    let build = build.file("csrc/wfc.c");
    let build = build.include("csrc/");

    build.compile("wfc");

    //let bindings = bindgen::Builder::default()
    //    .header("csrc/wfc.h")
    //    .generate()
    //    .expect("Unable to generate bindings!");

    //let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());

    //bindings.write_to_file(out_path.join("bindings.rs"))
    //    .expect("Could not write bindings.rs");

    //println!("cargo::rerun-if-changed=csrc/");
}

