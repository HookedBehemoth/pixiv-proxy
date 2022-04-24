use glob::glob;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let is_debug = env::var_os("PROFILE") == Some("debug".into());

    let mut build = cc::Build::new();

    if build.get_compiler().is_like_msvc() {
        env::set_var("CXXFLAGS", "/std:c++17");
    } else {
        env::set_var("CXXFLAGS", "-std=c++17");
    }

    build
        .cpp(true)
        .file("ugoira.cpp")
        .define("__STDC_CONSTANT_MACROS", None)
        .opt_level(if is_debug { 0 } else { 3 });

    if build.get_compiler().is_like_msvc() {
        let ffmpeg_dir = env::var("FFMPEG_DIR").unwrap();

        let include_dir = Path::new(&ffmpeg_dir).join("include");
        build.include(include_dir.display().to_string());

        let lib_dir = Path::new(&ffmpeg_dir).join("lib");
        println!("cargo:rustc-link-search=native={}", lib_dir.display());
    }

    build.compile("ugoira.a");

    println!("cargo:rerun-if-changed=ugoira.cpp");

    for lib in &["avcodec", "avformat", "avutil", "swresample", "swscale"] {
        println!("cargo:rustc-link-lib={}", lib);
    }

    let src: &str = "sass/main.scss";
    let dst: &str = "main.css";
    let out_dir: PathBuf = PathBuf::from(env::var_os("OUT_DIR").unwrap());
    for entry in glob("sass/**/*.scss").expect("Failed to glob") {
        println!("cargo:rerun-if-changed={}", entry.unwrap().display());
    }

    /* Compress css in release mode */
    let options = grass::Options::default().style(if is_debug {
        grass::OutputStyle::Expanded
    } else {
        grass::OutputStyle::Compressed
    });

    let css = grass::from_path(src, &options).unwrap();
    let dst = out_dir.join(dst);
    fs::write(dst, &css).unwrap();
}
