use std::path::Path;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    __check();

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=LIBRAW_DIR");
    println!("cargo:rerun-if-env-changed=LIBRAW_REPO");

    let _out_dir = &std::env::var_os("OUT_DIR").unwrap();
    let out_dir = Path::new(_out_dir);

    #[cfg(all(feature = "clone", not(feature = "no-build")))]
    clone(out_dir);
    // #[cfg(feature = "clone")]
    // let __head = commit(out_dir.join("libraw"))?;

    #[cfg(all(feature = "bindgen", not(feature = "no-build")))]
    bindings(out_dir)?;

    #[cfg(all(feature = "build", not(feature = "no-build")))]
    build(out_dir)?;

    let _ = out_dir;

    Ok(())
}

#[cfg(all(feature = "build", not(feature = "no-build")))]
fn build(out_dir: &Path) -> Result<()> {
    std::env::set_current_dir(out_dir).expect("Unable to set current dir");

    let mut libraw = cc::Build::new();
    libraw.cpp(true);
    libraw.include("libraw/");

    println!(
        "cargo:include={}",
        out_dir.join("libraw").join("libraw").display()
    );

    // #[cfg(feature = "jasper")]
    // if let Ok(jasper) = std::env::var("DEP_JASPER_INCLUDE") {
    //     let paths = std::env::split_paths(&jasper).collect::<Vec<_>>();
    //     for path in paths {
    //         if !path.exists() {
    //             panic!("{:?}", path);
    //         }
    //         libraw.include(&path);
    //     }
    //     // // panic!("{:?}", std::env::split_paths(&jasper).collect::<Vec<_>>());
    //     // panic!();
    //     libraw.includes(std::env::split_paths(&jasper));
    // }
    // if let Ok(jasper) = pkg_config::Config::new().probe("jasper") {
    //     libraw.includes(jasper.include_paths);
    // } else {
    //     eprintln!("jasper not found");
    //     return Err("jasper not found".into());
    // }

    #[cfg(feature = "zlib")]
    if let Ok(path) = dbg!(std::env::var("DEP_Z_INCLUDE")) {
        libraw.include(path);
    }
    // for (key, value) in std::env::vars() {
    //     dbg!(key, value);
    // }
    // panic!();

    // Fix builds on msys2
    #[cfg(windows)]
    libraw.define("HAVE_BOOLEAN", None);
    #[cfg(windows)]
    libraw.define("LIBRAW_WIN32_DLLDEFS", None);
    #[cfg(windows)]
    libraw.define("LIBRAW_BUILDLIB", None);

    #[cfg(feature = "jpeg")]
    if let Ok(path) = std::env::var("DEP_JPEG_INCLUDE") {
        libraw.includes(std::env::split_paths(&path));
    }
    libraw.file("libraw/src/decoders/canon_600.cpp");
    libraw.file("libraw/src/decoders/crx.cpp");
    libraw.file("libraw/src/decoders/decoders_dcraw.cpp");
    libraw.file("libraw/src/decoders/decoders_libraw.cpp");
    libraw.file("libraw/src/decoders/decoders_libraw_dcrdefs.cpp");
    libraw.file("libraw/src/decoders/dng.cpp");
    libraw.file("libraw/src/decoders/fp_dng.cpp");
    libraw.file("libraw/src/decoders/fuji_compressed.cpp");
    libraw.file("libraw/src/decoders/generic.cpp");
    libraw.file("libraw/src/decoders/kodak_decoders.cpp");
    libraw.file("libraw/src/decoders/load_mfbacks.cpp");
    if Path::new("libraw/src/decoders/pana8.cpp").exists() {
        libraw.file("libraw/src/decoders/pana8.cpp");
    }
    if Path::new("libraw/src/decoders/sonycc.cpp").exists() {
        libraw.file("libraw/src/decoders/sonycc.cpp");
    }
    if Path::new("libraw/src/decompressors/losslessjpeg.cpp").exists() {
        libraw.file("libraw/src/decompressors/losslessjpeg.cpp");
    }
    libraw.file("libraw/src/decoders/smal.cpp");
    libraw.file("libraw/src/decoders/unpack.cpp");
    libraw.file("libraw/src/decoders/unpack_thumb.cpp");
    libraw.file("libraw/src/demosaic/aahd_demosaic.cpp");
    libraw.file("libraw/src/demosaic/ahd_demosaic.cpp");
    libraw.file("libraw/src/demosaic/dcb_demosaic.cpp");
    libraw.file("libraw/src/demosaic/dht_demosaic.cpp");
    libraw.file("libraw/src/demosaic/misc_demosaic.cpp");
    libraw.file("libraw/src/demosaic/xtrans_demosaic.cpp");
    libraw.file("libraw/src/integration/dngsdk_glue.cpp");
    libraw.file("libraw/src/integration/rawspeed_glue.cpp");
    libraw.file("libraw/src/metadata/adobepano.cpp");
    libraw.file("libraw/src/metadata/canon.cpp");
    libraw.file("libraw/src/metadata/ciff.cpp");
    libraw.file("libraw/src/metadata/cr3_parser.cpp");
    libraw.file("libraw/src/metadata/epson.cpp");
    libraw.file("libraw/src/metadata/exif_gps.cpp");
    libraw.file("libraw/src/metadata/fuji.cpp");
    libraw.file("libraw/src/metadata/hasselblad_model.cpp");
    libraw.file("libraw/src/metadata/identify.cpp");
    libraw.file("libraw/src/metadata/identify_tools.cpp");
    libraw.file("libraw/src/metadata/kodak.cpp");
    libraw.file("libraw/src/metadata/leica.cpp");
    libraw.file("libraw/src/metadata/makernotes.cpp");
    libraw.file("libraw/src/metadata/mediumformat.cpp");
    libraw.file("libraw/src/metadata/minolta.cpp");
    libraw.file("libraw/src/metadata/misc_parsers.cpp");
    libraw.file("libraw/src/metadata/nikon.cpp");
    libraw.file("libraw/src/metadata/normalize_model.cpp");
    libraw.file("libraw/src/metadata/olympus.cpp");
    libraw.file("libraw/src/metadata/p1.cpp");
    libraw.file("libraw/src/metadata/pentax.cpp");
    libraw.file("libraw/src/metadata/samsung.cpp");
    libraw.file("libraw/src/metadata/sony.cpp");
    libraw.file("libraw/src/metadata/tiff.cpp");
    libraw.file("libraw/src/postprocessing/aspect_ratio.cpp");
    libraw.file("libraw/src/postprocessing/dcraw_process.cpp");
    libraw.file("libraw/src/postprocessing/mem_image.cpp");
    libraw.file("libraw/src/postprocessing/postprocessing_aux.cpp");
    //libraw.file("libraw/src/postprocessing/postprocessing_ph.cpp");
    libraw.file("libraw/src/postprocessing/postprocessing_utils.cpp");
    libraw.file("libraw/src/postprocessing/postprocessing_utils_dcrdefs.cpp");
    libraw.file("libraw/src/preprocessing/ext_preprocess.cpp");
    //libraw.file("libraw/src/preprocessing/preprocessing_ph.cpp");
    libraw.file("libraw/src/preprocessing/raw2image.cpp");
    libraw.file("libraw/src/preprocessing/subtract_black.cpp");
    libraw.file("libraw/src/tables/cameralist.cpp");
    libraw.file("libraw/src/tables/colorconst.cpp");
    libraw.file("libraw/src/tables/colordata.cpp");
    libraw.file("libraw/src/tables/wblists.cpp");
    libraw.file("libraw/src/utils/curves.cpp");
    libraw.file("libraw/src/utils/decoder_info.cpp");
    libraw.file("libraw/src/utils/init_close_utils.cpp");
    libraw.file("libraw/src/utils/open.cpp");
    libraw.file("libraw/src/utils/phaseone_processing.cpp");
    libraw.file("libraw/src/utils/read_utils.cpp");
    libraw.file("libraw/src/utils/thumb_utils.cpp");
    libraw.file("libraw/src/utils/utils_dcraw.cpp");
    libraw.file("libraw/src/utils/utils_libraw.cpp");
    libraw.file("libraw/src/write/apply_profile.cpp");
    libraw.file("libraw/src/write/file_write.cpp");
    libraw.file("libraw/src/write/tiff_writer.cpp");
    //libraw.file("libraw/src/write/write_ph.cpp");
    libraw.file("libraw/src/x3f/x3f_parse_process.cpp");
    libraw.file("libraw/src/x3f/x3f_utils_patched.cpp");
    libraw.file("libraw/src/libraw_c_api.cpp");
    // libraw.file("libraw/src/libraw_cxx.cpp");
    libraw.file("libraw/src/libraw_datastream.cpp");

    libraw.warnings(false);
    libraw.extra_warnings(false);
    // do I really have to supress all of these?
    libraw.flag_if_supported("-Wno-format-truncation");
    libraw.flag_if_supported("-Wno-unused-result");
    libraw.flag_if_supported("-Wno-format-overflow");
    // libraw.flag_if_supported("-fopenmp");

    // thread safety
    libraw.flag_if_supported("-pthread");

    // Add libraries
    #[cfg(feature = "jpeg")]
    libraw.flag("-DUSE_JPEG");

    #[cfg(feature = "zlib")]
    libraw.flag("-DUSE_ZLIB");

    // #[cfg(feature = "jasper")]
    // libraw.flag("-DUSE_JASPER");

    #[cfg(target_os = "linux")]
    libraw.cpp_link_stdlib("stdc++");
    #[cfg(target_os = "macos")]
    libraw.cpp_link_stdlib("c++");

    #[cfg(windows)]
    libraw.static_crt(true);
    #[cfg(unix)]
    libraw.static_flag(true);

    libraw.compile("raw_r");

    println!(
        "cargo:rustc-link-search=native={}",
        out_dir.join("lib").display()
    );
    println!("cargo:rustc-link-lib=static=raw_r");
    // #[cfg(feature = "jpeg")]
    // println!("cargo:rustc-link-lib=static=mozjpeg80");
    #[cfg(feature = "jpeg")]
    println!("cargo:rustc-link-lib=static=mozjpeg80");
    #[cfg(feature = "zlib")]
    println!("cargo:rustc-link-lib=static=z");

    Ok(())
}

