use std::path::Path;
fn main() -> anyhow::Result<()> {
    let _out_dir = &std::env::var_os("OUT_DIR").unwrap();
    let out_dir = Path::new(_out_dir);
    cc::Build::new()
        .include("includes")
        .cpp(true)
        .file("exif/libread.cpp")
        .static_flag(true)
        .compile("read");
    println!("cargo:rustc-link-lib=static=read");
    println!(
        "cargo:rustc-link-search=native={}",
        out_dir.join("lib").display()
    );

    Ok(())
}
