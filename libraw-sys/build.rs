use std::path::{Path, PathBuf};

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=LIBRAW_DIR");

    let _out_dir = &std::env::var_os("OUT_DIR").unwrap();
    let out_dir = Path::new(_out_dir);

    let libraw_dir = std::env::var("LIBRAW_DIR")
        .ok()
        .and_then(|p| {
            shellexpand::full(&p)
                .ok()
                .and_then(|p| dunce::canonicalize(p.to_string()).ok())
        })
        .unwrap_or(PathBuf::from(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/vendor"
        )));

    // println!("cargo:rerun-if-changed={}", libraw_dir.display());

    println!(
        "cargo:include={}",
        std::env::join_paths([
            Path::new(&libraw_dir).join("libraw"),
            Path::new(&libraw_dir).to_path_buf()
        ])
        .expect("Display")
        .to_string_lossy()
    );

    build(out_dir, &libraw_dir)?;

    #[cfg(all(feature = "bindgen"))]
    bindings(out_dir, &libraw_dir)?;

    let _ = out_dir;

    Ok(())
}

fn build(out_dir: impl AsRef<Path>, libraw_dir: impl AsRef<Path>) -> Result<()> {
    std::env::set_current_dir(out_dir.as_ref()).expect("Unable to set current dir");

    let mut libraw = cc::Build::new();
    libraw.cpp(true);
    libraw.include(libraw_dir.as_ref());

    #[cfg(feature = "zlib")]
    if let Ok(path) = std::env::var("DEP_Z_INCLUDE") {
        libraw.include(path);
    }

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
    // libraw.files(sources);
    // if Path::new("libraw/src/decoders/pana8.cpp").exists() {
    //     libraw.file("libraw/src/decoders/pana8.cpp");
    // }
    // if Path::new("libraw/src/decoders/sonycc.cpp").exists() {
    //     libraw.file("libraw/src/decoders/sonycc.cpp");
    // }
    // if Path::new("libraw/src/decompressors/losslessjpeg.cpp").exists() {
    //     libraw.file("libraw/src/decompressors/losslessjpeg.cpp");
    // }

    let sources = [
        "src/decoders/canon_600.cpp",
        "src/decoders/crx.cpp",
        "src/decoders/decoders_dcraw.cpp",
        "src/decoders/decoders_libraw.cpp",
        "src/decoders/decoders_libraw_dcrdefs.cpp",
        "src/decoders/dng.cpp",
        "src/decoders/fp_dng.cpp",
        "src/decoders/fuji_compressed.cpp",
        "src/decoders/generic.cpp",
        "src/decoders/kodak_decoders.cpp",
        "src/decoders/load_mfbacks.cpp",
        "src/decoders/pana8.cpp",
        "src/decoders/sonycc.cpp",
        "src/decompressors/losslessjpeg.cpp",
        "src/decoders/smal.cpp",
        "src/decoders/unpack.cpp",
        "src/decoders/unpack_thumb.cpp",
        "src/demosaic/aahd_demosaic.cpp",
        "src/demosaic/ahd_demosaic.cpp",
        "src/demosaic/dcb_demosaic.cpp",
        "src/demosaic/dht_demosaic.cpp",
        "src/demosaic/misc_demosaic.cpp",
        "src/demosaic/xtrans_demosaic.cpp",
        "src/integration/dngsdk_glue.cpp",
        "src/integration/rawspeed_glue.cpp",
        "src/metadata/adobepano.cpp",
        "src/metadata/canon.cpp",
        "src/metadata/ciff.cpp",
        "src/metadata/cr3_parser.cpp",
        "src/metadata/epson.cpp",
        "src/metadata/exif_gps.cpp",
        "src/metadata/fuji.cpp",
        "src/metadata/hasselblad_model.cpp",
        "src/metadata/identify.cpp",
        "src/metadata/identify_tools.cpp",
        "src/metadata/kodak.cpp",
        "src/metadata/leica.cpp",
        "src/metadata/makernotes.cpp",
        "src/metadata/mediumformat.cpp",
        "src/metadata/minolta.cpp",
        "src/metadata/misc_parsers.cpp",
        "src/metadata/nikon.cpp",
        "src/metadata/normalize_model.cpp",
        "src/metadata/olympus.cpp",
        "src/metadata/p1.cpp",
        "src/metadata/pentax.cpp",
        "src/metadata/samsung.cpp",
        "src/metadata/sony.cpp",
        "src/metadata/tiff.cpp",
        "src/postprocessing/aspect_ratio.cpp",
        "src/postprocessing/dcraw_process.cpp",
        "src/postprocessing/mem_image.cpp",
        "src/postprocessing/postprocessing_aux.cpp",
        //"src/postprocessing/postprocessing_ph.cpp",
        "src/postprocessing/postprocessing_utils.cpp",
        "src/postprocessing/postprocessing_utils_dcrdefs.cpp",
        "src/preprocessing/ext_preprocess.cpp",
        //"src/preprocessing/preprocessing_ph.cpp"
        "src/preprocessing/raw2image.cpp",
        "src/preprocessing/subtract_black.cpp",
        "src/tables/cameralist.cpp",
        "src/tables/colorconst.cpp",
        "src/tables/colordata.cpp",
        "src/tables/wblists.cpp",
        "src/utils/curves.cpp",
        "src/utils/decoder_info.cpp",
        "src/utils/init_close_utils.cpp",
        "src/utils/open.cpp",
        "src/utils/phaseone_processing.cpp",
        "src/utils/read_utils.cpp",
        "src/utils/thumb_utils.cpp",
        "src/utils/utils_dcraw.cpp",
        "src/utils/utils_libraw.cpp",
        "src/write/apply_profile.cpp",
        "src/write/file_write.cpp",
        "src/write/tiff_writer.cpp",
        //"src/write/write_ph.cpp"
        "src/x3f/x3f_parse_process.cpp",
        "src/x3f/x3f_utils_patched.cpp",
        "src/libraw_c_api.cpp",
        //"src/libraw_cxx.cpp"
        "src/libraw_datastream.cpp",
    ];

    let sources = sources
        .iter()
        .filter_map(|s| dunce::canonicalize(libraw_dir.as_ref().join(s)).ok())
        .collect::<Vec<_>>();

    if sources.is_empty() {
        panic!("Sources not found. Maybe try running \"git submodule update --init --recursive\"?");
    } else {
        sources
            .iter()
            .for_each(|s| println!("cargo:rerun-if-changed={}", s.display()));
    }

    libraw.files(sources);

    libraw.warnings(false);
    libraw.extra_warnings(false);
    // do I really have to supress all of these?
    libraw.flag_if_supported("-Wno-format-truncation");
    libraw.flag_if_supported("-Wno-unused-result");
    libraw.flag_if_supported("-Wno-format-overflow");
    #[cfg(feature = "openmp")]
    {
        libraw.define("LIBRAW_FORCE_OPENMP", None);
        std::env::var("DEP_OPENMP_FLAG")
            .unwrap()
            .split(' ')
            .for_each(|f| {
                libraw.flag(f);
            });
        if cfg!(target_os = "macos") {
            if libraw.get_compiler().is_like_apple_clang() {
                let homebrew_prefix =
                    PathBuf::from(std::env::var("HOMEBREW_PREFIX").unwrap_or_else(|_| {
                        if cfg!(target_arch = "aarch64") {
                            "/opt/homebrew".into()
                        } else {
                            "/usr/local".into()
                        }
                    }));

                if homebrew_prefix.join("opt/libomp/include").exists() {
                    libraw.include(homebrew_prefix.join("opt/libomp/include"));
                    println!(
                        "cargo:rustc-link-search={}{}opt/libomp/lib",
                        homebrew_prefix.display(),
                        std::path::MAIN_SEPARATOR
                    );
                    let statik = cfg!(feature = "openmp_static");
                    println!(
                        "cargo:rustc-link-lib{}=omp",
                        if statik { "=static" } else { "" }
                    );
                } else {
                    println!("cargo:warning:Unable to find libomp (maybe try installing libomp via homebrew?)")
                }
            }
        }
    }
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

    #[cfg(unix)]
    libraw.static_flag(true);

    #[cfg(windows)]
    libraw.static_crt(true);

    libraw.compile("raw_r");

    println!(
        "cargo:rustc-link-search=native={}",
        out_dir.as_ref().join("lib").display()
    );
    println!("cargo:rustc-link-lib=static=raw_r");
    #[cfg(feature = "jpeg")]
    if let Ok(name) = std::env::var("DEP_JPEG_NAME") {
        println!("cargo:rustc-link-lib=static={name}");
    }
    #[cfg(feature = "zlib")]
    println!("cargo:rustc-link-lib=static=z");

    Ok(())
}

#[cfg(feature = "bindgen")]
fn bindings(out_dir: impl AsRef<Path>, libraw_dir: impl AsRef<Path>) -> Result<()> {
    let bindings = bindgen::Builder::default()
        .header(
            libraw_dir
                .as_ref()
                .join("libraw")
                .join("libraw.h")
                .to_string_lossy(),
        )
        .use_core()
        .ctypes_prefix("libc")
        .generate_comments(true)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
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
        .write_to_file(out_dir.as_ref().join("bindings.rs"))
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

pub trait IsAppleClang {
    fn try_is_like_apple_clang(&self) -> Result<bool>;
    fn is_like_apple_clang(&self) -> bool {
        self.try_is_like_apple_clang()
            .expect("Failed to run compiler")
    }
}

impl IsAppleClang for cc::Tool {
    fn try_is_like_apple_clang(&self) -> Result<bool> {
        let output = std::process::Command::new(self.to_command().get_program())
            .arg("-v")
            .output()?;
        let stderr = String::from_utf8(output.stderr)?;
        Ok(stderr.starts_with("Apple") && (stderr.contains("clang") || self.is_like_clang()))
    }
}
