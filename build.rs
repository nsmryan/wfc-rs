use cc;

fn main() {
    let mut build = cc::Build::new();
    let build = build.file("csrc/wfc.c");
    let build = build.include("wfc/");
    let build = build.include("csrc/");
    let build = build.flag("-Wno-unused-function");
    let build = build.flag("-Wno-unused-variable");
    let build = build.flag("-Wno-unused-parameter");
    let build = build.flag("-Wno-unused-label");

    build.compile("wfc");
}

