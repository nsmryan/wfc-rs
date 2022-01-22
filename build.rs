use cc;

fn main() {
    let mut build = cc::Build::new();
    let build = build.file("csrc/wfc.c");
    let build = build.include("wfc/");

    build.compile("wfc");
}

