use glob::glob;
use std::env;
use std::fs;
use std::path::{PathBuf, Path};

fn main() {
    let is_debug = env::var_os("PROFILE") == Some("debug".into());

    let ffmpeg_dir = env::var("FFMPEG_DIR").unwrap();

    env::set_var("CXXFLAGS", "/std:c++latest");

    let include_dir = Path::new(&ffmpeg_dir).join("include");
    cc::Build::new()
        .cpp(true)
        .include(include_dir.display().to_string())
        .define("__STDC_CONSTANT_MACROS", None)
        .file("ugoira.cpp")
        .opt_level(if is_debug { 0 } else { 3 })
        .compile("ugoira.a");
    
    println!("cargo:rerun-if-changed=ugoira.cpp");

    let lib_path = Path::new(&ffmpeg_dir).join("lib");
    println!("cargo:rustc-link-search=native={}", lib_path.display());

    for lib in &["avcodec", "avformat", "avutil", "swresample", "swscale"] {
        println!("cargo:rustc-link-lib=static={}", lib);
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