#[cfg(feature = "bindgen")]
fn bindings(out_dir: &Path) -> Result<()> {
    std::env::set_current_dir(out_dir).expect("Unable to set current dir");

    println!(
        "cargo:include={}",
        out_dir.join("libraw").join("libraw").display()
    );

    let bindings = bindgen::Builder::default()
        .header("libraw/libraw/libraw.h")
        .use_core()
        .ctypes_prefix("libc")
        .generate_comments(true)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        // API improvements
        .derive_eq(true)
        .size_t_is_usize(true)
        // these are never part of the API
        .blocklist_function("_.*")
        // consts creating duplications
        .blocklist_item("FP_NAN")
        .blocklist_item("FP_INFINITE")
        .blocklist_item("FP_ZERO")
        .blocklist_item("FP_SUBNORMAL")
        .blocklist_item("FP_NORMAL")
        .blocklist_item("__mingw_ldbl_type_t")
        // Rust doesn't support long double, and bindgen can't skip it
        // https://github.com/rust-lang/rust-bindgen/issues/1549
        .blocklist_function("acoshl")
        .blocklist_function("acosl")
        .blocklist_function("asinhl")
        .blocklist_function("asinl")
        .blocklist_function("atan2l")
        .blocklist_function("atanhl")
        .blocklist_function("atanl")
        .blocklist_function("cbrtl")
        .blocklist_function("ceill")
        .blocklist_function("copysignl")
        .blocklist_function("coshl")
        .blocklist_function("cosl")
        .blocklist_function("dreml")
        .blocklist_function("ecvt_r")
        .blocklist_function("erfcl")
        .blocklist_function("erfl")
        .blocklist_function("exp2l")
        .blocklist_function("expl")
        .blocklist_function("expm1l")
        .blocklist_function("fabsl")
        .blocklist_function("fcvt_r")
        .blocklist_function("fdiml")
        .blocklist_function("finitel")
        .blocklist_function("floorl")
        .blocklist_function("fmal")
        .blocklist_function("fmaxl")
        .blocklist_function("fminl")
        .blocklist_function("fmodl")
        .blocklist_function("frexpl")
        .blocklist_function("gammal")
        .blocklist_function("hypotl")
        .blocklist_function("ilogbl")
        .blocklist_function("isinfl")
        .blocklist_function("isnanl")
        .blocklist_function("j0l")
        .blocklist_function("j1l")
        .blocklist_function("jnl")
        .blocklist_function("ldexpl")
        .blocklist_function("lgammal")
        .blocklist_function("lgammal_r")
        .blocklist_function("llrintl")
        .blocklist_function("llroundl")
        .blocklist_function("log10l")
        .blocklist_function("log1pl")
        .blocklist_function("log2l")
        .blocklist_function("logbl")
        .blocklist_function("logl")
        .blocklist_function("lrintl")
        .blocklist_function("lroundl")
        .blocklist_function("modfl")
        .blocklist_function("nanl")
        .blocklist_function("nearbyintl")
        .blocklist_function("nextafterl")
        .blocklist_function("nexttoward")
        .blocklist_function("nexttowardf")
        .blocklist_function("nexttowardl")
        .blocklist_function("powl")
        .blocklist_function("qecvt")
        .blocklist_function("qecvt_r")
        .blocklist_function("qfcvt")
        .blocklist_function("qfcvt_r")
        .blocklist_function("qgcvt")
        .blocklist_function("remainderl")
        .blocklist_function("remquol")
        .blocklist_function("rintl")
        .blocklist_function("roundl")
        .blocklist_function("scalbl")
        .blocklist_function("scalblnl")
        .blocklist_function("scalbnl")
        .blocklist_function("significandl")
        .blocklist_function("sinhl")
        .blocklist_function("sincosl")
        .blocklist_function("sinl")
        .blocklist_function("sqrtl")
        .blocklist_function("strtold")
        .blocklist_function("tanhl")
        .blocklist_function("tanl")
        .blocklist_function("tgammal")
        .blocklist_function("truncl")
        .blocklist_function("wcstold")
        .blocklist_function("y0l")
        .blocklist_function("y1l")
        .blocklist_function("ynl")
        .generate()
        .expect("Unable to generate bindings");

    bindings
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    #[cfg(feature = "copy")]
    bindings
        .write_to_file(
            #[cfg(target_os = "linux")]
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("src")
                .join("linux.rs"),
            #[cfg(target_os = "macos")]
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("src")
                .join("macos.rs"),
            #[cfg(target_family = "windows")]
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("src")
                .join("windows.rs"),
        )
        .expect("Failed to write bindings");
    Ok(())
}

