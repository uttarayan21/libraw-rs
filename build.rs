use std::path::Path;
fn main() -> anyhow::Result<()> {
    let _out_dir = &std::env::var_os("OUT_DIR").unwrap();
    let out_dir = Path::new(_out_dir);

    #[cfg(feature = "exif")]
    libread(out_dir)?;

    Ok(())
}

#[cfg(feature = "exif")]
pub fn libread(out_dir: impl AsRef<Path>) -> anyhow::Result<()> {
    let mut libread = cc::Build::new();
    libread
        .include("includes")
        .cpp(true)
        .file("exif/libread.cpp")
        .static_flag(true)
        .shared_flag(false)
        .cpp_set_stdlib("c++");

    #[cfg(windows)]
    libread.static_crt(true);

    libread.compile("read");

    println!("cargo:rustc-link-lib=static=read");
    println!(
        "cargo:rustc-link-search=native={}",
        out_dir.as_ref().join("lib").display()
    );

    Ok(())
}
