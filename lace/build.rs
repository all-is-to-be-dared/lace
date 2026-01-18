extern crate cc;

use std::path::PathBuf;

pub fn main() {
    println!("cargo:rerun-if-changed=.cargo/config.toml");

    let in_path = "bcm2711.ld";
    let out_dir = std::env::var("OUT_DIR")
        .map(PathBuf::from)
        .expect("No OUT_DIR defined by the environment");
    let out_path = out_dir.join("bcm2711.ld.out");

    let out: Vec<u8> = cc::Build::new()
        .flags(["-x", "c", "-CC", "-E"])
        .file(in_path)
        .expand();
    let out = String::from_utf8(out).expect("Expanded linker script contains invalid UTF-8");
    let out = out.replace("\"\"", "");
    std::fs::write(&out_path, out.as_bytes()).expect("Failed to write preprocessed linker script");

    println!("cargo:rerun-if-changed={in_path}");
    println!("cargo:rustc-link-arg=-T{}", out_path.display());

    // cat $< | clang-21 -E -CC - > $@
    // sed -i .bak "s/\"\"//g" $@
}