#[cfg(all(feature = "clone", not(feature = "no-build")))]
fn clone(out_dir: &Path) {
    use std::process::{Command, Stdio};
    eprintln!("\x1b[31mCloning libraw\x1b[0m");
    let libraw_dir = std::env::var("LIBRAW_DIR");

    if let Ok(libraw_dir) = libraw_dir {
        Command::new("cp")
            .arg("-r")
            .arg(&libraw_dir)
            .arg(out_dir.join("libraw"))
            .stdout(Stdio::inherit())
            .output()
            .unwrap_or_else(|_| panic!("Failed to copy {}", libraw_dir));
    } else {
        let libraw_repo_url = std::env::var("LIBRAW_REPO")
            .unwrap_or_else(|_| String::from("https://github.com/libraw/libraw"));

        let _git_out = Command::new("git")
            .arg("clone")
            .arg("--depth")
            .arg("1")
            .arg(&libraw_repo_url)
            .arg(out_dir.join("libraw"))
            .stdout(Stdio::inherit())
            .output()
            .unwrap_or_else(|_| panic!("Failed to clone libraw from {}", libraw_repo_url));
    }
}

fn __check() {
    #[cfg(all(feature = "build", not(any(feature = "clone", feature = "tarball"))))]
    compile_error!("You need to have clone enabled to use build");
}

// #[cfg(feature = "clone")]
// fn commit(out_dir: impl AsRef<Path>) -> Result<git2::Oid> {
//     let repo = git2::Repository::open(out_dir)?;
//     let head = repo
//         .head()?
//         .target()
//         .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "HEAD not found"))?;

//     std::fs::write(
//         std::path::Path::new(&std::env::var_os("OUT_DIR").unwrap()).join("commit.rs"),
//         format!("const LIBRAW_COMMIT: &str = \"{head}\"").as_bytes(),
//     )?;

//     Ok(head)
// }
