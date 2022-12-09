use glob::glob;
use std::env;
use std::fs;
use std::path::PathBuf;

#[cfg(feature = "ugoira")]
mod ugoira {
    pub fn compile() {
        compile_converter();
        compile_x264();
        compile_libav();
    }

    fn compile_converter() {
        use std::path::Path;

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

        build.compile("ugoira");

        println!("cargo:rerun-if-changed=ugoira.cpp");
    }

    fn compile_x264() {
        let output_base_path = output();
        let source_dir = output_base_path.join("x264");
        let _ = std::fs::remove_dir_all(&source_dir);

        /* Build x264 from source */
        let _ = Command::new("git")
            .current_dir(&output_base_path)
            .args(&[
                "clone",
                "--depth=1",
                "https://code.videolan.org/videolan/x264.git",
                "x264",
            ])
            .status()
            .unwrap();

        let prefix_dir = output_base_path.join("x264-out");

        let _ = Command::new("sh")
            .current_dir(&source_dir)
            .args(&[
                "configure",
                &format!("--prefix={}", prefix_dir.display()),
                "--enable-static",
                "--disable-cli",
                "--disable-opencl",
                "--bit-depth=8",
                "--disable-avs",
                "--disable-swscale",
                "--disable-lavf",
                "--disable-ffms",
                "--disable-gpac",
                "--disable-lsmash",
            ])
            .status()
            .unwrap();

        let _ = Command::new("make")
            .current_dir(&source_dir)
            .arg("-j")
            .status()
            .unwrap();

        let _ = Command::new("make")
            .current_dir(&source_dir)
            .arg("install")
            .status()
            .unwrap();

        println!("cargo:rustc-link-search=native={}/lib", prefix_dir.display());

        println!("cargo:rustc-link-lib=static=x264");
    }

    extern crate cc;

    use std::env;
    use std::fs::{self, File};
    use std::io::{self, BufRead, BufReader};
    use std::path::PathBuf;
    use std::process::Command;
    use std::str;

    const BRANCH: &str = "release/5.1";

    fn output() -> PathBuf {
        PathBuf::from(env::var("OUT_DIR").unwrap())
    }

    fn source() -> PathBuf {
        output().join("ffmpeg")
    }

    fn search() -> PathBuf {
        let mut absolute = env::current_dir().unwrap();
        absolute.push(&output());
        absolute.push("dist");

        absolute
    }

    fn fetch() -> io::Result<()> {
        let output_base_path = output();
        let _ = std::fs::remove_dir_all(output_base_path.join("ffmpeg"));
        let status = Command::new("git")
            .current_dir(&output_base_path)
            .arg("clone")
            .arg("--depth=1")
            .arg("-b")
            .arg(BRANCH)
            .arg("https://github.com/FFmpeg/FFmpeg")
            .arg("ffmpeg")
            .status()?;

        if status.success() {
            Ok(())
        } else {
            Err(io::Error::new(io::ErrorKind::Other, "fetch failed"))
        }
    }

    fn build() -> io::Result<()> {
        let source_dir = source();

        // Command's path is not relative to command's current_dir
        let configure_path = source_dir.join("configure");
        assert!(configure_path.exists());
        let mut configure = Command::new(&configure_path);
        configure.current_dir(&source_dir);

        configure.arg(format!("--prefix={}", search().to_string_lossy()));

        if env::var("TARGET").unwrap() != env::var("HOST").unwrap() {
            // Rust targets are subtly different than naming scheme for compiler prefixes.
            // The cc crate has the messy logic of guessing a working prefix,
            // and this is a messy way of reusing that logic.
            let cc = cc::Build::new();
            let compiler = cc.get_compiler();
            let compiler = compiler.path().file_stem().unwrap().to_str().unwrap();
            let suffix_pos = compiler.rfind('-').unwrap(); // cut off "-gcc"
            let prefix = compiler[0..suffix_pos].trim_end_matches("-wr"); // "wr-c++" compiler

            configure.arg(format!("--cross-prefix={}-", prefix));
            configure.arg(format!(
                "--arch={}",
                env::var("CARGO_CFG_TARGET_ARCH").unwrap()
            ));
            configure.arg(format!(
                "--target_os={}",
                env::var("CARGO_CFG_TARGET_OS").unwrap()
            ));
        }

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

        // do not build docs
        configure.arg("--disable-doc");

        // disable all components
        configure.arg("--disable-everything");

        configure.arg("--enable-gpl");

        // jpeg input
        configure.arg("--enable-demuxer=image_jpeg_pipe");
        configure.arg("--enable-parser=mjpeg");
        configure.arg("--enable-decoder=mjpeg");

        // mp4 output
        configure.arg("--enable-muxer=mp4");
        configure.arg("--enable-encoder=libx264");
        configure.arg("--enable-libx264");

        configure.arg("--enable-avcodec");
        configure.arg("--enable-avformat");
        configure.arg("--enable-swresample");
        configure.arg("--enable-swscale");

        // run ./configure
        let output = configure
            .output()
            .unwrap_or_else(|_| panic!("{:?} failed", configure));
        if !output.status.success() {
            println!("configure: {}", String::from_utf8_lossy(&output.stdout));

            return Err(io::Error::new(
                io::ErrorKind::Other,
                format!(
                    "configure failed {}",
                    String::from_utf8_lossy(&output.stderr)
                ),
            ));
        }

        // run make
        if !Command::new("make")
            .arg("-j")
            .current_dir(&source())
            .status()?
            .success()
        {
            return Err(io::Error::new(io::ErrorKind::Other, "make failed"));
        }

        // run make install
        if !Command::new("make")
            .current_dir(&source())
            .arg("install")
            .status()?
            .success()
        {
            return Err(io::Error::new(io::ErrorKind::Other, "make install failed"));
        }

        Ok(())
    }

    fn compile_libav() {
        println!(
            "cargo:rustc-link-search=native={}",
            search().join("lib").to_string_lossy()
        );
        for lib in &["avcodec", "avformat", "avutil", "swresample", "swscale"] {
            println!("cargo:rustc-link-lib=static={}", lib);
        }
        if fs::metadata(&search().join("lib").join("libavutil.a")).is_err()
        {
            fs::create_dir_all(&output()).expect("failed to create build directory");
            fetch().unwrap();
            build().unwrap();
        }

        // Check additional required libraries.
        {
            let config_mak = source().join("ffbuild/config.mak");
            let file = File::open(config_mak).unwrap();
            let reader = BufReader::new(file);
            let extra_libs = reader
                .lines()
                .find(|line| line.as_ref().unwrap().starts_with("EXTRALIBS"))
                .map(|line| line.unwrap())
                .unwrap();

            let linker_args = extra_libs.split('=').last().unwrap().split(' ');
            let include_libs = linker_args
                .filter(|v| v.starts_with("-l"))
                .map(|flag| &flag[2..]);

            for lib in include_libs {
                println!("cargo:rustc-link-lib={}", lib);
            }
        }
    }
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
    ugoira::compile();

    compile_css();
}
