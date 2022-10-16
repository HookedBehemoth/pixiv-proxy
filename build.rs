use glob::glob;
use std::env;
use std::fs;
use std::path::PathBuf;

#[cfg(feature = "ugoira")]
fn compile_ugoira() {
    use std::path::Path;
    use std::process::Command;

    fn output() -> PathBuf {
        PathBuf::from(env::var("OUT_DIR").unwrap())
    }

    fn search() -> PathBuf {
        let mut absolute = env::current_dir().unwrap();
        absolute.push(&output());
        absolute.push("dist");

        absolute
    }

    let output = output();
    let search_dir = search();
    let include_dir = search().join("include");
    let lib_dir = search().join("lib");

    fn set_non_native(configure: &mut Command) {
        if env::var("TARGET").unwrap() != env::var("HOST").unwrap() {
            panic!("Non native target is broken rn");
            // Rust targets are subtly different than naming scheme for compiler prefixes.
            // The cc crate has the messy logic of guessing a working prefix,
            // and this is a messy way of reusing that logic.
            let cc = cc::Build::new();
            let compiler = cc.get_compiler();
            let compiler = compiler.path().file_stem().unwrap().to_str().unwrap();
            let suffix_pos = compiler.rfind('-').unwrap(); // cut off "-gcc"
            let prefix = compiler[0..suffix_pos].trim_end_matches("-wr"); // "wr-c++" compiler

            env::set_var("CC", "musl-gcc -static");

            // configure.arg(format!("--cross-prefix={}-", prefix));
            configure.arg(format!(
                "--arch={}",
                env::var("CARGO_CFG_TARGET_ARCH").unwrap()
            ));
            configure.arg(format!(
                "--target_os={}",
                env::var("CARGO_CFG_TARGET_OS").unwrap()
            ));
        }
    }

    fn clone(url: &str, folder: &PathBuf) {
        if !Command::new("git")
            .current_dir(&folder)
            .arg("clone")
            .arg("--depth=1")
            .arg(url)
            .status()
            .unwrap()
            .success()
        {
            panic!("fetch failed");
        }
    }

    fn make_install(path: &PathBuf) {
        // run make
        if !Command::new("make")
            .arg("-j")
            .current_dir(path)
            .status()
            .unwrap()
            .success()
        {
            panic!("Failed to build {:?}", path);
        }

        // run make install
        if !Command::new("make")
            .current_dir(path)
            .arg("install")
            .status()
            .unwrap()
            .success()
        {
            panic!("Failed to install {:?}", path);
        }
    }

    if fs::metadata(lib_dir.join("libavutil.a")).is_err() {
        let source_dir = output.join("FFmpeg");
        let _ = std::fs::remove_dir_all(&source_dir);
        clone("https://github.com/FFmpeg/FFmpeg", &output);

        let configure_path = source_dir.join("configure");
        assert!(configure_path.exists());
        let mut configure = Command::new(&configure_path);
        configure.current_dir(&source_dir);
        configure.arg(format!("--prefix={}", search_dir.to_string_lossy()));

        set_non_native(&mut configure);

        // control debug build
        if env::var("DEBUG").is_ok() {
            configure.arg("--enable-debug");
            configure.arg("--disable-stripping");
        } else {
            configure.arg("--disable-debug");
            configure.arg("--enable-stripping");
        }

        // make it static
        configure.arg("--enable-static");
        configure.arg("--disable-shared");

        configure.arg("--enable-pic");

        // stop autodetected libraries enabling themselves, causing linking errors
        configure.arg("--disable-autodetect");

        // do not build programs since we don't need them
        configure.arg("--disable-programs");

        // Allow GPL
        configure.arg("--enable-gpl");

        // Enable x264 and openjpeg
        configure.arg("--enable-libx264");
        // configure.arg("--enable-libopenjpeg");

        configure.output().unwrap();

        make_install(&source_dir);
    }

    if fs::metadata(lib_dir.join("libx264.a")).is_err() {
        let source_dir = output.join("x264");

        let _ = std::fs::remove_dir_all(&source_dir);
        clone("https://code.videolan.org/videolan/x264.git", &output);

        let configure_path = source_dir.join("configure");
        assert!(configure_path.exists());
        let mut configure = Command::new(&configure_path);
        configure.current_dir(&source_dir);
        configure.arg(format!("--prefix={}", search_dir.to_string_lossy()));

        set_non_native(&mut configure);

        configure.arg("--enable-static");
        configure.arg("--disable-cli");
        configure.arg("--disable-opencl");
        configure.arg("--disable-bashcompletion");
        configure.arg("--enable-pic");

        configure.output().unwrap();

        make_install(&source_dir);
    }

    env::remove_var("CC");

    if fs::metadata(lib_dir.join("ugoira.a")).is_err() {
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
            .opt_level(if is_debug { 0 } else { 3 })
            .include(&include_dir)
            .compile("ugoira.a");
    }

    println!("cargo:rerun-if-changed=ugoira.cpp");

    for lib in &[
        "avcodec",
        "avformat",
        "avutil",
        "swresample",
        "swscale",
        "x264",
    ] {
        println!("cargo:rustc-link-lib=static={}", lib);
    }
    println!(
        "cargo:rustc-link-search=native={}",
        lib_dir.to_string_lossy()
    );
}

fn compile_css() {
    let is_debug = env::var_os("PROFILE") == Some("debug".into());

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

fn main() {
    #[cfg(feature = "ugoira")]
    compile_ugoira();

    compile_css();
}
